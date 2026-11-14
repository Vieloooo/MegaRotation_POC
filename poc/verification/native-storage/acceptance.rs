//! Independent NATIVE-STORAGE S1–S10 vectors. No witness/outbox/engine claim.
use bytes::Buf;
use commonware_codec::{Copying, DecodeExt, Encode};
use commonware_consensus::{
    simplex::{
        scheme::bls12381_multisig::Scheme,
        types::{Artifact, Finalize, Notarize, Nullify, Proposal},
    },
    types::{Epoch, Round, View},
};
use commonware_cryptography::{
    Signer as _,
    bls12381::primitives::{ops, variant::MinSig},
    certificate::{Scheme as _, Verifier as _},
    ed25519::{PrivateKey, PublicKey},
    sha256::Digest as PayloadDigest,
};
use commonware_parallel::Sequential;
use commonware_runtime::{
    Blob, BlobVersion, Clock, Error as NativeError, Handle, IoBuf, IoBufMut, ReadOptions,
    Runner as _, Spawner, Storage, Supervisor, WriteOptions,
    buffer::paged::{self, CacheRef},
    tokio::{Config, Runner},
};
use commonware_storage::journal::segmented::variable::{Config as JournalConfig, Journal};
use commonware_utils::{NZUsize, ordered::BiMap};
use futures::StreamExt;
use poc_native_runtime::{
    ClockSample, ControlledDriver, Error, Event, Grant, Input, IoOperation, IoResult, Profile,
    SimContext,
};
use rand_chacha::ChaCha20Rng;
use rand_core::{SeedableRng, TryRng};
use serde_json::{Value, json};
use sha2::{Digest as _, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    future::{Future, poll_fn},
    io::IoSlice,
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex, OnceLock,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    task::{Context, Poll, Wake, Waker},
    time::Duration,
};

const T: u64 = 1_000_000;
const CID: [u8; 32] = [0x33; 32];
const STOP: i64 = 2_000_000;
static NEXT: AtomicU64 = AtomicU64::new(0);
static ROOT: OnceLock<PathBuf> = OnceLock::new();
fn place(label: &str) -> PathBuf {
    let root = ROOT.get_or_init(|| {
        let p = std::env::var_os("NATIVE_STORAGE_VERIFY_ARTIFACTS")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                std::env::temp_dir().join(format!("native-storage-verifier-{}", std::process::id()))
            });
        fs::create_dir_all(&p).unwrap();
        p
    });
    let p = root.join(format!("{label}-{}", NEXT.fetch_add(1, Ordering::SeqCst)));
    fs::create_dir(&p).unwrap();
    fs::create_dir(p.join("store")).unwrap();
    p
}
fn cfg(path: &Path) -> Config {
    Config::default()
        .with_worker_threads(1)
        .with_storage_directory(path.join("store"))
}
fn hash(domain: &str, parts: &[&[u8]]) -> [u8; 32] {
    let mut h = Sha256::new();
    for p in std::iter::once(domain.as_bytes()).chain(parts.iter().copied()) {
        h.update((p.len() as u64).to_be_bytes());
        h.update(p);
    }
    h.finalize().into()
}
fn profile(seed: u64, inc: u64) -> Profile {
    Profile {
        run_id: hash("storage-run", &[&seed.to_be_bytes()]),
        config_hash: hash("storage-config", &[&seed.to_be_bytes()]),
        run_seed: seed,
        node: 0,
        incarnation: inc,
        epsilon_us: 1000,
    }
}
fn calibrate(d: &mut ControlledDriver, p: &Profile) {
    d.grant(
        Grant { id: 1, tick_us: T },
        Input::RequestSample,
        |_| Ok(()),
    )
    .unwrap();
    let sample = ClockSample {
        run_id: p.run_id,
        node: p.node,
        incarnation: p.incarnation,
        clock_domain: hash(
            "I1/clock-domain/v1",
            &[&p.run_id, &p.config_hash, &p.node.to_be_bytes()],
        ),
        rate_profile: hash("I1/rate/v1", &[b"fixed-rate-one-v1"]),
        request_id: 1,
        sequence: 1,
        as_of_tick_us: T,
        node_time_at_as_of_us: T as i64 + 1000,
        uncertainty_us: 1000,
        lease_end_tick_us: T + 100_000,
    };
    d.grant(
        Grant { id: 2, tick_us: T },
        Input::DeliverSample(sample),
        |_| Ok(()),
    )
    .unwrap();
}
fn g(id: u64) -> Grant {
    Grant {
        id,
        tick_us: T + 10,
    }
}
fn write_json(path: &Path, value: &impl serde::Serialize) {
    fs::write(path, serde_json::to_vec_pretty(value).unwrap()).unwrap();
}

