//! Source-derived independent acceptance for the runtime slice, not full I1b.
//! Run this binary under an external wall cap: a native nonreturning poll cannot
//! be interrupted by a Tokio timer on the same executor.
use commonware_runtime::{
    Clock, Metrics, Runner as _, Spawner, Supervisor,
    tokio::{Config, ControlledJob, Runner},
};
use futures::future::{pending, poll_fn};
use poc_native_runtime::{
    ClockSample, ControlledDriver, Error, Event, Grant, Input, Profile, RngCheckpoint, RoleRng,
    SimContext, Step, Timer,
};
use rand_core::{CryptoRng, TryRng};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    cell::Cell,
    fs,
    future::Future,
    path::PathBuf,
    pin::Pin,
    sync::{
        Arc, Mutex, OnceLock,
        atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
    },
    task::{Context, Poll},
    time::{Duration, UNIX_EPOCH},
};

const T: u64 = 1_000_000;
const CID: [u8; 32] = [0x33; 32];
const STOP: i64 = 2_000_000;
static COUNT: AtomicU64 = AtomicU64::new(0);
static ROOT: OnceLock<PathBuf> = OnceLock::new();
fn wall_pause() {
    static DELAY: OnceLock<Duration> = OnceLock::new();
    let delay = DELAY.get_or_init(|| {
        Duration::from_millis(
            std::env::var("I1B_VERIFY_WALL_PAUSE_MS")
                .ok()
                .map(|s| s.parse::<u64>().unwrap())
                .unwrap_or(0),
        )
    });
    if !delay.is_zero() {
        std::thread::sleep(*delay);
    }
}
fn dir(label: &str) -> PathBuf {
    let root = ROOT.get_or_init(|| {
        if let Some(path) = std::env::var_os("I1B_VERIFY_ARTIFACT_DIR") {
            let path = PathBuf::from(path);
            fs::create_dir_all(&path).unwrap();
            return path;
        }
        let mut suffix = 0u64;
        loop {
            let path = std::env::temp_dir()
                .join(format!("i1b-independent-{}-{suffix}", std::process::id()));
            match fs::create_dir(&path) {
                Ok(()) => return path,
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                    suffix = suffix
                        .checked_add(1)
                        .expect("temporary root suffix exhausted");
                }
                Err(error) => panic!("reserve temporary root {}: {error}", path.display()),
            }
        }
    });
    let path = root.join(format!("{label}-{}", COUNT.fetch_add(1, Ordering::SeqCst)));
    fs::create_dir(&path).unwrap();
    path
}
fn config(path: &std::path::Path) -> Config {
    Config::default()
        .with_storage_directory(path)
        .with_worker_threads(1)
}
fn h(domain: &str, parts: &[&[u8]]) -> [u8; 32] {
    let mut x = Sha256::new();
    for p in std::iter::once(domain.as_bytes()).chain(parts.iter().copied()) {
        x.update((p.len() as u64).to_be_bytes());
        x.update(p);
    }
    x.finalize().into()
}
fn unhex(s: &str) -> [u8; 32] {
    let mut a = [0; 32];
    for (i, v) in a.iter_mut().enumerate() {
        *v = u8::from_str_radix(&s[i * 2..i * 2 + 2], 16).unwrap();
    }
    a
}
fn profile(seed: u64, eps: i64) -> Profile {
    Profile {
        run_id: [0x11; 32],
        config_hash: [0x22; 32],
        run_seed: seed,
        node: 0,
        incarnation: 1,
        epsilon_us: eps,
    }
}
fn sample(p: &Profile, request: u64, sequence: u64, at: u64, end: u64) -> ClockSample {
    ClockSample {
        run_id: p.run_id,
        node: p.node,
        incarnation: p.incarnation,
        clock_domain: h(
            "I1/clock-domain/v1",
            &[&p.run_id, &p.config_hash, &p.node.to_be_bytes()],
        ),
        rate_profile: h("I1/rate/v1", &[b"fixed-rate-one-v1"]),
        request_id: request,
        sequence,
        as_of_tick_us: at,
        node_time_at_as_of_us: at as i64 + p.epsilon_us,
        uncertainty_us: p.epsilon_us.unsigned_abs(),
        lease_end_tick_us: end,
    }
}
fn idle(_: &mut ControlledDriver) -> Result<(), Error> {
    Ok(())
}
fn step(d: &mut ControlledDriver, id: u64, t: u64, input: Input) -> Step {
    wall_pause();
    let result = d.grant(Grant { id, tick_us: t }, input, idle).unwrap();
    wall_pause();
    result
}
fn calibrate(d: &mut ControlledDriver, p: &Profile, start: u64, end: u64) {
    step(d, 1, start, Input::RequestSample);
    let s = step(
        d,
        2,
        start,
        Input::DeliverSample(sample(p, 1, 1, start, end)),
    );
    assert!(
        s.events
            .iter()
            .any(|e| matches!(e, Event::SampleAccepted { sequence: 1, .. }))
    );
}
fn timers(s: &Step) -> Vec<Timer> {
    s.events
        .iter()
        .filter_map(|e| {
            if let Event::TimerArmed { timer } = e {
                Some(timer.clone())
            } else {
                None
            }
        })
        .collect()
}
fn no_ack(events: &[Event], grant: u64) {
    assert!(
        !events
            .iter()
            .any(|e| matches!(e,Event::Ack{grant:g} if *g==grant))
    );
}
fn audit(events: &[Event]) {
    let mut failure = false;
    for e in events {
        match e {
            Event::Failure { .. } => failure = true,
            Event::Ack { .. } => assert!(!failure, "Ack after failure"),
            _ => {}
        }
    }
    assert!(
        events.iter().any(|e| matches!(e, Event::CleanupComplete))
            || events
                .iter()
                .any(|e| matches!(e, Event::CleanupBoundExceeded))
    );
}
fn run(
    label: &str,
    p: Profile,
    f: impl FnOnce(&mut ControlledDriver) -> Result<(), Error>,
) -> (bool, Vec<Event>) {
    let path = dir(label);
    let recorded = Arc::new(Mutex::new(Vec::new()));
    let out = recorded.clone();
    let result = ControlledDriver::run(config(&path), p.clone(), |d| {
        wall_pause();
        let result = f(d);
        let _ = d.shutdown();
        *out.lock().unwrap() = d.events();
        result
    });
    // This statement is reached only after the native Runner task tracker drains.
    let events = recorded.lock().unwrap().clone();
    audit(&events);
    fs::write(
        path.join("trace.json"),
        serde_json::to_vec_pretty(&events).unwrap(),
    )
    .unwrap();
    fs::write(path.join("outcome.json"),serde_json::to_vec_pretty(&json!({"runner_returned":true,"ok":result.is_ok(),"error":result.as_ref().err().map(ToString::to_string),"seed":p.run_seed,"epsilon_us":p.epsilon_us})).unwrap()).unwrap();
    (result.is_ok(), events)
}

