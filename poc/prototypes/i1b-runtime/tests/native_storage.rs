mod support;

use commonware_runtime::{
    Blob, BlobVersion, Error as NativeError, IoBufMut, ReadOptions, Spawner, Storage, Supervisor,
    WriteOptions,
};
use futures::StreamExt as _;
use poc_native_runtime::{ControlledDriver, Event, Grant, Input, IoOperation, IoResult, Step};
use std::{
    future::{Future, pending, poll_fn},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    task::Poll,
};
use support::*;

fn step(id: u64) -> Grant {
    Grant {
        id,
        tick_us: START + id,
    }
}

fn check_order(step: &Step) {
    let issued: Vec<_> = step
        .events
        .iter()
        .filter_map(|e| match e {
            Event::IoIssued { identity, .. } => {
                assert_eq!(identity.grant, step.grant.id);
                assert_eq!(identity.tick_us, step.grant.tick_us);
                Some(identity.io_id)
            }
            _ => None,
        })
        .collect();
    let ready: Vec<_> = step
        .events
        .iter()
        .filter_map(|e| match e {
            Event::IoReady { io_id, .. } => Some(*io_id),
            _ => None,
        })
        .collect();
    let released: Vec<_> = step
        .events
        .iter()
        .filter_map(|e| match e {
            Event::IoReleased { io_id, .. } => Some(*io_id),
            _ => None,
        })
        .collect();
    assert_eq!(issued, ready);
    assert_eq!(issued, released);
    assert!(issued.windows(2).all(|pair| pair[0] < pair[1]));
    assert!(matches!(step.events.last(), Some(Event::Ack { .. })));
    for (ordinal, id) in issued.into_iter().enumerate() {
        let issue = step
            .events
            .iter()
            .position(|e| matches!(e, Event::IoIssued { identity, .. } if identity.io_id == id))
            .unwrap();
        let completion = step
            .events
            .iter()
            .position(|e| matches!(e, Event::IoReady { io_id, .. } if *io_id == id))
            .unwrap();
        let release = step
            .events
            .iter()
            .position(|e| matches!(e, Event::IoReleased { io_id, .. } if *io_id == id))
            .unwrap();
        assert!(issue < completion && completion < release);
        if ordinal > 0 {
            let prior_release = step
                .events
                .iter()
                .position(|e| matches!(e, Event::IoReleased { io_id, .. } if *io_id == id - 1));
            if let Some(prior_release) = prior_release {
                assert!(prior_release < completion);
            }
        }
    }
}