fn drive(
    path: &Path,
    label: &str,
    p: Profile,
    pause: u64,
    configure: impl FnOnce(&mut ControlledDriver),
    f: impl AsyncFnOnce(&mut ControlledDriver) -> Result<(), Error>,
) -> (bool, Vec<Event>) {
    let saved = Arc::new(Mutex::new(Vec::new()));
    let output = saved.clone();
    let result = ControlledDriver::run_async(cfg(path), p.clone(), async |d| {
        d.set_io_wall_pause(Duration::from_millis(pause))?;
        configure(d);
        calibrate(d, &p);
        let result = f(d).await;
        let cleanup = d.shutdown_async().await;
        *output.lock().unwrap() = d.events();
        result.and(cleanup)
    });
    let events = saved.lock().unwrap().clone();
    assert!(
        !events.is_empty(),
        "controller assertion/panic escaped before evidence capture"
    );
    let mut failed = false;
    for e in &events {
        match e {
            Event::Failure { .. } => failed = true,
            Event::Ack { .. } => assert!(!failed, "successful Ack after failure"),
            _ => {}
        }
    }
    write_json(&path.join(format!("{label}-events.json")), &events);
    write_json(
        &path.join(format!("{label}-outcome.json")),
        &json!({"runner_returned":true,"ok":result.is_ok(),"error":result.as_ref().err().map(ToString::to_string)}),
    );
    (result.is_ok(), events)
}

fn io_order(events: &[Event], success: bool) {
    let mut last_issue = 0;
    let mut ready = 0;
    let mut finished = 0;
    let mut issued = BTreeMap::new();
    let mut physical_ready = false;
    for e in events {
        match e {
            Event::IoIssued {
                identity,
                target,
                operation,
            } => {
                assert_eq!(identity.io_id, last_issue + 1);
                last_issue += 1;
                assert_eq!(identity.tick_us, T + 10);
                assert_eq!(identity.grant, 3);
                assert_eq!(identity.node, 0);
                assert!(identity.task > 0);
                assert_eq!(identity.cid, CID);
                if matches!(operation, IoOperation::Open { .. }) {
                    assert_eq!(target.open_id, Some(identity.io_id));
                }
                issued.insert(identity.io_id, (target, operation));
            }
            Event::IoReady { io_id, .. } => {
                assert_eq!(*io_id, ready + 1);
                assert_eq!(finished, ready);
                assert!(issued.contains_key(io_id));
                ready = *io_id;
                physical_ready = true;
            }
            Event::IoReleased { io_id, .. } | Event::IoDiscarded { io_id } => {
                assert_eq!(*io_id, finished + 1);
                assert_eq!(*io_id, ready);
                finished = *io_id;
                physical_ready = false;
            }
            Event::Poll { .. } | Event::Construct { .. } => assert!(
                !physical_ready,
                "protocol ran before ready I/O result release/disposal"
            ),
            Event::Ack { grant: 3 } => assert_eq!(last_issue, finished, "Ack with pending I/O"),
            _ => {}
        }
    }
    assert_eq!(last_issue, finished);
    if success {
        assert_eq!(last_issue, ready);
        assert!(!events.iter().any(|e| matches!(e, Event::Failure { .. })));
    }
}

#[test]
fn s1_s4_actual_buffers_options_ranges_and_typed_errors() {
    let path = place("primitives");
    let done = Arc::new(AtomicBool::new(false));
    let observed = done.clone();
    let (ok, events) = drive(
        &path,
        "run",
        profile(7, 1),
        2,
        |_| {},
        async move |d| {
            d.grant_async(g(3),Input::None,|d|d.spawn_session(CID,STOP,move|mut ctx|async move{
            let before=ctx.current();let v=BlobVersion::new(7);
            let(blob,size,version)=ctx.open_versioned("primitives",b"x",v..=v).await.unwrap();assert_eq!((size,version),(0,v));
            assert!(matches!(ctx.open_versioned("primitives",b"x",v..=v).await,Err(NativeError::BlobAlreadyOpen(p,n)) if p=="primitives"&&n=="78"));
            let other=BlobVersion::new(9);assert!(matches!(ctx.open_versioned("primitives",b"x",other..=other).await,Err(NativeError::BlobVersionMismatch{expected,found}) if expected== (other..=other)&&found==v));
            for partition in ["","a/b","a.b","a b","α"]{assert!(matches!(ctx.open(partition,b"x").await,Err(NativeError::PartitionNameInvalid(p)) if p==partition));}
            blob.write_at(0,[IoBuf::from(b"ab".to_vec()),IoBuf::from(b"cdef".to_vec())],WriteOptions::DONT_CACHE).await.unwrap();
            blob.write_at(2,b"XY",WriteOptions::SYNC|WriteOptions::DONT_CACHE).await.unwrap();
            let bytes=blob.read_at_buf(1,5,[IoBufMut::with_capacity(2),IoBufMut::with_capacity(3)],ReadOptions::DONT_CACHE).await.unwrap();
            let mut slices=[IoSlice::new(&[]);4];assert_eq!(bytes.chunks_vectored(&mut slices),2);assert_eq!(&*slices[0],b"bX");assert_eq!(&*slices[1],b"Yef");let _=slices;
            assert_eq!(bytes.coalesce().as_ref(),b"bXYef");blob.resize(10).await.unwrap();
            assert_eq!(blob.read_at(0,10,ReadOptions::default()).await.unwrap().coalesce().as_ref(),b"abXYef\0\0\0\0");
            let ranges=[(4,2),(0,2),(8,2)];let mut stream=Box::pin(blob.read_many(&ranges,ReadOptions::DONT_CACHE));let mut many=vec![];
            while let Some(r)=stream.next().await{let(i,data)=r.unwrap();many.push((i,data.coalesce().as_ref().to_vec()));}many.sort();assert_eq!(many,vec![(0,b"ef".to_vec()),(1,b"ab".to_vec()),(2,vec![0,0])]);drop(stream);
            let values=blob.start_read_many(&ranges,ReadOptions::default()).await.await.unwrap();assert_eq!(values.into_iter().map(|b|b.coalesce().as_ref().to_vec()).collect::<Vec<_>>(),vec![b"ef".to_vec(),b"ab".to_vec(),vec![0,0]]);
            assert!(matches!(blob.read_at(10,1,ReadOptions::default()).await,Err(NativeError::Io(e)) if e.kind()==std::io::ErrorKind::UnexpectedEof));
            assert!(matches!(blob.read_at(u64::MAX,1,ReadOptions::default()).await,Err(NativeError::OffsetOverflow)));
            assert!(matches!(blob.write_at(u64::MAX,Vec::<u8>::new(),WriteOptions::SYNC).await,Err(NativeError::OffsetOverflow)));
            assert!(matches!(blob.resize(u64::MAX).await,Err(NativeError::OffsetOverflow)));
            assert!(blob.read_at(100,0,ReadOptions::default()).await.unwrap().is_empty());
            blob.resize(3).await.unwrap();blob.resize(8).await.unwrap();blob.sync().await.unwrap();drop(blob);
            let(reopened,size,_)=ctx.open_versioned("primitives",b"x",v..=v).await.unwrap();assert_eq!(size,8);assert_eq!(reopened.read_at(0,8,ReadOptions::default()).await.unwrap().coalesce().as_ref(),b"abX\0\0\0\0\0");drop(reopened);
            assert_eq!(ctx.current(),before);ctx.try_next_u32().unwrap();ctx.remove("primitives",Some(b"x")).await.unwrap();assert!(ctx.scan("primitives").await.unwrap().is_empty());ctx.remove("primitives",None).await.unwrap();assert!(matches!(ctx.scan("primitives").await,Err(NativeError::PartitionMissing(p)) if p=="primitives"));observed.store(true,Ordering::SeqCst);
        })).await?;
            Ok(())
        },
    );
    assert!(ok && done.load(Ordering::SeqCst));
    io_order(&events, true);
    let writes: Vec<_> = events
        .iter()
        .filter_map(|e| {
            if let Event::IoIssued {
                operation:
                    IoOperation::Write {
                        offset,
                        len,
                        digest,
                        sync,
                        dont_cache,
                    },
                ..
            } = e
            {
                Some((*offset, *len, *digest, *sync, *dont_cache))
            } else {
                None
            }
        })
        .collect();
    assert_eq!(
        writes,
        vec![
            (0, 6, Sha256::digest(b"abcdef").into(), false, true),
            (2, 2, Sha256::digest(b"XY").into(), true, true),
            // Issuance records failed calls too; this is not a successful barrier.
            (u64::MAX, 0, Sha256::digest(b"").into(), true, false)
        ]
    );
}