struct DropCount {
    count: Arc<AtomicUsize>,
    bomb: bool,
}
impl Drop for DropCount {
    fn drop(&mut self) {
        wall_pause();
        self.count.fetch_add(1, Ordering::SeqCst);
        assert!(!self.bomb, "injected destructor panic");
    }
}
struct GuardDrop {
    ctx: SimContext,
    count: Arc<AtomicUsize>,
    allowed: Arc<AtomicBool>,
}
impl Drop for GuardDrop {
    fn drop(&mut self) {
        wall_pause();
        self.allowed
            .store(self.ctx.publication_allowed(), Ordering::SeqCst);
        self.count.fetch_add(1, Ordering::SeqCst);
    }
}
struct WakeTimes {
    polls: Arc<AtomicUsize>,
    target: usize,
    copies: usize,
}
struct PendingBomb {
    polls: Arc<AtomicUsize>,
    _drop: DropCount,
}
impl Future for PendingBomb {
    type Output = ();
    fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<()> {
        self.polls.fetch_add(1, Ordering::SeqCst);
        Poll::Pending
    }
}
impl Future for WakeTimes {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        let n = self.polls.fetch_add(1, Ordering::SeqCst) + 1;
        if n <= 2 {
            wall_pause();
        }
        if n >= self.target {
            Poll::Ready(())
        } else {
            for _ in 0..self.copies {
                cx.waker().wake_by_ref();
            }
            Poll::Pending
        }
    }
}

thread_local! {static NATIVE_PERMIT:Cell<bool>=const{Cell::new(false)};}
struct NativeDrop(Arc<AtomicUsize>);
impl Drop for NativeDrop {
    fn drop(&mut self) {
        wall_pause();
        NATIVE_PERMIT.with(|p| {
            assert!(
                p.get(),
                "native capture dropped outside test's scheduler permit"
            )
        });
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}

#[test]
fn p1_p4_native_registration_defers_factory_and_abort_drops_captures() {
    for abort in [false, true] {
        let path = dir("native_handoff");
        let ctor = Arc::new(AtomicUsize::new(0));
        let drops = Arc::new(AtomicUsize::new(0));
        let failed = Arc::new(AtomicBool::new(false));
        let c = ctor.clone();
        let before_poll = ctor.clone();
        let d = drops.clone();
        let fail = failed.clone();
        Runner::new(config(&path).with_catch_panics(true)).start(|context| async move {
            let slot = Arc::new(Mutex::new(None::<ControlledJob>));
            let owned = slot.clone();
            let capture = NativeDrop(d);
            let handle = context.child("native_unit").spawn_controlled(
                move |_| {
                    c.fetch_add(1, Ordering::SeqCst);
                    async move { drop(capture) }
                },
                fail,
                move |job| *owned.lock().unwrap() = Some(job),
            );
            assert_eq!(before_poll.load(Ordering::SeqCst), 0);
            if abort {
                handle.abort();
            }
            let mut job = slot.lock().unwrap().take().unwrap();
            NATIVE_PERMIT.with(|p| p.set(true));
            assert!(
                job.as_mut()
                    .poll(&mut Context::from_waker(futures::task::noop_waker_ref()))
                    .is_ready()
            );
            drop(job);
            NATIVE_PERMIT.with(|p| p.set(false));
            drop(handle);
        });
        assert_eq!(drops.load(Ordering::SeqCst), 1);
        assert_eq!(ctor.load(Ordering::SeqCst), usize::from(!abort));
        assert!(!failed.load(Ordering::SeqCst));
    }
}

#[test]
fn p2_p3_unawaited_constructor_and_poll_panics_stop_later_work() {
    for constructor in [false, true] {
        let p = profile(7, 1000);
        let calls = Arc::new(AtomicUsize::new(0));
        let seen = calls.clone();
        let (ok, events) = run("constructor_poll_panic", p.clone(), move |d| {
            calibrate(d, &p, T, T + 100_000);
            let result = d.grant(Grant { id: 3, tick_us: T }, Input::None, |d| {
                d.spawn_session(CID, STOP, move |ctx| async move {
                    drop(ctx.child("panic").spawn(move |_| {
                        wall_pause();
                        if constructor {
                            panic!("injected constructor panic")
                        };
                        async move {
                            wall_pause();
                            panic!("injected poll panic")
                        }
                    }));
                    drop(ctx.child("must_not_construct").spawn(move |_| {
                        seen.fetch_add(1, Ordering::SeqCst);
                        async {}
                    }));
                    pending::<()>().await;
                })
            });
            assert!(result.is_err());
            let later = d.grant(
                Grant {
                    id: 4,
                    tick_us: T + 1,
                },
                Input::None,
                |_| panic!("ordinary callback after failure"),
            );
            assert!(later.is_err());
            Ok(())
        });
        assert!(!ok);
        no_ack(&events, 3);
        assert_eq!(calls.load(Ordering::SeqCst), 0);
    }
}

#[test]
fn p5_cancel_pending_future_observes_drop_panic() {
    let p = profile(7, 1000);
    let dropped = Arc::new(AtomicUsize::new(0));
    let count = dropped.clone();
    let user_polls = Arc::new(AtomicUsize::new(0));
    let polls = user_polls.clone();
    let (ok, events) = run("pending_cancel", p.clone(), move |d| {
        calibrate(d, &p, T, T + 100_000);
        let first = d
            .grant(Grant { id: 3, tick_us: T }, Input::None, |d| {
                d.spawn_session(CID, STOP, move |ctx| async move {
                    let h = ctx.child("victim").spawn(move |_| PendingBomb {
                        polls,
                        _drop: DropCount { count, bomb: true },
                    });
                    ctx.sleep(Duration::from_micros(10)).await;
                    h.abort();
                    pending::<()>().await;
                })
            })
            .unwrap();
        let timer = timers(&first).pop().unwrap();
        assert!(
            d.grant(
                Grant {
                    id: 4,
                    tick_us: T + 10
                },
                Input::Timer(timer),
                idle
            )
            .is_err()
        );
        Ok(())
    });
    assert!(!ok);
    assert_eq!(dropped.load(Ordering::SeqCst), 1);
    assert_eq!(
        user_polls.load(Ordering::SeqCst),
        1,
        "cancelled user future was polled again"
    );
    no_ack(&events, 4);
    assert_eq!(
        events
            .iter()
            .filter(|e| matches!(e, Event::Construct { task: 2 }))
            .count(),
        1
    );
}

#[test]
fn p6_missing_receiver_and_p7_retained_handle_or_value_drop_are_observed() {
    for ownership in 0..3 {
        let p = profile(7, 1000);
        let drops = Arc::new(AtomicUsize::new(0));
        let probe = drops.clone();
        let (ok, events) = run("result_disposal", p.clone(), move |d| {
            calibrate(d, &p, T, T + 100_000);
            let first = d.grant(Grant { id: 3, tick_us: T }, Input::None, |d| {
                d.spawn_session(CID, STOP, move |ctx| async move {
                    let h = ctx.child("producer").spawn(move |_| async move {
                        DropCount {
                            count: probe,
                            bomb: true,
                        }
                    });
                    match ownership {
                        0 => {
                            drop(h);
                            pending::<()>().await;
                        }
                        1 => {
                            ctx.sleep(Duration::from_micros(10)).await;
                            drop(h);
                        }
                        _ => {
                            let value = h.await.unwrap();
                            ctx.sleep(Duration::from_micros(10)).await;
                            drop(value);
                        }
                    }
                })
            });
            if ownership == 0 {
                assert!(first.is_err());
            } else {
                let first = first.unwrap();
                assert!(
                    !first
                        .events
                        .iter()
                        .any(|e| matches!(e, Event::Failure { .. }))
                );
                let timer = timers(&first).pop().unwrap();
                assert!(
                    d.grant(
                        Grant {
                            id: 4,
                            tick_us: T + 10
                        },
                        Input::Timer(timer),
                        idle
                    )
                    .is_err()
                );
            }
            Ok(())
        });
        assert!(!ok);
        assert_eq!(drops.load(Ordering::SeqCst), 1);
        no_ack(&events, if ownership == 0 { 3 } else { 4 });
        if ownership > 0 {
            let producer = events
                .iter()
                .position(|e| matches!(e, Event::Dispose { task: 2, .. }))
                .unwrap();
            let timer_fire = events
                .iter()
                .position(|e| matches!(e, Event::TimerFired { .. }))
                .unwrap();
            assert!(producer < timer_fire);
        }
    }
}

#[test]
fn p8_normal_shutdown_explicit_job_drop_sets_failure_from_false() {
    let p = profile(7, 1000);
    let drops = Arc::new(AtomicUsize::new(0));
    let probe = drops.clone();
    let (ok, events) = run("explicit_drop", p.clone(), move |d| {
        calibrate(d, &p, T, T + 100_000);
        let first = d
            .grant(Grant { id: 3, tick_us: T }, Input::None, |d| {
                d.spawn_session(CID, STOP, move |_| async move {
                    let bomb = DropCount {
                        count: probe,
                        bomb: true,
                    };
                    pending::<()>().await;
                    drop(bomb)
                })
            })
            .unwrap();
        assert!(
            !first
                .events
                .iter()
                .any(|e| matches!(e, Event::Failure { .. }))
        );
        assert!(d.shutdown().is_err());
        Ok(())
    });
    assert!(!ok);
    assert_eq!(drops.load(Ordering::SeqCst), 1);
    assert_eq!(
        events
            .iter()
            .filter(|e| matches!(e, Event::Poll { task: 1, .. }))
            .count(),
        1
    );
    assert!(events.iter().any(|e| matches!(
        e,
        Event::Failure {
            reason: "native disposal unwind"
        }
    )));
}

#[test]
fn p9_parent_completion_and_abort_close_nested_unspawned_scopes() {
    for abort in [false, true] {
        let p = profile(7, 1000);
        let ctor = Arc::new(AtomicUsize::new(0));
        let drops = Arc::new(AtomicUsize::new(0));
        let c = ctor.clone();
        let probe = drops.clone();
        let (ok, events) = run("nested_supervision", p.clone(), move |d| {
            calibrate(d, &p, T, T + 100_000);
            d.grant(Grant { id: 3, tick_us: T }, Input::None, |d| {
                d.spawn_session(CID, STOP, move |ctx| async move {
                    let capture = DropCount {
                        count: probe,
                        bomb: false,
                    };
                    let handle = ctx
                        .child("unspawned_a")
                        .child("unspawned_b")
                        .spawn(move |_| {
                            c.fetch_add(1, Ordering::SeqCst);
                            async move {
                                pending::<()>().await;
                                drop(capture)
                            }
                        });
                    if abort {
                        handle.abort();
                        pending::<()>().await;
                    } else {
                        drop(handle);
                    }
                })
            })?;
            Ok(())
        });
        assert!(ok);
        assert_eq!(ctor.load(Ordering::SeqCst), 0);
        assert_eq!(drops.load(Ordering::SeqCst), 1);
        let ack = events
            .iter()
            .position(|e| matches!(e, Event::Ack { grant: 3 }))
            .unwrap();
        let leaf = events
            .iter()
            .position(|e| matches!(e, Event::Dispose { task: 2, .. }))
            .unwrap();
        assert!(leaf < ack);
    }
}

#[test]
fn p9_parent_abort_cascades_before_grandchild_first_poll() {
    let p = profile(7, 1000);
    let constructed = Arc::new(AtomicUsize::new(0));
    let calls = constructed.clone();
    let dropped = Arc::new(AtomicUsize::new(0));
    let count = dropped.clone();
    let (ok, events) = run("parent_abort", p.clone(), move |d| {
        calibrate(d, &p, T, T + 100_000);
        d.grant(Grant { id: 3, tick_us: T }, Input::None, |d| {
            d.spawn_session(CID, STOP, move |ctx| async move {
                let (tx, rx) = tokio::sync::oneshot::channel::<()>();
                let middle = ctx.child("middle").spawn(move |ctx| async move {
                    let capture = DropCount { count, bomb: false };
                    drop(ctx.child("unspawned_scope").child("leaf").spawn(move |_| {
                        calls.fetch_add(1, Ordering::SeqCst);
                        async move {
                            pending::<()>().await;
                            drop(capture);
                        }
                    }));
                    tx.send(()).unwrap();
                    pending::<()>().await;
                });
                rx.await.unwrap();
                middle.abort();
                pending::<()>().await;
            })
        })?;
        Ok(())
    });
    assert!(ok);
    assert_eq!(constructed.load(Ordering::SeqCst), 0);
    assert_eq!(dropped.load(Ordering::SeqCst), 1);
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::Construct { task: 2 }))
    );
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, Event::Construct { task: 3 }))
    );
    let leaf = events
        .iter()
        .position(|e| matches!(e, Event::Dispose { task: 3, .. }))
        .unwrap();
    let ack = events
        .iter()
        .position(|e| matches!(e, Event::Ack { grant: 3 }))
        .unwrap();
    assert!(leaf < ack);
}

