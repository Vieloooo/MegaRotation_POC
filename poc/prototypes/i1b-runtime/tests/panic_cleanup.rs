mod support;

use commonware_runtime::{Spawner as _, Supervisor as _};
use poc_native_runtime::{ControlledDriver, Event, Grant, Input};
use std::{
    future::{pending, poll_fn},
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    task::Poll,
};
use support::*;

struct DropProbe(Arc<AtomicUsize>, bool);

impl Drop for DropProbe {
    fn drop(&mut self) {
        self.0.fetch_add(1, Ordering::SeqCst);
        assert!(!self.1, "drop probe panic");
    }
}

fn assert_failed(events: &[Event], grant: u64) {
    assert!(
        events
            .iter()
            .any(|event| matches!(event, Event::Failure { .. }))
    );
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::Ack { grant: id } if *id == grant))
    );
    assert!(matches!(events.last(), Some(Event::CleanupComplete)));
}

#[test]
fn factory_and_poll_panics_fence_ignored_handles_and_remaining_jobs() {
    for panic_in_constructor in [true, false] {
        let dir = TestDir::new();
        let profile = profile();
        let later_ran = Arc::new(AtomicUsize::new(0));
        let probe = later_ran.clone();
        let result = ControlledDriver::run(dir.config(), profile.clone(), move |driver| {
            calibrate(driver, &profile, START + 500);
            let result = driver.grant(
                Grant {
                    id: 3,
                    tick_us: START,
                },
                Input::None,
                |driver| {
                    driver.spawn_session(CID, STOP, move |context| async move {
                        drop(context.child("panics").spawn(move |_| {
                            assert!(!panic_in_constructor, "constructor panic");
                            async {
                                panic!("poll panic");
                            }
                        }));
                        drop(context.child("later").spawn(move |context| async move {
                            assert!(context.publication_allowed());
                            probe.fetch_add(1, Ordering::SeqCst);
                        }));
                        pending::<()>().await;
                    })
                },
            );
            assert!(result.is_err());
            assert_failed(&driver.events(), 3);
            assert!(
                !driver
                    .events()
                    .iter()
                    .any(|e| matches!(e, Event::Construct { task: 3 }))
            );
            Ok(())
        });
        assert!(result.is_err());
        assert_eq!(later_ran.load(Ordering::SeqCst), 0);
    }
}

#[test]
fn abort_after_poll_observes_panicking_pending_future_destruction() {
    let dir = TestDir::new();
    let profile = profile();
    let count = Arc::new(AtomicUsize::new(0));
    let probe = count.clone();
    let result = ControlledDriver::run(dir.config(), profile.clone(), move |driver| {
        calibrate(driver, &profile, START + 500);
        assert!(
            driver
                .grant(
                    Grant {
                        id: 3,
                        tick_us: START
                    },
                    Input::None,
                    |driver| {
                        driver.spawn_session(CID, STOP, move |context| async move {
                            let (awake, signal) = tokio::sync::oneshot::channel();
                            let handle = context.child("pending").spawn(move |_| async move {
                                let bomb = DropProbe(probe, true);
                                awake.send(()).unwrap();
                                pending::<()>().await;
                                drop(bomb);
                            });
                            signal.await.unwrap();
                            handle.abort();
                            pending::<()>().await;
                        })
                    }
                )
                .is_err()
        );
        assert_failed(&driver.events(), 3);
        Ok(())
    });
    assert!(result.is_err());
    assert_eq!(count.load(Ordering::SeqCst), 1);
}