#[test]
fn s3_pending_open_observer_abandonment_does_not_hold_native_exclusivity() {
    let path = place("abandoned_pending_open");
    let done = Arc::new(AtomicBool::new(false));
    let finished = done.clone();
    let (ok, events) = drive(
        &path,
        "run",
        profile(7, 1),
        2,
        |_| {},
        async move |d| {
            d.grant_async(g(3), Input::None, |d| {
                d.spawn_session(CID, STOP, move |ctx| async move {
                    let mut abandoned = Box::pin(ctx.open("pending", b"x"));
                    poll_fn(|cx| {
                        assert!(abandoned.as_mut().poll(cx).is_pending());
                        Poll::Ready(())
                    })
                    .await;
                    drop(abandoned);
                    let (blob, size) = ctx.open("pending", b"x").await.unwrap();
                    assert_eq!(size, 0);
                    drop(blob);
                    finished.store(true, Ordering::SeqCst);
                })
            })
            .await?;
            Ok(())
        },
    );
    assert!(ok && done.load(Ordering::SeqCst));
    io_order(&events, true);
    assert!(events.iter().any(|e| matches!(
        e,
        Event::IoReleased {
            io_id: 1,
            delivered: false
        }
    )));
}

struct AbortOnReady {
    handle: Arc<Mutex<Option<Handle<()>>>>,
    delegate: Waker,
}
impl Wake for AbortOnReady {
    fn wake(self: Arc<Self>) {
        self.wake_by_ref();
    }
    fn wake_by_ref(self: &Arc<Self>) {
        self.handle.lock().unwrap().as_ref().unwrap().abort();
        self.delegate.wake_by_ref();
    }
}
struct NotifyOnDrop(Option<tokio::sync::oneshot::Sender<()>>);
impl Drop for NotifyOnDrop {
    fn drop(&mut self) {
        if let Some(tx) = self.0.take() {
            let _ = tx.send(());
        }
    }
}