#[test]
fn p10_cancelled_transport_pump_leaves_registry_with_root() {
    let path = dir("pump_loss");
    let p = profile(7, 1000);
    let drops = Arc::new(AtomicUsize::new(0));
    let capture = drops.clone();
    let after = Arc::new(Mutex::new(Vec::new()));
    let saved = after.clone();
    let result = ControlledDriver::run_async(config(&path), p.clone(), async move |d| {
        calibrate(d, &p, T, T + 100_000);
        d.grant(Grant { id: 3, tick_us: T }, Input::None, |d| {
            d.spawn_session(CID, STOP, move |_| async move {
                let guard = DropCount {
                    count: capture,
                    bomb: false,
                };
                pending::<()>().await;
                drop(guard)
            })
        })?;
        let (tx, rx) = tokio::sync::oneshot::channel::<()>();
        let pump = tokio::spawn(async move {
            pending::<()>().await;
            drop(tx)
        });
        pump.abort();
        assert!(rx.await.is_err());
        assert!(d.fail_transport().is_err());
        *saved.lock().unwrap() = d.events();
        Ok(())
    });
    assert!(result.is_err());
    assert_eq!(drops.load(Ordering::SeqCst), 1);
    audit(&after.lock().unwrap());
    fs::write(
        path.join("trace.json"),
        serde_json::to_vec_pretty(&*after.lock().unwrap()).unwrap(),
    )
    .unwrap();
}