#[test]
fn native_storage_roundtrip_options_and_expected_errors() {
    let dir = TestDir::new();
    let profile = profile();
    let ran = Arc::new(AtomicBool::new(false));
    let complete = ran.clone();
    ControlledDriver::run_async(dir.config(), profile.clone(), async move |driver| {
        calibrate(driver, &profile, START + 500);
        let step = driver
            .grant_async(step(3), Input::None, |driver| {
                driver.spawn_session(CID, STOP, move |context| async move {
                    let version = BlobVersion::new(3);
                    let (blob, len, actual_version) = context
                        .open_versioned("storage", b"item", version..=version)
                        .await
                        .unwrap();
                    assert_eq!((len, actual_version), (0, version));
                    assert!(matches!(
                        context
                            .open_versioned("storage", b"item", version..=version)
                            .await,
                        Err(NativeError::BlobAlreadyOpen(..))
                    ));
                    let other = BlobVersion::new(7);
                    assert!(matches!(
                        context
                            .open_versioned("storage", b"item", other..=other)
                            .await,
                        Err(NativeError::BlobVersionMismatch { .. })
                    ));
                    assert!(matches!(
                        context.open("invalid/path", b"item").await,
                        Err(NativeError::PartitionNameInvalid(_))
                    ));
                    blob.write_at(0, vec![1, 2, 3, 4], WriteOptions::DONT_CACHE)
                        .await
                        .unwrap();
                    blob.write_at(4, vec![5, 6], WriteOptions::DONT_CACHE | WriteOptions::SYNC)
                        .await
                        .unwrap();
                    blob.resize(8).await.unwrap();
                    blob.sync().await.unwrap();
                    let read = blob
                        .read_at_buf(
                            1,
                            4,
                            [IoBufMut::with_capacity(2), IoBufMut::with_capacity(2)],
                            ReadOptions::DONT_CACHE,
                        )
                        .await
                        .unwrap();
                    assert!(!read.is_single());
                    assert_eq!(read.coalesce().as_ref(), &[2, 3, 4, 5]);
                    assert!(blob.read_at(7, 3, ReadOptions::default()).await.is_err());
                    let parts = blob
                        .start_read_many(&[(0, 2), (5, 3)], ReadOptions::DONT_CACHE)
                        .await
                        .await
                        .unwrap();
                    assert_eq!(parts[0].as_single().unwrap().as_ref(), &[1, 2]);
                    assert_eq!(parts[1].as_single().unwrap().as_ref(), &[6, 0, 0]);
                    let ranges = [(0, 1), (1, 1)];
                    let stream = blob.read_many(&ranges, ReadOptions::default());
                    let mut reads = Box::pin(stream);
                    let mut observed = vec![];
                    while let Some(read) = reads.next().await {
                        let (index, read) = read.unwrap();
                        observed.push((index, read.coalesce().as_ref()[0]));
                    }
                    observed.sort_unstable();
                    assert_eq!(observed, [(0, 1), (1, 2)]);
                    drop(reads);
                    drop(blob);
                    let (reopened, len, _) = context
                        .open_versioned("storage", b"item", version..=version)
                        .await
                        .unwrap();
                    assert_eq!(len, 8);
                    drop(reopened);
                    assert_eq!(context.scan("storage").await.unwrap(), [b"item".to_vec()]);
                    context.remove("storage", Some(b"item")).await.unwrap();
                    assert!(context.scan("storage").await.unwrap().is_empty());
                    context.remove("storage", None).await.unwrap();
                    assert!(matches!(
                        context.scan("storage").await,
                        Err(NativeError::PartitionMissing(_))
                    ));
                    complete.store(true, Ordering::SeqCst);
                })
            })
            .await?;
        check_order(&step);
        assert!(step.events.iter().any(|e| matches!(
            e,
            Event::IoIssued {
                operation: IoOperation::Write {
                    offset: 4,
                    len: 2,
                    sync: true,
                    dont_cache: true,
                    ..
                },
                ..
            }
        )));
        assert!(ran.load(Ordering::SeqCst));
        Ok(())
    })
    .unwrap();
}

#[test]
fn abandoned_opens_release_exclusive_native_ownership_before_following_open() {
    let dir = TestDir::new();
    let profile = profile();
    ControlledDriver::run_async(dir.config(), profile.clone(), async move |driver| {
        calibrate(driver, &profile, START + 500);
        let result = driver
            .grant_async(step(3), Input::None, |driver| {
                driver.spawn_session(CID, STOP, |context| async move {
                    let mut abandoned = Box::pin(context.open("abandoned", b"item"));
                    poll_fn(|cx| {
                        assert!(abandoned.as_mut().poll(cx).is_pending());
                        Poll::Ready(())
                    })
                    .await;
                    drop(abandoned);
                    let (blob, _) = context.open("abandoned", b"item").await.unwrap();
                    drop(blob);

                    let (signal, accepted) = tokio::sync::oneshot::channel();
                    let child = context.child("open").spawn(|child| async move {
                        let mut open = Box::pin(child.open("abandoned", b"item"));
                        poll_fn(|cx| {
                            assert!(open.as_mut().poll(cx).is_pending());
                            Poll::Ready(())
                        })
                        .await;
                        signal.send(()).unwrap();
                        open.await.unwrap().0
                    });
                    accepted.await.unwrap();
                    drop(child);
                    let (blob, _) = context.open("abandoned", b"item").await.unwrap();
                    drop(blob);
                })
            })
            .await?;
        check_order(&result);
        assert!(result.events.iter().any(|e| matches!(
            e,
            Event::IoReleased {
                io_id: 1,
                delivered: false
            }
        )));
        assert!(result.events.iter().any(|e| matches!(
            e,
            Event::IoReady {
                io_id: 1,
                result: IoResult::Open { .. }
            }
        )));
        Ok(())
    })
    .unwrap();
}