#[test]
fn s3_ready_open_observer_abort_drops_channel_resource_before_next_open() {
    let path = place("abandoned_ready_open");
    let consumer = Arc::new(AtomicBool::new(false));
    let consumed = consumer.clone();
    let done = Arc::new(AtomicBool::new(false));
    let finished = done.clone();
    let (ok, events) = drive(
        &path,
        "run",
        profile(7, 1),
        0,
        |_| {},
        async move |d| {
            d.grant_async(g(3), Input::None, |d| {
                d.spawn_session(CID, STOP, move |ctx| async move {
                    let slot = Arc::new(Mutex::new(None));
                    let child_slot = slot.clone();
                    let (tx, rx) = tokio::sync::oneshot::channel();
                    let h = ctx.child("observer").spawn(move |ctx| async move {
                        let notify = NotifyOnDrop(Some(tx));
                        let mut open = Box::pin(ctx.open("ready", b"x"));
                        let result = poll_fn(|cx| {
                            let w = Waker::from(Arc::new(AbortOnReady {
                                handle: child_slot.clone(),
                                delegate: cx.waker().clone(),
                            }));
                            open.as_mut().poll(&mut Context::from_waker(&w))
                        })
                        .await;
                        consumed.store(true, Ordering::SeqCst);
                        drop(result);
                        drop(notify);
                    });
                    *slot.lock().unwrap() = Some(h);
                    rx.await.unwrap();
                    drop(slot.lock().unwrap().take());
                    let (blob, size) = ctx.open("ready", b"x").await.unwrap();
                    assert_eq!(size, 0);
                    drop(blob);
                    finished.store(true, Ordering::SeqCst);
                })
            })
            .await?;
            Ok(())
        },
    );
    assert!(ok && done.load(Ordering::SeqCst));
    assert!(!consumer.load(Ordering::SeqCst));
    io_order(&events, true);
    assert!(events.iter().any(|e| matches!(
        e,
        Event::IoReleased {
            io_id: 1,
            delivered: true
        }
    )));
}

#[test]
fn s5_remove_name_and_partition_keep_old_instances_readable() {
    let path = place("removed_instances");
    let (ok, events) = drive(
        &path,
        "run",
        profile(7, 1),
        0,
        |_| {},
        async |d| {
            d.grant_async(g(3), Input::None, |d| {
                d.spawn_session(CID, STOP, |ctx| async move {
                    let (old, _) = ctx.open("versions", b"x").await.unwrap();
                    old.write_at(0, b"old", WriteOptions::default())
                        .await
                        .unwrap();
                    ctx.remove("versions", Some(b"x")).await.unwrap();
                    let (new, size) = ctx.open("versions", b"x").await.unwrap();
                    assert_eq!(size, 0);
                    new.write_at(0, b"new", WriteOptions::SYNC).await.unwrap();
                    assert_eq!(
                        old.read_at(0, 3, ReadOptions::default())
                            .await
                            .unwrap()
                            .coalesce()
                            .as_ref(),
                        b"old"
                    );
                    assert_eq!(
                        new.read_at(0, 3, ReadOptions::default())
                            .await
                            .unwrap()
                            .coalesce()
                            .as_ref(),
                        b"new"
                    );
                    ctx.remove("versions", None).await.unwrap();
                    assert!(matches!(
                        ctx.scan("versions").await,
                        Err(NativeError::PartitionMissing(_))
                    ));
                    let (third, size) = ctx.open("versions", b"x").await.unwrap();
                    assert_eq!(size, 0);
                    assert_eq!(
                        old.read_at(0, 3, ReadOptions::DONT_CACHE)
                            .await
                            .unwrap()
                            .coalesce()
                            .as_ref(),
                        b"old"
                    );
                    assert_eq!(
                        new.read_at(0, 3, ReadOptions::DONT_CACHE)
                            .await
                            .unwrap()
                            .coalesce()
                            .as_ref(),
                        b"new"
                    );
                    drop((old, new, third));
                })
            })
            .await?;
            Ok(())
        },
    );
    assert!(ok);
    io_order(&events, true);
    let opens: Vec<_> = events
        .iter()
        .filter_map(|e| {
            if let Event::IoIssued {
                target,
                operation: IoOperation::Open { .. },
                ..
            } = e
            {
                target.open_id
            } else {
                None
            }
        })
        .collect();
    assert_eq!(opens.len(), 3);
    assert!(opens.windows(2).all(|x| x[0] < x[1]));
}

#[test]
fn s6_dropped_and_aborted_sync_observers_preserve_flush_and_release_ownership() {
    for abort in [false, true] {
        let path = place("detached_sync");
        let accepted = Arc::new(AtomicBool::new(false));
        let yes = accepted.clone();
        let (ok, events) = drive(
            &path,
            "run",
            profile(7, 1),
            2,
            |_| {},
            async move |d| {
                d.grant_async(g(3), Input::None, |d| {
                    d.spawn_session(CID, STOP, move |ctx| async move {
                        let (blob, _) = ctx.open("sync_observer", b"x").await.unwrap();
                        blob.write_at(0, b"accepted", WriteOptions::default())
                            .await
                            .unwrap();
                        let observer = blob.start_sync().await;
                        yes.store(true, Ordering::SeqCst);
                        if abort {
                            observer.abort();
                        }
                        let retained = if abort {
                            Some(observer)
                        } else {
                            drop(observer);
                            None
                        };
                        drop(blob);
                        let (reopened, size) = ctx.open("sync_observer", b"x").await.unwrap();
                        assert_eq!(size, 8);
                        assert_eq!(
                            reopened
                                .read_at(0, 8, ReadOptions::default())
                                .await
                                .unwrap()
                                .coalesce()
                                .as_ref(),
                            b"accepted"
                        );
                        if let Some(observer) = retained {
                            assert!(matches!(observer.await, Err(NativeError::Aborted)));
                        }
                        drop(reopened);
                    })
                })
                .await?;
                Ok(())
            },
        );
        assert!(ok && accepted.load(Ordering::SeqCst));
        io_order(&events, true);
        let sync_issue=events.iter().position(|e|matches!(e,Event::IoIssued{identity,operation:IoOperation::Sync{detached:true},..} if identity.io_id==3)).unwrap();
        let next_open = events
            .iter()
            .position(|e| matches!(e,Event::IoIssued{identity,..} if identity.io_id==4))
            .unwrap();
        let sync_ready = events
            .iter()
            .position(|e| matches!(e, Event::IoReady { io_id: 3, .. }))
            .unwrap();
        let next_ready = events
            .iter()
            .position(|e| matches!(e, Event::IoReady { io_id: 4, .. }))
            .unwrap();
        assert!(sync_issue < next_open && next_open < sync_ready && sync_ready < next_ready);
    }
}