#[test]
fn p11_driver_panic_and_repeated_failed_reentry_preserve_custody() {
    for first in 0..4 {
        let p = profile(7, 1000);
        let drops = Arc::new(AtomicUsize::new(0));
        let count = drops.clone();
        let allowed = Arc::new(AtomicBool::new(true));
        let observed_drops = drops.clone();
        let publish = allowed.clone();
        let (ok, events) = run("driver_error_reentry", p.clone(), move |d| {
            calibrate(d, &p, T, T + 100_000);
            d.grant(Grant { id: 3, tick_us: T }, Input::None, |d| {
                d.spawn_session(CID, STOP, move |ctx| async move {
                    let guard = GuardDrop {
                        ctx,
                        count,
                        allowed: publish,
                    };
                    pending::<()>().await;
                    drop(guard)
                })
            })?;
            let r = d.grant(
                Grant {
                    id: 4,
                    tick_us: T + 1,
                },
                Input::None,
                |d| {
                    if first == 0 {
                        panic!("injected action panic");
                    }
                    if first == 1 {
                        assert!(
                            d.grant(
                                Grant {
                                    id: 5,
                                    tick_us: T + 2
                                },
                                Input::None,
                                |_| panic!("nested callback")
                            )
                            .is_err()
                        );
                    }
                    if first == 2 {
                        assert!(d.fail_transport().is_err());
                    }
                    if first == 3 {
                        assert!(d.shutdown().is_err());
                    }
                    for id in [6, 7] {
                        assert!(
                            d.grant(Grant { id, tick_us: T + 3 }, Input::None, |_| panic!(
                                "callback after failure"
                            ))
                            .is_err()
                        );
                    }
                    assert_eq!(drops.load(Ordering::SeqCst), 0);
                    Ok(())
                },
            );
            assert!(r.is_err());
            Ok(())
        });
        assert!(!ok);
        assert!(!allowed.load(Ordering::SeqCst));
        assert_eq!(observed_drops.load(Ordering::SeqCst), 1);
        no_ack(&events, 4);
        assert!(!events.iter().any(|e| matches!(
            e,
            Event::Grant {
                grant: 5 | 6 | 7,
                ..
            }
        )));
    }
}

#[test]
fn p11_root_controller_panic_still_returns_from_native_runner() {
    let path = dir("controller_panic");
    let p = profile(7, 1000);
    let drops = Arc::new(AtomicUsize::new(0));
    let count = drops.clone();
    let injected = Arc::new(AtomicBool::new(false));
    let reached = injected.clone();
    let result: Result<(), Error> = ControlledDriver::run(config(&path), p.clone(), move |d| {
        calibrate(d, &p, T, T + 100_000);
        d.grant(Grant { id: 3, tick_us: T }, Input::None, |d| {
            d.spawn_session(CID, STOP, move |_| async move {
                let guard = DropCount { count, bomb: false };
                pending::<()>().await;
                drop(guard)
            })
        })?;
        reached.store(true, Ordering::SeqCst);
        panic!("injected root controller panic")
    });
    assert!(result.is_err());
    assert!(injected.load(Ordering::SeqCst));
    assert_eq!(drops.load(Ordering::SeqCst), 1);
    fs::write(
        path.join("outcome.json"),
        b"{\"runner_returned\":true,\"dropped\":1,\"expected_failure\":true}",
    )
    .unwrap();
}

#[test]
fn p12_cancel_drop_wakes_sibling_and_quiescence_waits_for_it() {
    let p = profile(7, 1000);
    let received = Arc::new(AtomicUsize::new(0));
    let seen = received.clone();
    let (ok, events) = run("drop_wake_fixed_point", p.clone(), move |d| {
        calibrate(d, &p, T, T + 100_000);
        let first = d
            .grant(Grant { id: 3, tick_us: T }, Input::None, |d| {
                d.spawn_session(CID, STOP, move |ctx| async move {
                    let (tx, rx) = tokio::sync::oneshot::channel::<()>();
                    let victim = ctx.child("sender_owner").spawn(move |_| async move {
                        pending::<()>().await;
                        drop(tx)
                    });
                    drop(ctx.child("receiver").spawn(move |_| async move {
                        assert!(rx.await.is_err());
                        seen.fetch_add(1, Ordering::SeqCst);
                    }));
                    ctx.sleep(Duration::from_micros(10)).await;
                    victim.abort();
                    pending::<()>().await;
                })
            })
            .unwrap();
        let timer = timers(&first).pop().unwrap();
        let after = step(d, 4, T + 10, Input::Timer(timer));
        assert!(matches!(after.events.last(), Some(Event::Ack { grant: 4 })));
        Ok(())
    });
    assert!(ok);
    assert_eq!(received.load(Ordering::SeqCst), 1);
    let receiver = events
        .iter()
        .position(|e| matches!(e, Event::Dispose { task: 3, .. }))
        .unwrap();
    let ack = events
        .iter()
        .position(|e| matches!(e, Event::Ack { grant: 4 }))
        .unwrap();
    assert!(receiver < ack);
}

struct ForbiddenDrop {
    ctx: Option<SimContext>,
    constructed: Arc<AtomicUsize>,
}
impl Drop for ForbiddenDrop {
    fn drop(&mut self) {
        let c = self.constructed.clone();
        drop(
            self.ctx
                .take()
                .unwrap()
                .child("forbidden_cleanup_constructor")
                .spawn(move |_| {
                    c.fetch_add(1, Ordering::SeqCst);
                    async {}
                }),
        );
    }
}

#[test]
fn p12_p13_cleanup_forbids_new_constructors_and_failed_step_is_not_released() {
    let p = profile(7, 1000);
    let forbidden = Arc::new(AtomicUsize::new(0));
    let c = forbidden.clone();
    let staged = Arc::new(AtomicUsize::new(0));
    let s = staged.clone();
    let released = AtomicUsize::new(0);
    let (ok, events) = run("cleanup_permissions", p.clone(), move |d| {
        calibrate(d, &p, T, T + 100_000);
        let result = d.grant(Grant { id: 3, tick_us: T }, Input::None, |d| {
            d.spawn_session(CID, STOP, move |ctx| async move {
                assert!(ctx.publication_allowed());
                s.fetch_add(1, Ordering::SeqCst);
                let capture = ForbiddenDrop {
                    ctx: Some(ctx.child("capture")),
                    constructed: c,
                };
                drop(
                    ctx.child("panic")
                        .spawn(move |_| async move { panic!("failure before release") }),
                );
                pending::<()>().await;
                drop(capture);
            })
        });
        if result.is_ok() {
            released.fetch_add(1, Ordering::SeqCst);
        }
        assert!(result.is_err());
        assert_eq!(released.load(Ordering::SeqCst), 0);
        Ok(())
    });
    assert!(!ok);
    assert_eq!(staged.load(Ordering::SeqCst), 1);
    assert_eq!(forbidden.load(Ordering::SeqCst), 0);
    no_ack(&events, 3);
}