#[test]
fn drop_bomb_output_is_observed_with_missing_or_retained_receiver() {
    for retain in [false, true] {
        let dir = TestDir::new();
        let profile = profile();
        let count = Arc::new(AtomicUsize::new(0));
        let probe = count.clone();
        let result = ControlledDriver::run(dir.config(), profile.clone(), move |driver| {
            calibrate(driver, &profile, START + 500);
            assert!(
                driver
                    .grant(
                        Grant {
                            id: 3,
                            tick_us: START
                        },
                        Input::None,
                        |driver| {
                            driver.spawn_session(CID, STOP, move |context| async move {
                                let handle = context
                                    .child("produces")
                                    .spawn(move |_| async move { DropProbe(probe, true) });
                                if retain {
                                    drop(handle.await.unwrap());
                                } else {
                                    drop(handle);
                                }
                                pending::<()>().await;
                            })
                        }
                    )
                    .is_err()
            );
            let events = driver.events();
            assert_failed(&events, 3);
            let producer_disposed = events
                .iter()
                .position(|event| matches!(event, Event::Dispose { task: 2, .. }))
                .unwrap();
            let failure = events
                .iter()
                .position(|event| matches!(event, Event::Failure { .. }))
                .unwrap();
            assert!(producer_disposed < failure);
            if retain {
                assert_eq!(
                    events
                        .iter()
                        .filter(|event| matches!(event, Event::Poll { task: 1, .. }))
                        .count(),
                    2
                );
            }
            Ok(())
        });
        assert!(result.is_err());
        assert_eq!(count.load(Ordering::SeqCst), 1);
    }
}

#[test]
fn native_parent_completion_aborts_never_polled_descendant_and_drains_capture() {
    let dir = TestDir::new();
    let profile = profile();
    let count = Arc::new(AtomicUsize::new(0));
    let probe = count.clone();
    ControlledDriver::run(dir.config(), profile.clone(), move |driver| {
        calibrate(driver, &profile, START + 500);
        let step = driver
            .grant(
                Grant {
                    id: 3,
                    tick_us: START,
                },
                Input::None,
                |driver| {
                    driver.spawn_session(CID, STOP, move |context| async move {
                        let capture = DropProbe(probe, false);
                        drop(context.child("never_called").spawn(move |_| {
                            assert!(capture.1, "cancelled constructor");
                            async move {
                                drop(capture);
                            }
                        }));
                    })
                },
            )
            .unwrap();
        assert!(step.events.iter().any(|event| matches!(
            event,
            Event::TaskRegistered {
                task: 2,
                parent: 1,
                ..
            }
        )));
        assert!(
            !step
                .events
                .iter()
                .any(|event| matches!(event, Event::Construct { task: 2 }))
        );
        let disposed = step
            .events
            .iter()
            .position(|event| matches!(event, Event::Dispose { task: 2, .. }))
            .unwrap();
        let ack = step
            .events
            .iter()
            .position(|event| matches!(event, Event::Ack { grant: 3 }))
            .unwrap();
        assert!(disposed < ack);
        Ok(())
    })
    .unwrap();
    assert_eq!(count.load(Ordering::SeqCst), 1);
}

#[test]
fn transport_closure_keeps_custody_and_catches_explicit_disposal_panic() {
    let dir = TestDir::new();
    let profile = profile();
    let count = Arc::new(AtomicUsize::new(0));
    let probe = count.clone();
    let result = ControlledDriver::run_async(dir.config(), profile.clone(), async move |driver| {
        calibrate(driver, &profile, START + 500);
        driver.grant(
            Grant {
                id: 3,
                tick_us: START,
            },
            Input::None,
            |driver| {
                driver.spawn_session(CID, STOP, move |context| async move {
                    let bomb = DropProbe(probe, true);
                    drop(context.child("waiting").spawn(|_| pending::<()>()));
                    pending::<()>().await;
                    drop(bomb);
                })
            },
        )?;
        // This pump owns only a transport sender, never the controller or a job.
        let (tx, rx) = tokio::sync::oneshot::channel::<()>();
        let pump = tokio::spawn(async move {
            pending::<()>().await;
            drop(tx);
        });
        pump.abort();
        assert!(rx.await.is_err());
        assert!(driver.fail_transport().is_err());
        let events = driver.events();
        assert!(events.iter().any(|e| matches!(
            e,
            Event::Failure {
                reason: "native disposal unwind"
            }
        )));
        assert!(events.iter().any(|e| matches!(
            e,
            Event::Dispose {
                task: 2,
                cleanup: true,
                ..
            }
        )));
        assert!(matches!(events.last(), Some(Event::CleanupComplete)));
        assert!(
            driver
                .grant(
                    Grant {
                        id: 4,
                        tick_us: START + 1
                    },
                    Input::None,
                    idle
                )
                .is_err()
        );
        Ok(())
    });
    assert!(result.is_err());
    assert_eq!(count.load(Ordering::SeqCst), 1);
}