fn raw_probe(path: &Path, partition: &str, name: &[u8]) {
    Runner::new(cfg(path)).start(|ctx| async move {
        let (blob, _) = ctx.open(partition, name).await.unwrap();
        drop(blob);
    });
}

#[test]
fn s8_cancel_borrowed_grant_with_owned_active_io_then_repeat_failed_calls() {
    let path = place("cancelled_grant");
    let gate = Arc::new(Mutex::new(None));
    let configured = gate.clone();
    let used = Arc::new(AtomicBool::new(false));
    let observed = used.clone();
    let (ok, events) = drive(
        &path,
        "run",
        profile(7, 1),
        0,
        move |d| {
            *configured.lock().unwrap() = Some(d.hold_io_before_dispatch(1).unwrap());
        },
        async move |d| {
            let mut grant = Box::pin(d.grant_async(g(3), Input::None, |d| {
                d.spawn_session(CID, STOP, move |ctx| async move {
                    let (blob, _) = ctx.open("cancelled", b"x").await.unwrap();
                    observed.store(true, Ordering::SeqCst);
                    drop(blob);
                })
            }));
            poll_fn(|cx| {
                assert!(grant.as_mut().poll(cx).is_pending());
                Poll::Ready(())
            })
            .await;
            drop(grant);
            assert_eq!(d.active_io_id(), Some(1));
            for id in [4, 5] {
                assert!(
                    d.grant(g(id), Input::None, |_| panic!("callback after abandonment"))
                        .is_err()
                );
                assert_eq!(d.active_io_id(), Some(1));
            }
            assert!(d.fail_transport().is_err());
            assert!(d.shutdown().is_err());
            gate.lock().unwrap().take().unwrap().send(()).unwrap();
            assert!(d.shutdown_async().await.is_err());
            assert_eq!(d.active_io_id(), None);
            Ok(())
        },
    );
    assert!(!ok && !used.load(Ordering::SeqCst));
    io_order(&events, false);
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, Event::Ack { grant: 3 | 4 | 5 }))
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::IoDiscarded { io_id: 1 }))
    );
    raw_probe(&path, "cancelled", b"x");
}

#[test]
fn s8_ready_result_is_retained_until_root_cleanup_and_reopen_is_unblocked() {
    let path = place("ready_failure");
    let used = Arc::new(AtomicBool::new(false));
    let observed = used.clone();
    let (ok, events) = drive(
        &path,
        "run",
        profile(7, 1),
        0,
        |d| d.fail_before_io_release(1).unwrap(),
        async move |d| {
            assert!(
                d.grant_async(g(3), Input::None, |d| d.spawn_session(
                    CID,
                    STOP,
                    move |ctx| async move {
                        let (blob, _) = ctx.open("ready_failure", b"x").await.unwrap();
                        observed.store(true, Ordering::SeqCst);
                        drop(blob);
                    }
                ))
                .await
                .is_err()
            );
            let before = d.events();
            assert!(before.iter().any(|e| matches!(
                e,
                Event::IoReady {
                    io_id: 1,
                    result: IoResult::Open { .. }
                }
            )));
            assert!(!before.iter().any(|e| matches!(
                e,
                Event::IoReleased { io_id: 1, .. } | Event::IoDiscarded { io_id: 1 }
            )));
            assert!(d.shutdown().is_err());
            assert!(d.shutdown_async().await.is_err());
            Ok(())
        },
    );
    assert!(!ok && !used.load(Ordering::SeqCst));
    io_order(&events, false);
    assert!(!events.iter().any(|e| matches!(e, Event::Ack { grant: 3 })));
    raw_probe(&path, "ready_failure", b"x");
}

#[test]
fn s8_synchronous_misuse_cannot_ack_or_drop_accepted_io() {
    let path = place("sync_misuse");
    let used = Arc::new(AtomicBool::new(false));
    let observed = used.clone();
    let (ok, events) = drive(
        &path,
        "run",
        profile(7, 1),
        0,
        |_| {},
        async move |d| {
            assert!(
                d.grant(g(3), Input::None, |d| d.spawn_session(
                    CID,
                    STOP,
                    move |ctx| async move {
                        let (blob, _) = ctx.open("sync_misuse", b"x").await.unwrap();
                        observed.store(true, Ordering::SeqCst);
                        drop(blob);
                    }
                ))
                .is_err()
            );
            assert!(d.shutdown_async().await.is_err());
            Ok(())
        },
    );
    assert!(!ok && !used.load(Ordering::SeqCst));
    io_order(&events, false);
    assert!(!events.iter().any(|e| matches!(e, Event::Ack { grant: 3 })));
    raw_probe(&path, "sync_misuse", b"x");
}