#[test]
fn close_and_recreate_keep_old_open_readable_across_same_driver_cids() {
    let dir = TestDir::new();
    let profile = profile();
    let second = [0x34; 32];
    ControlledDriver::run_async(dir.config(), profile.clone(), async move |driver| {
        calibrate(driver, &profile, START + 500);
        let (tx, rx) = tokio::sync::oneshot::channel();
        let first = driver
            .grant_async(step(3), Input::None, |driver| {
                driver.spawn_session(CID, STOP, |context| async move {
                    let (old, _) = context.open("shared", b"item").await.unwrap();
                    old.write_at(0, b"past".to_vec(), WriteOptions::SYNC)
                        .await
                        .unwrap();
                    assert!(tx.send(old).is_ok());
                })
            })
            .await?;
        check_order(&first);
        apply(driver, 4, START + 4, Input::Close(CID));
        let current = driver
            .grant_async(step(5), Input::None, |driver| {
                driver.spawn_session(second, STOP, |context| async move {
                    let old = rx.await.unwrap();
                    context.remove("shared", Some(b"item")).await.unwrap();
                    let (new, _) = context.open("shared", b"item").await.unwrap();
                    new.write_at(0, b"next".to_vec(), WriteOptions::SYNC)
                        .await
                        .unwrap();
                    assert_eq!(
                        old.read_at(0, 4, ReadOptions::default())
                            .await
                            .unwrap()
                            .coalesce()
                            .as_ref(),
                        b"past"
                    );
                    assert_eq!(
                        new.read_at(0, 4, ReadOptions::default())
                            .await
                            .unwrap()
                            .coalesce()
                            .as_ref(),
                        b"next"
                    );
                })
            })
            .await?;
        check_order(&current);
        let reads: Vec<_> = current
            .events
            .iter()
            .filter_map(|e| match e {
                Event::IoIssued {
                    identity,
                    target,
                    operation: IoOperation::Read { .. },
                } => {
                    assert_eq!(identity.cid, second);
                    target.open_id
                }
                _ => None,
            })
            .collect();
        assert_eq!(reads, [1, 4]);
        Ok(())
    })
    .unwrap();
}

#[test]
fn abandoned_and_aborted_sync_observers_leave_flush_owned_by_driver() {
    let dir = TestDir::new();
    let profile = profile();
    ControlledDriver::run_async(dir.config(), profile.clone(), async move |driver| {
        calibrate(driver, &profile, START + 500);
        let result = driver.grant_async(step(3), Input::None, |driver| {
            driver.spawn_session(CID, STOP, |context| async move {
                let (blob, _) = context.open("storage", b"sync").await.unwrap();
                blob.write_at(0, vec![1], WriteOptions::default()).await.unwrap();
                drop(blob.start_sync().await);
                blob.write_at(1, vec![2], WriteOptions::default()).await.unwrap();
                let handle = blob.start_sync().await;
                handle.abort();
                assert!(matches!(handle.await, Err(NativeError::Aborted)));
                let read = blob.read_at(0, 2, ReadOptions::default()).await.unwrap();
                assert_eq!(read.coalesce().as_ref(), &[1, 2]);
            })
        }).await?;
        check_order(&result);
        assert!(result.events.iter().any(|e| matches!(e, Event::IoIssued { identity, operation: IoOperation::Sync { detached: true }, .. } if identity.io_id == 3)));
        assert!(result.events.iter().any(|e| matches!(e, Event::IoReleased { io_id: 3, delivered: false })));
        assert!(result.events.iter().any(|e| matches!(e, Event::IoReleased { io_id: 5, delivered: false })));
        Ok(())
    }).unwrap();
}

#[test]
fn scan_uses_raw_name_order_under_different_directory_creation_orders() {
    let replay = |order: [&'static [u8]; 3]| {
        let dir = TestDir::new();
        let profile = profile();
        ControlledDriver::run_async(dir.config(), profile.clone(), async move |driver| {
            calibrate(driver, &profile, START + 500);
            driver
                .grant_async(step(3), Input::None, |driver| {
                    driver.spawn_session(CID, STOP, move |context| async move {
                        for name in order {
                            drop(context.open("ordered", name).await.unwrap());
                        }
                    })
                })
                .await?;
            driver
                .grant_async(step(4), Input::None, |driver| {
                    driver.spawn_session([0x34; 32], STOP, |context| async move {
                        let names = context.scan("ordered").await.unwrap();
                        assert_eq!(names, [vec![0], vec![0, 1], vec![0xff]]);
                        for name in names {
                            drop(context.open("ordered", &name).await.unwrap());
                        }
                    })
                })
                .await
        })
        .unwrap()
    };
    assert_eq!(
        replay([b"\xff", b"\0\x01", b"\0"]),
        replay([b"\0", b"\xff", b"\0\x01"])
    );
}