#[test]
fn p14_exact_default_poll_bound_and_cleanup_limit_report_truthfully() {
    for target in [10_000, 10_001] {
        let p = profile(7, 1000);
        let polls = Arc::new(AtomicUsize::new(0));
        let count = polls.clone();
        let (ok, events) = run("poll_bound", p.clone(), move |d| {
            calibrate(d, &p, T, T + 100_000);
            let result = d.grant(Grant { id: 3, tick_us: T }, Input::None, |d| {
                d.spawn_session(CID, STOP, move |_| WakeTimes {
                    polls: count,
                    target,
                    copies: 3,
                })
            });
            assert_eq!(result.is_ok(), target == 10_000);
            Ok(())
        });
        assert_eq!(ok, target == 10_000);
        assert_eq!(polls.load(Ordering::SeqCst), 10_000);
        if target > 10_000 {
            no_ack(&events, 3);
        }
    }
    let p = profile(7, 1000);
    let (ok, events) = run("cleanup_bound", p.clone(), move |d| {
        d.with_limits(10, 0);
        calibrate(d, &p, T, T + 100_000);
        d.grant(Grant { id: 3, tick_us: T }, Input::None, |d| {
            d.spawn_session(CID, STOP, |_| pending::<()>())
        })?;
        assert!(d.shutdown().is_err());
        Ok(())
    });
    assert!(!ok);
    assert!(events.contains(&Event::CleanupBoundExceeded));
    assert!(!events.contains(&Event::CleanupComplete));
}

#[test]
fn clock_projection_governor_and_deadline_ceil_require_granted_timer() {
    for eps in [-1000, 1000] {
        let p = profile(7, eps);
        let done = Arc::new(AtomicBool::new(false));
        let finished = done.clone();
        let (ok, _) = run("clock_ceil", p.clone(), move |d| {
            step(d, 1, T, Input::RequestSample);
            step(
                d,
                2,
                T + 10_000,
                Input::DeliverSample(sample(&p, 1, 1, T, T + 100_000)),
            );
            let first = d
                .grant(
                    Grant {
                        id: 3,
                        tick_us: T + 10_000,
                    },
                    Input::None,
                    |d| {
                        assert_eq!(d.manager_time_us().unwrap(), 1_010_000 + eps);
                        d.spawn_session(CID, STOP, move |ctx| async move {
                            fn bounds<C: Clock + CryptoRng + Metrics + Spawner + Supervisor>(
                                _: &C,
                            ) {
                            }
                            bounds(&ctx);
                            let now = ctx.current();
                            assert_eq!(now, governor::clock::Clock::now(&ctx));
                            assert_eq!(
                                now,
                                UNIX_EPOCH + Duration::from_micros((1_010_000 + eps) as u64)
                            );
                            ctx.sleep_until(now + Duration::from_nanos(1)).await;
                            finished.store(true, Ordering::SeqCst);
                        })
                    },
                )
                .unwrap();
            let timer = timers(&first).pop().unwrap();
            assert_eq!(timer.deadline_us, 1_010_001 + eps);
            step(d, 4, T + 10_000, Input::Timer(timer.clone()));
            assert!(!done.load(Ordering::SeqCst));
            step(d, 5, T + 10_001, Input::None);
            assert!(!done.load(Ordering::SeqCst));
            step(d, 6, T + 10_001, Input::Timer(timer));
            assert!(done.load(Ordering::SeqCst));
            Ok(())
        });
        assert!(ok);
    }
}

#[test]
fn duration_deadline_is_captured_before_first_poll_and_past_time_needs_event() {
    let p = profile(7, 0);
    let done = Arc::new(AtomicBool::new(false));
    let flag = done.clone();
    let (ok, _) = run("sleep_construction", p.clone(), move |d| {
        calibrate(d, &p, T, T + 100_000);
        let first = d
            .grant(Grant { id: 3, tick_us: T }, Input::None, |d| {
                d.spawn_session(CID, STOP, move |ctx| async move {
                    let saved = ctx.sleep(Duration::from_micros(1000));
                    ctx.sleep(Duration::from_micros(2000)).await;
                    saved.await;
                    flag.store(true, Ordering::SeqCst);
                })
            })
            .unwrap();
        let gate = timers(&first).pop().unwrap();
        assert_eq!(gate.deadline_us, (T + 2000) as i64);
        let later = step(d, 4, T + 2000, Input::Timer(gate));
        let saved = timers(&later).pop().unwrap();
        assert_eq!(saved.deadline_us, (T + 1000) as i64);
        assert!(!done.load(Ordering::SeqCst));
        step(d, 5, T + 2000, Input::None);
        assert!(!done.load(Ordering::SeqCst));
        step(d, 6, T + 2000, Input::Timer(saved));
        assert!(done.load(Ordering::SeqCst));
        Ok(())
    });
    assert!(ok);
}

#[test]
fn signed_pre_epoch_deadline_rounding_is_ceiling() {
    for (nanos, expected_us, due) in [(1u64, 0i64, 1000u64), (1001, -1, 999)] {
        let p = profile(7, -1000);
        let done = Arc::new(AtomicBool::new(false));
        let flag = done.clone();
        let (ok, _) = run("pre_epoch", p.clone(), move |d| {
            calibrate(d, &p, 0, 100_000);
            let first = d
                .grant(Grant { id: 3, tick_us: 0 }, Input::None, |d| {
                    d.spawn_session(CID, STOP, move |ctx| async move {
                        assert_eq!(ctx.current(), UNIX_EPOCH - Duration::from_micros(1000));
                        ctx.sleep_until(UNIX_EPOCH - Duration::from_nanos(nanos))
                            .await;
                        flag.store(true, Ordering::SeqCst);
                    })
                })
                .unwrap();
            let timer = timers(&first).pop().unwrap();
            assert_eq!(timer.deadline_us, expected_us);
            step(d, 4, due - 1, Input::Timer(timer.clone()));
            assert!(!done.load(Ordering::SeqCst));
            step(d, 5, due, Input::Timer(timer));
            assert!(done.load(Ordering::SeqCst));
            Ok(())
        });
        assert!(ok);
    }
}

#[test]
fn clock_expiry_and_local_stop_retire_only_old_cid() {
    for local_stop in [false, true] {
        let p = profile(7, 1000);
        let new_ctor = Arc::new(AtomicUsize::new(0));
        let new_seen = new_ctor.clone();
        let (ok, events) = run("retirement", p.clone(), move |d| {
            calibrate(d, &p, T, if local_stop { T + 100_000 } else { T + 1000 });
            d.grant(Grant { id: 3, tick_us: T }, Input::None, |d| {
                d.spawn_session(
                    CID,
                    if local_stop { (T + 2000) as i64 } else { STOP },
                    |_| pending::<()>(),
                )
            })?;
            let retired = step(d, 4, T + 1000, Input::None);
            assert!(
                retired
                    .events
                    .iter()
                    .any(|e| matches!(e,Event::Retired{cid,..} if *cid==CID))
            );
            step(d, 5, T + 1001, Input::RequestSample);
            let mut bad = sample(&p, 2, 100, T + 1001, T + 100_000);
            bad.node_time_at_as_of_us += 1;
            let invalid = step(d, 6, T + 1002, Input::DeliverSample(bad));
            assert!(
                invalid
                    .events
                    .iter()
                    .any(|e| matches!(e, Event::SampleRejected { .. }))
            );
            step(
                d,
                7,
                T + 1003,
                Input::DeliverSample(sample(&p, 2, 2, T + 1001, T + 100_000)),
            );
            d.grant(
                Grant {
                    id: 8,
                    tick_us: T + 1004,
                },
                Input::None,
                |d| {
                    assert!(d.spawn_session(CID, STOP, |_| async {}).is_err());
                    d.spawn_session([0x44; 32], STOP, move |_| {
                        new_seen.fetch_add(1, Ordering::SeqCst);
                        async {}
                    })
                },
            )?;
            Ok(())
        });
        assert!(ok);
        assert_eq!(new_ctor.load(Ordering::SeqCst), 1);
        assert!(!events.iter().any(|e| matches!(e, Event::Failure { .. })));
    }
}