struct ReleaseProbe {
    context: SimContext,
    delegate: Mutex<Option<Waker>>,
    observations: Arc<Mutex<Vec<bool>>>,
    attack: bool,
}
impl Wake for ReleaseProbe {
    fn wake(self: Arc<Self>) {
        self.wake_by_ref();
    }
    fn wake_by_ref(self: &Arc<Self>) {
        self.observations
            .lock()
            .unwrap()
            .push(self.context.publication_allowed());
        if self.attack {
            let _ = self.context.current();
        }
        self.delegate
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .wake_by_ref();
    }
}

#[test]
fn s2_io_release_is_not_ordinary_authority() {
    for attack in [false, true] {
        let path = place("release_authority");
        let values = Arc::new(Mutex::new(vec![]));
        let reported = values.clone();
        let after = Arc::new(AtomicBool::new(false));
        let used = after.clone();
        let (ok, events) = drive(
            &path,
            "run",
            profile(7, 1),
            2,
            |_| {},
            async move |d| {
                let result = d
                    .grant_async(g(3), Input::None, |d| {
                        d.spawn_session(CID, STOP, move |ctx| async move {
                            let probe = Arc::new(ReleaseProbe {
                                context: ctx.child("release_probe"),
                                delegate: Mutex::new(None),
                                observations: reported,
                                attack,
                            });
                            let mut future = Box::pin(ctx.open("authority", b"x"));
                            let opened = poll_fn(|cx| {
                                *probe.delegate.lock().unwrap() = Some(cx.waker().clone());
                                let w = Waker::from(probe.clone());
                                future.as_mut().poll(&mut Context::from_waker(&w))
                            })
                            .await
                            .unwrap();
                            assert!(ctx.publication_allowed());
                            used.store(true, Ordering::SeqCst);
                            drop(opened);
                        })
                    })
                    .await;
                if attack {
                    assert!(result.is_err());
                } else {
                    result?;
                }
                Ok(())
            },
        );
        assert_eq!(ok, !attack);
        let observations = values.lock().unwrap();
        assert!(!observations.is_empty() && observations.iter().all(|v| !*v));
        assert_eq!(after.load(Ordering::SeqCst), !attack);
        if !attack {
            io_order(&events, true);
        } else {
            assert!(!events.iter().any(|e| matches!(e, Event::Ack { grant: 3 })));
        }
        raw_probe(&path, "authority", b"x");
    }
}

fn prepare_names(path: &Path, order: &[usize], names: &[Vec<u8>]) {
    Runner::new(cfg(path)).start(|ctx| async move {
        for i in order {
            let (blob, _) = ctx.open("scan", &names[*i]).await.unwrap();
            blob.write_at(0, names[*i].clone(), WriteOptions::SYNC)
                .await
                .unwrap();
            drop(blob);
        }
    });
}

#[test]
fn s7_binary_scan_sorting_does_not_depend_on_directory_creation_order() {
    let names = vec![vec![0], vec![0, 1], vec![0x7f], vec![0x80], vec![0xff, 0]];
    let mut traces = vec![];
    for order in [vec![0, 1, 2, 3, 4], vec![4, 2, 0, 3, 1]] {
        let path = place("scan_order");
        prepare_names(&path, &order, &names);
        let expected = names.clone();
        let (ok, events) = drive(
            &path,
            "protocol",
            profile(7, 1),
            0,
            |_| {},
            async move |d| {
                d.grant_async(g(3), Input::None, |d| {
                    d.spawn_session(CID, STOP, move |ctx| async move {
                        let found = ctx.scan("scan").await.unwrap();
                        assert_eq!(found, expected);
                        for name in found {
                            let (blob, len) = ctx.open("scan", &name).await.unwrap();
                            assert_eq!(len, name.len() as u64);
                            assert_eq!(
                                blob.read_at(0, name.len(), ReadOptions::default())
                                    .await
                                    .unwrap()
                                    .coalesce()
                                    .as_ref(),
                                name
                            );
                            drop(blob);
                        }
                    })
                })
                .await?;
                Ok(())
            },
        );
        assert!(ok);
        io_order(&events, true);
        traces.push(serde_json::to_vec(&events).unwrap());
        write_json(&path.join("setup-order.json"), &order);
    }
    assert_eq!(traces[0], traces[1]);
}