#[test]
fn cancel_controller_grant_with_inflight_io_and_fail_before_result_release() {
    for cancel in [true, false] {
        let dir = TestDir::new();
        let profile = profile();
        let protocol_advanced = Arc::new(AtomicBool::new(false));
        let advanced = protocol_advanced.clone();
        let mut root_finished_cleanup = false;
        let result = ControlledDriver::run_async(dir.config(), profile.clone(), async |driver| {
            let release_hold = if cancel {
                Some(driver.hold_io_before_dispatch(1)?)
            } else {
                driver.fail_before_io_release(1)?;
                None
            };
            calibrate(driver, &profile, START + 500);
            let pending_grant = driver.grant_async(step(3), Input::None, |driver| {
                driver.spawn_session(CID, STOP, move |context| async move {
                    let (blob, _) = context.open("storage", b"unreleased").await.unwrap();
                    drop(blob);
                    advanced.store(true, Ordering::SeqCst);
                    pending::<()>().await;
                })
            });
            if cancel {
                let (tx, rx) = tokio::sync::oneshot::channel::<()>();
                let pump = tokio::spawn(async move {
                    pending::<()>().await;
                    drop(tx);
                });
                {
                    let mut grant = Box::pin(pending_grant);
                    poll_fn(|cx| {
                        assert!(grant.as_mut().poll(cx).is_pending());
                        Poll::Ready(())
                    })
                    .await;
                    pump.abort();
                    assert!(rx.await.is_err());
                }
            } else {
                assert!(pending_grant.await.is_err());
            }
            if let Some(release_hold) = release_hold {
                assert_eq!(driver.active_io_id(), Some(1));
                assert!(
                    !driver
                        .events()
                        .iter()
                        .any(|event| matches!(event, Event::IoReady { io_id: 1, .. }))
                );
                release_hold.send(()).unwrap();
            }
            assert!(driver.shutdown_async().await.is_err());
            let events = driver.events();
            assert!(events.iter().any(|e| matches!(
                e,
                Event::IoReady {
                    io_id: 1,
                    result: IoResult::Open { .. }
                }
            )));
            assert!(events.contains(&Event::IoDiscarded { io_id: 1 }));
            assert!(!events.iter().any(|e| matches!(
                e,
                Event::Ack { grant: 3 } | Event::IoReleased { io_id: 1, .. }
            )));
            assert!(matches!(events.last(), Some(Event::CleanupComplete)));
            root_finished_cleanup = true;
            Ok(())
        });
        assert!(result.is_err());
        assert!(root_finished_cleanup);
        assert!(!protocol_advanced.load(Ordering::SeqCst));
    }
}

#[test]
fn synchronous_grant_fails_closed_and_root_completes_accepted_disk_work() {
    let dir = TestDir::new();
    let profile = profile();
    let mut cleaned = false;
    let result = ControlledDriver::run_async(dir.config(), profile.clone(), async |driver| {
        calibrate(driver, &profile, START + 500);
        assert!(driver.set_io_wall_pause(std::time::Duration::ZERO).is_err());
        let result = driver.grant(step(3), Input::None, |driver| {
            driver.spawn_session(CID, STOP, |context| async move {
                let _ = context.open("storage", b"must-complete").await.unwrap();
                panic!("consumer polled during failure");
            })
        });
        assert!(result.is_err());
        assert!(driver.shutdown().is_err());
        assert!(driver.shutdown_async().await.is_err());
        let events = driver.events();
        assert!(events.iter().any(|event| matches!(
            event,
            Event::IoReady {
                io_id: 1,
                result: IoResult::Open { .. }
            }
        )));
        assert!(events.contains(&Event::IoDiscarded { io_id: 1 }));
        assert!(
            !events
                .iter()
                .any(|event| matches!(event, Event::Ack { grant: 3 } | Event::IoReleased { .. }))
        );
        assert!(matches!(events.last(), Some(Event::CleanupComplete)));
        cleaned = true;
        Ok(())
    });
    assert!(result.is_err());
    assert!(cleaned);
}