#[test]
fn driver_panic_and_poll_bound_failure_reclaim_jobs_without_a_successful_step() {
    for driver_panic in [false, true] {
        let dir = TestDir::new();
        let profile = profile();
        let count = Arc::new(AtomicUsize::new(0));
        let probe = count.clone();
        let result = ControlledDriver::run(dir.config(), profile.clone(), move |driver| {
            driver.with_limits(3, 100);
            calibrate(driver, &profile, START + 500);
            if driver_panic {
                driver
                    .grant(
                        Grant {
                            id: 3,
                            tick_us: START,
                        },
                        Input::None,
                        |driver| {
                            let capture = DropProbe(probe, false);
                            driver.spawn_session(
                                CID,
                                STOP,
                                |_: poc_native_runtime::SimContext| async move {
                                    drop(capture);
                                },
                            )?;
                            panic!("driver action failed after enrollment");
                        },
                    )
                    .unwrap_err();
                assert!(
                    !driver
                        .events()
                        .iter()
                        .any(|e| matches!(e, Event::Construct { task: 1 }))
                );
            } else {
                driver
                    .grant(
                        Grant {
                            id: 3,
                            tick_us: START,
                        },
                        Input::None,
                        |driver| {
                            driver.spawn_session(CID, STOP, move |_| async move {
                                let capture = DropProbe(probe, false);
                                poll_fn(|cx| {
                                    cx.waker().wake_by_ref();
                                    Poll::Pending::<()>
                                })
                                .await;
                                drop(capture);
                            })
                        },
                    )
                    .unwrap_err();
                assert_eq!(
                    driver
                        .events()
                        .iter()
                        .filter(|e| matches!(e, Event::Poll { grant: 3, .. }))
                        .count(),
                    3
                );
            }
            assert_failed(&driver.events(), 3);
            Ok(())
        });
        assert!(result.is_err());
        assert_eq!(count.load(Ordering::SeqCst), 1);
    }
}

#[test]
fn normal_shutdown_disposes_waits_and_bound_exceeded_reports_failure() {
    for disposal_limit in [0, 10] {
        let dir = TestDir::new();
        let profile = profile();
        let count = Arc::new(AtomicUsize::new(0));
        let probe = count.clone();
        let result = ControlledDriver::run(dir.config(), profile.clone(), move |driver| {
            driver.with_limits(100, disposal_limit);
            calibrate(driver, &profile, START + 500);
            driver.grant(
                Grant {
                    id: 3,
                    tick_us: START,
                },
                Input::None,
                |driver| {
                    driver.spawn_session(CID, STOP, move |_| async move {
                        let probe = DropProbe(probe, false);
                        pending::<()>().await;
                        drop(probe);
                    })
                },
            )?;
            let shutdown = driver.shutdown();
            let events = driver.events();
            if disposal_limit == 0 {
                assert!(shutdown.is_err());
                assert!(events.contains(&Event::CleanupBoundExceeded));
                assert!(!events.contains(&Event::CleanupComplete));
            } else {
                shutdown.unwrap();
                assert!(events.contains(&Event::CleanupComplete));
            }
            Ok(())
        });
        assert_eq!(result.is_ok(), disposal_limit > 0);
        assert_eq!(count.load(Ordering::SeqCst), 1);
    }
}

#[test]
fn wake_without_a_grant_is_failure_and_never_polls_protocol_work() {
    let dir = TestDir::new();
    let profile = profile();
    let out = Arc::new(std::sync::Mutex::new(None));
    let saved = out.clone();
    let result = ControlledDriver::run(dir.config(), profile.clone(), move |driver| {
        calibrate(driver, &profile, START + 500);
        driver.grant(
            Grant {
                id: 3,
                tick_us: START,
            },
            Input::None,
            |driver| {
                driver.spawn_session(CID, STOP, move |_| async move {
                    poll_fn(|cx| {
                        *saved.lock().unwrap() = Some(cx.waker().clone());
                        Poll::Pending::<()>
                    })
                    .await;
                })
            },
        )?;
        out.lock().unwrap().take().unwrap().wake();
        assert!(
            driver
                .grant(
                    Grant {
                        id: 4,
                        tick_us: START + 1
                    },
                    Input::None,
                    idle
                )
                .is_err()
        );
        assert_failed(&driver.events(), 4);
        assert!(
            !driver
                .events()
                .iter()
                .any(|e| matches!(e, Event::Grant { grant: 4, .. }))
        );
        Ok(())
    });
    assert!(result.is_err());
}