#[test]
fn sample_identity_time_and_sequence_rejections_do_not_poison_calibration() {
    let p = profile(7, 1000);
    let (ok, events) = run("sample_rejections", p.clone(), move |d| {
        calibrate(d, &p, T, T + 100_000);
        step(d, 3, T + 1, Input::RequestSample);
        let good = sample(&p, 2, 3, T + 1, T + 200_000);
        for mode in 0..10 {
            let mut bad = good.clone();
            match mode {
                0 => {
                    bad.sequence = 100;
                    bad.node_time_at_as_of_us += 1;
                }
                1 => bad.run_id = [0; 32],
                2 => bad.node = 1,
                3 => bad.incarnation = 2,
                4 => bad.clock_domain = [0; 32],
                5 => bad.rate_profile = [0; 32],
                6 => bad.request_id = 99,
                7 => bad.as_of_tick_us = T + 100,
                8 => bad.lease_end_tick_us = T + 2,
                _ => bad.uncertainty_us = 999,
            }
            let s = d
                .grant(
                    Grant {
                        id: 4 + mode,
                        tick_us: T + 2,
                    },
                    Input::DeliverSample(bad),
                    |d| {
                        assert_eq!(
                            d.manager_time_us().unwrap(),
                            (T + 2) as i64 + p.epsilon_us,
                            "rejected sample changed the prior usable NodeClock"
                        );
                        Ok(())
                    },
                )
                .unwrap();
            assert!(
                s.events
                    .iter()
                    .any(|e| matches!(e, Event::SampleRejected { .. }))
            );
            assert!(
                !s.events
                    .iter()
                    .any(|e| matches!(e, Event::SampleAccepted { .. }))
            );
        }
        let accepted = step(d, 14, T + 2, Input::DeliverSample(good.clone()));
        assert!(
            accepted
                .events
                .iter()
                .any(|e| matches!(e, Event::SampleAccepted { sequence: 3, .. }))
        );
        let duplicate = step(d, 15, T + 2, Input::DeliverSample(good));
        assert!(
            duplicate
                .events
                .iter()
                .any(|e| matches!(e, Event::SampleRejected { .. }))
        );
        Ok(())
    });
    assert!(ok);
    assert!(!events.iter().any(|e| matches!(e, Event::Failure { .. })));
}

#[test]
fn actual_metrics_registration_names_attributes_and_lifetime_are_delegated() {
    use commonware_runtime::telemetry::metrics::raw;
    let p = profile(7, 1000);
    let (ok, _) = run("native_metrics", p.clone(), move |d| {
        calibrate(d, &p, T, T + 100_000);
        d.grant(Grant { id: 3, tick_us: T }, Input::None, |d| {
            d.spawn_session(CID, STOP, |ctx| async move {
                let scope = ctx.child("probe").with_attribute("epoch", 5);
                assert_eq!(scope.name().label, "session_probe");
                assert_eq!(
                    scope.name().attributes,
                    vec![("epoch".to_owned(), "5".to_owned())]
                );
                let first = scope.register(
                    "hits",
                    "Independent native metric",
                    raw::Counter::<u64, AtomicU64>::default(),
                );
                first.inc();
                let second = scope.register(
                    "hits",
                    "Independent native metric",
                    raw::Counter::<u64, AtomicU64>::default(),
                );
                second.inc();
                let expected = |encoded: &str| {
                    encoded.lines().any(|line| {
                        line.starts_with("session_probe_hits")
                            && line.contains("epoch=\"5\"")
                            && line.ends_with(" 2")
                    })
                };
                assert!(expected(&scope.encode()));
                drop(first);
                assert!(expected(&scope.encode()));
                drop(second);
                assert!(
                    !scope
                        .encode()
                        .lines()
                        .any(|line| line.starts_with("session_probe_hits"))
                );
            })
        })?;
        Ok(())
    });
    assert!(ok);
}

#[test]
fn regression_sleep_can_transfer_between_sibling_tasks_without_deadline_reset() {
    let p = profile(7, 1000);
    let done = Arc::new(AtomicBool::new(false));
    let flag = done.clone();
    let (ok, events) = run("sleep_transfer", p.clone(), move |d| {
        calibrate(d, &p, T, T + 100_000);
        let first = d
            .grant(Grant { id: 3, tick_us: T }, Input::None, |d| {
                d.spawn_session(CID, STOP, move |ctx| async move {
                    let (tx, rx) =
                        tokio::sync::oneshot::channel::<Pin<Box<dyn Future<Output = ()> + Send>>>();
                    drop(ctx.child("source").spawn(move |ctx| async move {
                        let mut sleep: Pin<Box<dyn Future<Output = ()> + Send>> =
                            Box::pin(ctx.sleep(Duration::from_micros(50)));
                        poll_fn(|cx| {
                            assert!(sleep.as_mut().poll(cx).is_pending());
                            Poll::Ready(())
                        })
                        .await;
                        assert!(tx.send(sleep).is_ok());
                        pending::<()>().await;
                    }));
                    drop(ctx.child("consumer").spawn(move |_| async move {
                        rx.await.unwrap().await;
                        flag.store(true, Ordering::SeqCst);
                    }));
                    pending::<()>().await;
                })
            })
            .unwrap();
        let timer = timers(&first).pop().unwrap();
        assert_eq!(timer.deadline_us, (T + 1050) as i64);
        assert!(
            first
                .events
                .iter()
                .any(|e| matches!(e, Event::TimerTransferred { from: 2, to: 3, .. }))
        );
        step(d, 4, T + 49, Input::Timer(timer.clone()));
        assert!(!done.load(Ordering::SeqCst));
        let fired = step(d, 5, T + 50, Input::Timer(timer));
        assert!(done.load(Ordering::SeqCst));
        assert!(
            !fired
                .events
                .iter()
                .any(|e| matches!(e, Event::Poll { task: 2, .. }))
        );
        Ok(())
    });
    assert!(ok);
    assert_eq!(
        events
            .iter()
            .filter(|e| matches!(e, Event::TimerArmed { .. }))
            .count(),
        1
    );
}