type S = Scheme<PublicKey, MinSig>;
type A = Artifact<S, PayloadDigest>;
type J = Journal<SimContext, A>;
fn proposal(view: u64, parent: u64, value: u8) -> Proposal<PayloadDigest> {
    Proposal::new(
        Round::new(Epoch::new(1), View::new(view)),
        View::new(parent),
        PayloadDigest([value; 32]),
    )
}
fn fixture() -> (S, Vec<(u64, A)>, Value) {
    let mut rng = ChaCha20Rng::from_seed([42; 32]);
    let mut keys = vec![];
    let mut map = vec![];
    for i in 0..5u8 {
        let identity = PrivateKey::decode(Copying(&[16 + i; 32]))
            .unwrap()
            .public_key();
        let (secret, public) = ops::keypair::<_, MinSig>(&mut rng);
        let pop = ops::sign_proof_of_possession::<MinSig>(&secret, b"I1b/storage/pop/v1");
        ops::verify_proof_of_possession::<MinSig>(&public, b"I1b/storage/pop/v1", &pop).unwrap();
        map.push((identity.clone(), public));
        keys.push((i, identity, secret));
    }
    let map = BiMap::try_from(map).unwrap();
    let verifier = S::verifier(b"I1b/storage/vote/v1", map.clone());
    let mut rows = vec![];
    let mut mapping = vec![];
    let mut vrng = ChaCha20Rng::from_seed([91; 32]);
    for (member, identity, secret) in keys {
        let signer = S::signer(b"I1b/storage/vote/v1", map.clone(), secret).unwrap();
        let index = signer.me().unwrap().signer().unwrap();
        assert_eq!(signer.participants().get(index.into()), Some(&identity));
        mapping.push(json!({"member":member,"signer_index":format!("{index:?}"),"identity_bytes":identity.encode().to_vec()}));
        let n = Notarize::sign(&signer, proposal(1, 0, 0x44)).unwrap();
        assert!(n.verify(&mut vrng, &verifier, &Sequential));
        rows.push((1, A::Notarize(n)));
        if member == 0 {
            let f = Finalize::sign(&signer, proposal(1, 0, 0x44)).unwrap();
            assert!(f.verify(&mut vrng, &verifier, &Sequential));
            rows.push((1, A::Finalize(f)));
            let n = Notarize::sign(&signer, proposal(2, 1, 0x55)).unwrap();
            assert!(n.verify(&mut vrng, &verifier, &Sequential));
            rows.push((2, A::Notarize(n)));
        }
        if member == 1 {
            let n =
                Nullify::sign::<PayloadDigest>(&signer, Round::new(Epoch::new(1), View::new(1)))
                    .unwrap();
            assert!(n.verify::<_, PayloadDigest>(&mut vrng, &verifier, &Sequential));
            rows.push((1, A::Nullify(n)));
        }
    }
    rows.push((
        1,
        A::Certification(Round::new(Epoch::new(1), View::new(1)), true),
    ));
    (verifier, rows, json!(mapping))
}
fn journal_cfg(s: &S) -> JournalConfig<<A as commonware_codec::Read>::Cfg> {
    JournalConfig {
        partition: "i1b-voter-0".to_owned(),
        compression: None,
        codec_config: s.certificate_codec_config(),
        page_cache: CacheRef::new(paged::page_size(256), NZUsize!(4)),
        write_buffer: NZUsize!(488),
    }
}
fn varint(mut n: u32) -> Vec<u8> {
    let mut out = vec![];
    while n >= 128 {
        out.push((n as u8 & 127) | 128);
        n >>= 7;
    }
    out.push(n as u8);
    out
}
fn crc32c(bytes: &[u8]) -> u32 {
    let mut c = !0u32;
    for x in bytes {
        c ^= *x as u32;
        for _ in 0..8 {
            c = if c & 1 != 0 {
                (c >> 1) ^ 0x82f63b78
            } else {
                c >> 1
            };
        }
    }
    !c
}
fn page_payload(raw: &[u8]) -> Vec<u8> {
    assert!(raw.len() >= 4096);
    assert_eq!((raw.len() - 4096) % 256, 0);
    let mut logical = vec![];
    for page in raw[4096..].chunks_exact(256) {
        let f = &page[244..];
        let slots = [
            (
                u16::from_be_bytes(f[..2].try_into().unwrap()) as usize,
                u32::from_be_bytes(f[2..6].try_into().unwrap()),
            ),
            (
                u16::from_be_bytes(f[6..8].try_into().unwrap()) as usize,
                u32::from_be_bytes(f[8..12].try_into().unwrap()),
            ),
        ];
        let first = usize::from(slots[1].0 > slots[0].0);
        let mut selected = None;
        for i in [first, first ^ 1] {
            let (len, crc) = slots[i];
            if len > 0 && len <= 244 && crc32c(&page[..len]) == crc {
                selected = Some(len);
                break;
            }
        }
        let len = selected.expect("invalid native page CRC");
        logical.extend_from_slice(&page[..len]);
    }
    logical
}