#[test]
fn reentrant_grant_and_shutdown_latch_failure_then_drain_outside_the_permit() {
    for nested_grant in [true, false] {
        let dir = TestDir::new();
        let profile = profile();
        let count = Arc::new(AtomicUsize::new(0));
        let probe = count.clone();
        let result = ControlledDriver::run(dir.config(), profile.clone(), move |driver| {
            calibrate(driver, &profile, START + 500);
            driver.grant(
                Grant {
                    id: 3,
                    tick_us: START,
                },
                Input::None,
                |driver| {
                    driver.spawn_session(CID, STOP, move |_| async move {
                        let captured = DropProbe(probe, false);
                        pending::<()>().await;
                        drop(captured);
                    })
                },
            )?;
            let failure = driver.grant(
                Grant {
                    id: 4,
                    tick_us: START + 1,
                },
                Input::None,
                |driver| {
                    if nested_grant {
                        let nested = driver.grant(
                            Grant {
                                id: 5,
                                tick_us: START + 5,
                            },
                            Input::None,
                            |_| panic!("nested callback ran"),
                        );
                        assert!(nested.is_err());
                    } else {
                        assert!(driver.shutdown().is_err());
                    }
                    assert_eq!(count.load(Ordering::SeqCst), 0);
                    Ok(())
                },
            );
            assert!(failure.is_err());
            let events = driver.events();
            assert_failed(&events, 4);
            assert!(
                !events
                    .iter()
                    .any(|e| matches!(e, Event::Grant { grant: 5, .. }))
            );
            assert_eq!(count.load(Ordering::SeqCst), 1);
            Ok(())
        });
        assert!(result.is_err());
    }
}

#[test]
fn repeated_nested_grant_errors_defer_cleanup_until_outer_permit_ends() {
    assert_failed_reentry(|driver| {
        assert!(
            driver
                .grant(
                    Grant {
                        id: 5,
                        tick_us: START + 5
                    },
                    Input::None,
                    |_| panic!("first nested callback ran"),
                )
                .is_err()
        );
    });
}

#[test]
fn transport_failure_then_nested_grant_defers_cleanup_until_outer_permit_ends() {
    assert_failed_reentry(|driver| assert!(driver.fail_transport().is_err()));
}

fn assert_failed_reentry(first_failure: impl FnOnce(&mut ControlledDriver)) {
    let dir = TestDir::new();
    let profile = profile();
    let dropped = Arc::new(AtomicUsize::new(0));
    let capture = DropProbe(dropped.clone(), false);
    let mut completed_callback = false;
    let mut observed_cleanup = false;
    let result = ControlledDriver::run(dir.config(), profile.clone(), |driver| {
        calibrate(driver, &profile, START + 500);
        driver.grant(
            Grant {
                id: 3,
                tick_us: START,
            },
            Input::None,
            |driver| {
                driver.spawn_session(CID, STOP, |_| async move {
                    pending::<()>().await;
                    drop(capture);
                })
            },
        )?;
        let failed = driver.grant(
            Grant {
                id: 4,
                tick_us: START + 1,
            },
            Input::None,
            |driver| {
                first_failure(driver);
                assert!(
                    driver
                        .grant(
                            Grant {
                                id: 6,
                                tick_us: START + 6
                            },
                            Input::None,
                            |_| panic!("nested callback ran after failure"),
                        )
                        .is_err()
                );
                assert_eq!(dropped.load(Ordering::SeqCst), 0);
                completed_callback = true;
                Ok(())
            },
        );
        assert!(failed.is_err());
        let events = driver.events();
        assert_failed(&events, 4);
        assert!(
            !events
                .iter()
                .any(|event| matches!(event, Event::Grant { grant: 5 | 6, .. }))
        );
        assert!(events.iter().any(|event| matches!(
            event,
            Event::Dispose {
                task: 1,
                grant: 4,
                cleanup: true,
                ..
            }
        )));
        assert_eq!(dropped.load(Ordering::SeqCst), 1);
        observed_cleanup = true;
        Ok(())
    });
    assert!(result.is_err());
    assert!(completed_callback && observed_cleanup);
    assert_eq!(dropped.load(Ordering::SeqCst), 1);
}