#[test]
fn default_timeout_ready_result_wins_a_same_poll_tie() {
    for ready in [false, true] {
        let p = profile(7, 1000);
        let observed = Arc::new(Mutex::new(None));
        let result = observed.clone();
        let (ok, _) = run("timeout_tie", p.clone(), move |d| {
            calibrate(d, &p, T, T + 100_000);
            let (tx, rx) = tokio::sync::oneshot::channel::<u8>();
            let mut sender = Some(tx);
            let first = d
                .grant(Grant { id: 3, tick_us: T }, Input::None, |d| {
                    d.spawn_session(CID, STOP, move |ctx| async move {
                        let r = ctx
                            .timeout(Duration::from_micros(100), async move { rx.await.unwrap() })
                            .await;
                        *result.lock().unwrap() = Some(r.map_err(|e| e.to_string()));
                    })
                })
                .unwrap();
            let timer = timers(&first).pop().unwrap();
            d.grant(
                Grant {
                    id: 4,
                    tick_us: T + 100,
                },
                Input::Timer(timer),
                |_| {
                    if ready {
                        assert!(sender.take().unwrap().send(9).is_ok());
                    }
                    Ok(())
                },
            )?;
            drop(sender);
            let v = observed.lock().unwrap();
            let r = v.as_ref().unwrap();
            assert_eq!(r.is_ok(), ready);
            if ready {
                assert_eq!(r.as_ref().unwrap(), &9);
            }
            Ok(())
        });
        assert!(ok);
    }
}

#[test]
fn graceful_stop_preserves_first_value_and_signal_holder_timeout() {
    let p = profile(7, 1000);
    let values = Arc::new(Mutex::new(Vec::<i32>::new()));
    let out = values.clone();
    let (ok, _) = run("graceful_stop", p.clone(), move |d| {
        calibrate(d, &p, T, T + 100_000);
        let first = d
            .grant(Grant { id: 3, tick_us: T }, Input::None, |d| {
                d.spawn_session(CID, STOP, move |ctx| async move {
                    let mut holder = ctx.stopped();
                    let v = out.clone();
                    drop(ctx.child("stop_first").spawn(move |ctx| async move {
                        let r = ctx.stop(17, Some(Duration::from_micros(100))).await;
                        v.lock().unwrap().push(if r.is_err() { -1 } else { -2 });
                    }));
                    let first = (&mut holder).await.unwrap();
                    out.lock().unwrap().push(first);
                    assert_eq!(ctx.stopped().await.unwrap(), 17);
                    let v = out.clone();
                    drop(ctx.child("stop_second").spawn(move |ctx| async move {
                        ctx.stop(99, None).await.unwrap();
                        v.lock().unwrap().push(99);
                    }));
                    ctx.sleep(Duration::from_micros(200)).await;
                    drop(holder);
                    pending::<()>().await;
                })
            })
            .unwrap();
        let all = timers(&first);
        assert_eq!(all.len(), 2);
        let timeout = all
            .iter()
            .find(|x| x.deadline_us == (T + 1100) as i64)
            .unwrap()
            .clone();
        let release = all
            .iter()
            .find(|x| x.deadline_us == (T + 1200) as i64)
            .unwrap()
            .clone();
        step(d, 4, T + 100, Input::Timer(timeout));
        assert_eq!(*values.lock().unwrap(), vec![17, -1]);
        step(d, 5, T + 200, Input::Timer(release));
        assert_eq!(*values.lock().unwrap(), vec![17, -1, 99]);
        Ok(())
    });
    assert!(ok);
}

fn checkpoint(seed: [u8; 32], stream: u64, position: u128) -> RngCheckpoint {
    RngCheckpoint {
        seed,
        stream,
        word_position_hi: (position >> 64) as u64,
        word_position_lo: position as u64,
    }
}

#[test]
fn rng_literal_mixed_fill_and_seek_cursor_vectors() {
    let mut r = RoleRng::restore(checkpoint([0; 32], 0, 0)).unwrap();
    assert_eq!(r.try_next_u32().unwrap(), 0xade0b876);
    assert_eq!(r.checkpoint().word_position_lo, 1);
    assert_eq!(r.try_next_u64().unwrap(), 0xe56a5d40903df1a0);
    let mut three = [0; 3];
    r.try_fill_bytes(&mut three).unwrap();
    assert_eq!(three, [0x53, 0x86, 0xbd]);
    assert_eq!(r.try_next_u32().unwrap(), 0xb819d2bd);
    let mut five = [0; 5];
    r.try_fill_bytes(&mut five).unwrap();
    assert_eq!(five, [0xa0, 0x8d, 0xed, 0x1a, 0xa8]);
    assert_eq!(r.checkpoint().word_position_lo, 7);
    r.try_fill_bytes(&mut []).unwrap();
    assert_eq!(r.checkpoint().word_position_lo, 7);
    assert_eq!(r.try_next_u64().unwrap(), 0x7c5941dac70d778b);
    let mut restored = RoleRng::restore(r.checkpoint()).unwrap();
    assert_eq!(restored.try_next_u64().unwrap(), 0x3fe024778d485751);
    for (stream, pos, value) in [
        (0, 15, 0xbee7079f8665eeb2),
        (0, 63, 0x7488a6e57e166731),
        (51, 15, 0x922704dfb05e7395),
        (51, 63, 0x9782fb529d5bfa29),
    ] {
        let mut r = RoleRng::restore(checkpoint([0; 32], stream, pos)).unwrap();
        assert_eq!(r.try_next_u64().unwrap(), value);
        assert_eq!(r.checkpoint().word_position_lo, (pos + 2) as u64);
    }
}

#[test]
fn rng_kdf_literal_paths_and_registry_reuse() {
    let root = unhex("7e18f65d5736bd4967d4213d11aeab38e5fb52520f9fd3e625c6d476396d379c");
    let voter = unhex("ccb1b4a0e0532ddc1d79bafea0f5feec8fcb2ef8f668b3c2df0a097f89f834d0");
    let attr = unhex("6eefcf1ca14a62f0266612905bb661f27397a0aeca6207bc6494c3b02d6ec9ff");
    assert_eq!(RoleRng::root_role(), root);
    assert_eq!(RoleRng::child_role(root, "voter"), voter);
    assert_eq!(RoleRng::attribute_role(voter, "epoch", "5"), attr);
    for (role, seed) in [
        (
            root,
            "6b906de1ead6e62491ccf447cf1d77e95f29e14f0b12593a7dbe3852b3a4f282",
        ),
        (
            voter,
            "aca4bc8e1b450449f0813234ac45bf9bfc037803f72132437483e49ba92dc6c8",
        ),
        (
            attr,
            "6c07d3625500da2a95e23c6a9fbea867c5d6072f6d98e41c90e85a1ab12d1882",
        ),
    ] {
        assert_eq!(
            RoleRng::derive(7, 0, CID, role).checkpoint().seed,
            unhex(seed)
        );
    }
    let p = profile(7, 1000);
    let values = Arc::new(Mutex::new(Vec::new()));
    let out = values.clone();
    let (ok, _) = run("rng_registry", p.clone(), move |d| {
        calibrate(d, &p, T, T + 100_000);
        d.grant(Grant { id: 3, tick_us: T }, Input::None, |d| {
            d.spawn_session(CID, STOP, move |ctx| async move {
                for _ in 0..3 {
                    out.lock()
                        .unwrap()
                        .push(ctx.child("voter").try_next_u32().unwrap());
                }
                out.lock().unwrap().push(
                    ctx.child("voter")
                        .with_attribute("epoch", 5)
                        .try_next_u32()
                        .unwrap(),
                );
            })
        })?;
        Ok(())
    });
    assert!(ok);
    assert_eq!(
        *values.lock().unwrap(),
        vec![0x03815b53, 0xee086c42, 0xe997babc, 0x62e3cb46]
    );
}