fn journal_case(seed: u64, pause: u64) -> Value {
    let path = place("real_artifact_journal");
    let (verifier, entries, mapping) = fixture();
    let mut logical = BTreeMap::<u64, Vec<u8>>::new();
    let mut expected = vec![];
    for (section, value) in &entries {
        let bytes = value.encode().to_vec();
        assert!(bytes.len() <= 4096);
        let body = logical.entry(*section).or_default();
        let offset = body.len() as u64;
        body.extend(varint(bytes.len() as u32));
        body.extend(&bytes);
        expected.push((*section, offset, bytes.len() as u32, bytes));
    }
    expected.sort_by_key(|r| (r.0, r.1));
    assert!(logical[&1].len() > 488 && logical[&1].len() % 244 != 0);
    let expected_append = expected.clone();
    let vc = verifier.clone();
    let (ok, written) = drive(
        &path,
        "written",
        profile(seed, 1),
        pause,
        |_| {},
        async move |d| {
            d.grant_async(g(3), Input::None, |d| {
                d.spawn_session(CID, STOP, move |ctx| async move {
                    let now = ctx.current();
                    let mut journal = J::init(ctx.child("journal"), journal_cfg(&vc))
                        .await
                        .unwrap();
                    let mut offset = BTreeMap::<u64, u64>::new();
                    for (section, value) in entries {
                        let encoded = value.encode();
                        let expected_offset = *offset.entry(section).or_default();
                        let (next, at, len) = journal.append(section, &value).await.unwrap();
                        assert_eq!(at, expected_offset);
                        assert_eq!(len as usize, encoded.len());
                        *offset.get_mut(&section).unwrap() +=
                            encoded.len() as u64 + varint(len).len() as u64;
                        journal = next.sync(section).await.unwrap();
                    }
                    drop(journal);
                    assert_eq!(ctx.current(), now);
                })
            })
            .await?;
            Ok(())
        },
    );
    assert!(ok);
    io_order(&written, true);
    let mut before = vec![];
    for section in [1u64, 2] {
        let bytes = fs::read(
            path.join("store/i1b-voter-0")
                .join(format!("{section:016x}")),
        )
        .unwrap();
        assert_eq!(page_payload(&bytes), logical[&section]);
        assert!(bytes.len() <= 69632);
        fs::write(
            path.join(format!("section-{section}-before-replay.bin")),
            &bytes,
        )
        .unwrap();
        before.push((section, bytes));
    }
    let replayed = Arc::new(Mutex::new(Vec::new()));
    let recovered = replayed.clone();
    let vc = verifier.clone();
    let extra = A::Certification(Round::new(Epoch::new(1), View::new(2)), true);
    let extra_bytes = extra.encode().to_vec();
    let extra_offset = logical[&2].len() as u64;
    let (ok, read) = drive(
        &path,
        "replayed",
        profile(seed, 2),
        pause,
        |_| {},
        async move |d| {
            d.grant_async(g(3), Input::None, |d| {
                d.spawn_session(CID, STOP, move |ctx| async move {
                    let journal = J::init(ctx.child("journal"), journal_cfg(&vc))
                        .await
                        .unwrap();
                    let mut replay = journal
                        .replay(0, 0, NZUsize!(256), ReadOptions::DONT_CACHE)
                        .await
                        .unwrap();
                    let mut got = vec![];
                    while let Some(row) = replay.next().await {
                        let (section, offset, len, artifact) = row.unwrap();
                        got.push((section, offset, len, artifact.encode().to_vec()));
                    }
                    assert_eq!(got, expected_append);
                    *recovered.lock().unwrap() = got;
                    let journal = replay.finish().unwrap();
                    let (journal, offset, len) = journal.append(2, &extra).await.unwrap();
                    assert_eq!(offset, extra_offset);
                    assert_eq!(len as usize, extra.encode().len());
                    drop(journal.sync(2).await.unwrap());
                })
            })
            .await?;
            Ok(())
        },
    );
    assert!(ok);
    io_order(&read, true);
    assert_eq!(*replayed.lock().unwrap(), expected);
    logical
        .get_mut(&2)
        .unwrap()
        .extend(varint(extra_bytes.len() as u32));
    logical.get_mut(&2).unwrap().extend(extra_bytes);
    let mut final_digests = vec![];
    for section in [1u64, 2] {
        let bytes = fs::read(
            path.join("store/i1b-voter-0")
                .join(format!("{section:016x}")),
        )
        .unwrap();
        assert_eq!(page_payload(&bytes), logical[&section]);
        fs::write(
            path.join(format!("section-{section}-after-replay.bin")),
            &bytes,
        )
        .unwrap();
        final_digests.push((section, Sha256::digest(&bytes).to_vec()));
    }
    let writes: Vec<_> = written
        .iter()
        .filter_map(|e| {
            if let Event::IoIssued {
                target,
                operation:
                    IoOperation::Write {
                        offset, len, sync, ..
                    },
                ..
            } = e
            {
                Some((target.name.clone(), *offset, *len, *sync))
            } else {
                None
            }
        })
        .collect();
    assert!(
        writes
            .iter()
            .filter(
                |(name, offset, len, _)| name.as_deref() == Some(&1u64.to_be_bytes())
                    && *offset == 0
                    && *len == 256
            )
            .count()
            >= 2
    );
    assert!(writes.iter().any(
        |(name, offset, _, _)| name.as_deref() == Some(&1u64.to_be_bytes()) && *offset >= 256
    ));
    // Pinned Writer::sync may use either a SYNC write or a full Blob barrier,
    // depending on earlier mutation debt. Exercise and distinguish both paths.
    assert!(writes.iter().any(|(_, _, _, sync)| *sync));
    assert!(written.iter().any(|e| matches!(
        e,
        Event::IoIssued {
            operation: IoOperation::Sync { detached: false },
            ..
        }
    )));
    let record = json!({"seed":seed,"config":{"partition":"i1b-voter-0","physical_page_size":256,"logical_page_size":244,"cache_pages":4,"write_buffer":488,"replay_buffer":256,"compression":null,"epoch":1},"mapping":mapping,"expected_artifacts":expected,"replayed_artifacts":*replayed.lock().unwrap(),"written_events":written,"replayed_events":read,"final_file_digests":final_digests});
    write_json(&path.join("canonical.json"), &record);
    record
}

#[test]
fn s9_s10_real_artifacts_partial_pages_rewrite_replay_and_all_seed_wall_pairs() {
    assert_eq!(crc32c(b"123456789"), 0xe3069283);
    assert_eq!(crc32c(&[0; 32]), 0x8a9136aa);
    assert_eq!(paged::page_size(256).get(), 244);
    for seed in [7, 42, 99] {
        assert_eq!(
            journal_case(seed, 0),
            journal_case(seed, 2),
            "seed{seed} complete trace differs under physical wall pauses"
        );
    }
}