#[test]
fn rng_restore_validates_role_and_reproduces_continuation() {
    let p = profile(7, 1000);
    let role = RoleRng::root_role();
    let saved = Arc::new(Mutex::new(None));
    let stash = saved.clone();
    let (ok, _) = run("rng_checkpoint", p.clone(), |d| {
        calibrate(d, &p, T, T + 100_000);
        d.grant(Grant { id: 3, tick_us: T }, Input::None, |d| {
            d.spawn_session(CID, STOP, |mut ctx| async move {
                for _ in 0..3 {
                    ctx.try_next_u32().unwrap();
                }
            })
        })?;
        d.grant(Grant { id: 4, tick_us: T }, Input::None, |d| {
            *stash.lock().unwrap() = Some(d.rng_checkpoint(CID, role)?);
            Ok(())
        })?;
        Ok(())
    });
    assert!(ok);
    let cp = saved.lock().unwrap().take().unwrap();
    assert_eq!(cp.word_position_lo, 3);
    let values = Arc::new(Mutex::new(Vec::new()));
    let out = values.clone();
    let (ok, _) = run("rng_restored", p.clone(), move |d| {
        calibrate(d, &p, T, T + 100_000);
        d.grant(Grant { id: 3, tick_us: T }, Input::None, |d| {
            assert!(
                d.restore_rng(CID, RoleRng::child_role(role, "other"), cp.clone())
                    .is_err()
            );
            d.restore_rng(CID, role, cp.clone())?;
            assert!(d.restore_rng(CID, role, cp.clone()).is_err());
            d.spawn_session(CID, STOP, move |mut ctx| async move {
                out.lock().unwrap().push(ctx.try_next_u32().unwrap());
            })
        })?;
        Ok(())
    });
    assert!(ok);
    assert_eq!(*values.lock().unwrap(), vec![0x7f5f5a03]);
}

#[test]
fn rng_exhaustion_is_atomic_and_infallible_adapter_fails() {
    let limit = 1u128 << 68;
    assert!(RoleRng::restore(checkpoint([0; 32], 0, limit)).is_err());
    let mut r = RoleRng::restore(checkpoint([0; 32], 0, limit - 2)).unwrap();
    assert!(r.try_next_u64().is_err());
    assert_eq!(r.checkpoint(), checkpoint([0; 32], 0, limit - 2));
    r.try_next_u32().unwrap();
    assert_eq!(r.checkpoint(), checkpoint([0; 32], 0, limit - 1));
    let mut buf = [0xaa; 1];
    assert!(r.try_fill_bytes(&mut buf).is_err());
    assert_eq!(buf, [0xaa]);
    r.try_fill_bytes(&mut []).unwrap();
    assert_eq!(r.checkpoint(), checkpoint([0; 32], 0, limit - 1));
    let p = profile(7, 1000);
    let role = RoleRng::root_role();
    let seed = unhex("6b906de1ead6e62491ccf447cf1d77e95f29e14f0b12593a7dbe3852b3a4f282");
    let (ok, events) = run("rng_exhaustion", p.clone(), move |d| {
        calibrate(d, &p, T, T + 100_000);
        let r = d.grant(Grant { id: 3, tick_us: T }, Input::None, |d| {
            d.restore_rng(CID, role, checkpoint(seed, 0, limit - 1))?;
            d.spawn_session(CID, STOP, |mut ctx| async move {
                ctx.try_next_u32().unwrap();
                panic!("draw unexpectedly returned")
            })
        });
        assert!(r.is_err());
        Ok(())
    });
    assert!(!ok);
    no_ack(&events, 3);
    assert!(!events.iter().any(|e| matches!(e, Event::RngDraw { .. })));
}

fn replay(seed: u64, pause: bool, host_budget: bool) -> Value {
    let path = dir("replay");
    let p = profile(seed, 1000);
    let outputs = Arc::new(Mutex::new(Vec::<u32>::new()));
    let values = outputs.clone();
    let recorded = Arc::new(Mutex::new(Vec::new()));
    let trace = recorded.clone();
    let outcome = ControlledDriver::run_async(config(&path), p.clone(), async move |d| {
        calibrate(d, &p, T, T + 100_000);
        if host_budget {
            for _ in 0..256 {
                tokio::task::consume_budget().await;
            }
        }
        if pause {
            std::thread::sleep(Duration::from_millis(2));
        }
        let first = d
            .grant(Grant { id: 3, tick_us: T }, Input::None, |d| {
                d.spawn_session(CID, STOP, move |ctx| async move {
                    for label in ["alpha", "beta"] {
                        let out = values.clone();
                        drop(
                            ctx.child(label)
                                .dedicated()
                                .spawn(move |mut ctx| async move {
                                    if pause {
                                        std::thread::sleep(Duration::from_millis(2));
                                    }
                                    // A ready Tokio receive stream crosses the default
                                    // cooperative budget inside ONE logical native poll.
                                    // Without unconstrained polling it would self-yield
                                    // according to the host budget, not our task schedule.
                                    let (tx, mut rx) = tokio::sync::mpsc::channel(300);
                                    for n in 0..300u32 {
                                        tx.try_send(n).unwrap();
                                    }
                                    drop(tx);
                                    let mut sum = 0;
                                    while let Some(n) = rx.recv().await {
                                        sum += n;
                                    }
                                    assert_eq!(sum, 44_850);
                                    out.lock().unwrap().push(ctx.try_next_u32().unwrap());
                                    WakeTimes {
                                        polls: Arc::new(AtomicUsize::new(0)),
                                        target: 2,
                                        copies: 100,
                                    }
                                    .await;
                                    ctx.sleep(Duration::from_micros(50)).await;
                                    out.lock().unwrap().push(ctx.try_next_u32().unwrap());
                                }),
                        );
                    }
                    pending::<()>().await;
                })
            })
            .unwrap();
        let mut ts = timers(&first);
        assert_eq!(ts.len(), 2);
        if seed % 2 == 1 {
            ts.reverse();
        }
        for (i, timer) in ts.into_iter().enumerate() {
            if pause {
                std::thread::sleep(Duration::from_millis(2));
            }
            step(d, 4 + i as u64, T + 50, Input::Timer(timer));
        }
        d.shutdown()?;
        *trace.lock().unwrap() = d.events();
        Ok(())
    });
    assert!(outcome.is_ok());
    let events = recorded.lock().unwrap().clone();
    audit(&events);
    let value = json!({"seed":seed,"events":events,"draws":*outputs.lock().unwrap()});
    fs::write(
        path.join("canonical-trace.json"),
        serde_json::to_vec_pretty(&value).unwrap(),
    )
    .unwrap();
    value
}

#[test]
fn approved_seeds_wall_pause_and_host_budget_have_identical_complete_traces() {
    for seed in [7, 42, 99] {
        let a = replay(seed, false, false);
        let b = replay(seed, true, true);
        assert_eq!(
            a, b,
            "seed{seed} trace changed under wall pause/host budget"
        );
    }
}
