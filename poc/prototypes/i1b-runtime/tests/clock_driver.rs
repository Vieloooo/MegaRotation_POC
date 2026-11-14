mod support;

use commonware_runtime::{Clock, Metrics, Spawner, Supervisor, telemetry::metrics::raw::Counter};
use poc_native_runtime::{ControlledDriver, Event, Grant, Input, SimContext, Step, Timer};
use rand_core::CryptoRng;
use std::{
    future::{pending, poll_fn},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    task::Poll,
    time::{Duration, UNIX_EPOCH},
};
use support::*;

fn arm(step: &Step) -> Timer {
    step.events
        .iter()
        .find_map(|event| match event {
            Event::TimerArmed { timer } => Some(timer.clone()),
            _ => None,
        })
        .expect("timer registration")
}

fn at(local_us: u64) -> std::time::SystemTime {
    UNIX_EPOCH + Duration::from_micros(local_us)
}

fn native_bounds<T: Clock + CryptoRng + Metrics + Spawner>(_: &T) {}

#[test]
fn clock_and_governor_share_historical_projection_and_absolute_timer_deadline() {
    for epsilon in [1000, -1000] {
        let dir = TestDir::new();
        let mut profile = profile();
        profile.epsilon_us = epsilon;
        let result = Arc::new(Mutex::new(None));
        let probe = result.clone();
        ControlledDriver::run(dir.config(), profile.clone(), move |driver| {
            apply(driver, 1, START, Input::RequestSample);
            let delivered = sample(&profile, 1, 1, START, START + 100_000);
            let expected = u64::try_from(1_010_000i64 + epsilon).unwrap();
            let step = driver.grant(
                Grant {
                    id: 2,
                    tick_us: START + 10_000,
                },
                Input::DeliverSample(delivered),
                |driver| {
                    assert_eq!(driver.manager_time_us()?, expected as i64);
                    driver.spawn_session(CID, STOP, move |context| async move {
                        native_bounds(&context);
                        assert_eq!(context.current(), at(expected));
                        assert_eq!(governor::clock::Clock::now(&context), context.current());
                        assert!(context.publication_allowed());
                        let named = context.child("voter").with_attribute("epoch", 5);
                        assert!(named.name().label.ends_with("session_voter"));
                        assert_eq!(
                            named.name().attributes,
                            vec![("epoch".to_owned(), "5".to_owned())]
                        );
                        let counter: Counter = Counter::default();
                        let metric = named.register("votes", "Count votes", counter);
                        metric.inc();
                        assert!(named.encode().contains("votes_total{epoch=\"5\"} 1"));
                        context.sleep(Duration::from_nanos(1)).await;
                        *probe.lock().unwrap() = Some(context.current());
                    })
                },
            )?;
            let timer = arm(&step);
            assert_eq!(timer.deadline_us, expected as i64 + 1);
            let early = apply(driver, 3, START + 10_000, Input::Timer(timer.clone()));
            assert!(
                early
                    .events
                    .iter()
                    .any(|e| matches!(e, Event::TimerSuppressed { .. }))
            );
            let no_timer = apply(driver, 4, START + 10_001, Input::None);
            assert!(
                !no_timer
                    .events
                    .iter()
                    .any(|e| matches!(e, Event::Poll { .. }))
            );
            apply(driver, 5, START + 10_002, Input::RequestSample);
            let renewal = sample(&profile, 2, 2, START + 10_002, START + 110_002);
            apply(driver, 6, START + 10_003, Input::DeliverSample(renewal));
            assert!(result.lock().unwrap().is_none());
            let fired = apply(driver, 7, START + 10_004, Input::Timer(timer.clone()));
            assert!(
                fired
                    .events
                    .iter()
                    .any(|e| matches!(e, Event::TimerFired { .. }))
            );
            let replay = apply(driver, 8, START + 10_005, Input::Timer(timer));
            assert!(
                replay
                    .events
                    .iter()
                    .any(|e| matches!(e, Event::TimerSuppressed { .. }))
            );
            assert_eq!(*result.lock().unwrap(), Some(at(expected + 4)));
            Ok(())
        })
        .unwrap();
    }
}

#[test]
fn invalid_sample_does_not_poison_request_and_lease_equality_retires_old_cid() {
    let dir = TestDir::new();
    let profile = profile();
    let old_ran = Arc::new(AtomicUsize::new(0));
    let old = old_ran.clone();
    ControlledDriver::run(dir.config(), profile.clone(), move |driver| {
        calibrate(driver, &profile, START + 10);
        let step = driver.grant(
            Grant {
                id: 3,
                tick_us: START,
            },
            Input::None,
            |driver| {
                driver.spawn_session(CID, STOP, move |context| async move {
                    context.sleep(Duration::from_micros(10)).await;
                    old.fetch_add(1, Ordering::SeqCst);
                })
            },
        )?;
        let timer = arm(&step);
        apply(driver, 4, START + 8, Input::RequestSample);
        let mut invalid = sample(&profile, 2, 100, START + 8, START + 100);
        invalid.node_time_at_as_of_us += 1;
        let denied = apply(driver, 5, START + 9, Input::DeliverSample(invalid));
        assert!(
            denied
                .events
                .iter()
                .any(|e| matches!(e, Event::SampleRejected { request: 2, .. }))
        );
        let expiry = apply(driver, 6, START + 10, Input::Timer(timer));
        assert!(expiry.events.iter().any(|e| matches!(
            e,
            Event::Retired {
                reason: "ClockUnsafe",
                ..
            }
        )));
        assert!(
            expiry
                .events
                .iter()
                .any(|e| matches!(e, Event::TimerCancelled { .. }))
        );
        assert!(
            !expiry
                .events
                .iter()
                .any(|e| matches!(e, Event::Poll { .. }))
        );
        let valid = sample(&profile, 2, 2, START + 9, START + 100);
        let step = driver.grant(
            Grant {
                id: 7,
                tick_us: START + 11,
            },
            Input::DeliverSample(valid),
            |driver| {
                assert_eq!(driver.manager_time_us()?, 1_001_011);
                assert!(driver.spawn_session(CID, STOP, |_| async {}).is_err());
                driver.spawn_session([0x35; 32], STOP, |context| async move {
                    assert!(context.publication_allowed());
                })
            },
        )?;
        assert!(step.events.iter().any(|e| matches!(
            e,
            Event::SampleAccepted {
                request: 2,
                sequence: 2
            }
        )));
        Ok(())
    })
    .unwrap();
    assert_eq!(old_ran.load(Ordering::SeqCst), 0);
}

#[test]
fn local_stop_equality_blocks_poll_and_preserves_terminal_scope() {
    let dir = TestDir::new();
    let profile = profile();
    ControlledDriver::run(dir.config(), profile.clone(), move |driver| {
        calibrate(driver, &profile, START + 100);
        let step = driver.grant(
            Grant {
                id: 3,
                tick_us: START,
            },
            Input::None,
            |driver| {
                driver.spawn_session(CID, 1_001_001, |context| async move {
                    context.sleep(Duration::from_micros(1)).await;
                    panic!("task polled at local stop");
                })
            },
        )?;
        let stop = apply(driver, 4, START + 1, Input::Timer(arm(&step)));
        assert!(stop.events.iter().any(|e| matches!(
            e,
            Event::Retired {
                reason: "LocalStop",
                ..
            }
        )));
        assert!(!stop.events.iter().any(|e| matches!(e, Event::Poll { .. })));
        Ok(())
    })
    .unwrap();
}

#[test]
fn default_timeout_prefers_ready_result_and_preserves_cancelled_generation() {
    let dir = TestDir::new();
    let profile = profile();
    let is_ready = Arc::new(AtomicBool::new(false));
    let completed = Arc::new(AtomicUsize::new(0));
    let flag = is_ready.clone();
    let completed_job = completed.clone();
    ControlledDriver::run(dir.config(), profile.clone(), move |driver| {
        calibrate(driver, &profile, START + 100);
        let step = driver.grant(
            Grant {
                id: 3,
                tick_us: START,
            },
            Input::None,
            |driver| {
                driver.spawn_session(CID, STOP, move |context| async move {
                    let result = context
                        .timeout(
                            Duration::from_micros(5),
                            poll_fn(move |_| {
                                if flag.load(Ordering::SeqCst) {
                                    Poll::Ready(9)
                                } else {
                                    Poll::Pending
                                }
                            }),
                        )
                        .await;
                    assert_eq!(result.unwrap(), 9);
                    completed_job.fetch_add(1, Ordering::SeqCst);
                    let next = context.timeout(Duration::ZERO, pending::<()>()).await;
                    assert!(matches!(next, Err(commonware_runtime::Error::Timeout)));
                    completed_job.fetch_add(1, Ordering::SeqCst);
                })
            },
        )?;
        let timer = arm(&step);
        let tied = driver.grant(
            Grant {
                id: 4,
                tick_us: START + 5,
            },
            Input::Timer(timer.clone()),
            |_| {
                is_ready.store(true, Ordering::SeqCst);
                Ok(())
            },
        )?;
        assert!(
            tied.events
                .iter()
                .any(|e| matches!(e, Event::TimerCancelled { timer: value } if *value == timer))
        );
        let zero = arm(&tied);
        assert_ne!(zero.id, timer.id);
        assert_eq!(zero.deadline_us, 1_001_005);
        assert_eq!(completed.load(Ordering::SeqCst), 1);
        apply(driver, 5, START + 5, Input::Timer(zero));
        assert_eq!(completed.load(Ordering::SeqCst), 2);
        Ok(())
    })
    .unwrap();
}

#[test]
fn native_stop_preserves_first_value_and_waits_for_signal_holders() {
    let dir = TestDir::new();
    let profile = profile();
    let observed = Arc::new(AtomicUsize::new(0));
    let probe = observed.clone();
    ControlledDriver::run(dir.config(), profile.clone(), move |driver| {
        calibrate(driver, &profile, START + 100);
        let step = driver.grant(
            Grant {
                id: 3,
                tick_us: START,
            },
            Input::None,
            |driver| {
                driver.spawn_session(CID, STOP, move |context: SimContext| async move {
                    let mut open = context.stopped();
                    let stopper = context.child("stopper").spawn(|context| async move {
                        assert!(matches!(
                            context.stop(7, Some(Duration::from_micros(6))).await,
                            Err(commonware_runtime::Error::Timeout)
                        ));
                    });
                    let another = context.child("second").spawn(|context| async move {
                        context.stop(8, None).await.unwrap();
                    });
                    assert_eq!((&mut open).await, Ok(7));
                    assert_eq!(context.stopped().await, Ok(7));
                    stopper.await.unwrap();
                    drop(open);
                    another.await.unwrap();
                    probe.fetch_add(1, Ordering::SeqCst);
                })
            },
        )?;
        assert_eq!(observed.load(Ordering::SeqCst), 0);
        apply(driver, 4, START + 6, Input::Timer(arm(&step)));
        assert_eq!(observed.load(Ordering::SeqCst), 1);
        Ok(())
    })
    .unwrap();
}

#[test]
fn stable_wake_order_survives_wall_pauses_and_hints() {
    let execute =
        |pause| {
            let dir = TestDir::new();
            let profile = profile();
            ControlledDriver::run(dir.config(), profile.clone(), move |driver| {
                calibrate(driver, &profile, START + 100);
                let step =
                    driver.grant(
                        Grant {
                            id: 3,
                            tick_us: START,
                        },
                        Input::None,
                        |driver| {
                            driver.spawn_session(CID, STOP, move |context| async move {
                                let mut first = true;
                                poll_fn(|cx| {
                                    if first {
                                        first = false;
                                        cx.waker().wake_by_ref();
                                        cx.waker().wake_by_ref();
                                        if pause {
                                            std::thread::sleep(Duration::from_millis(2));
                                        }
                                        Poll::Pending
                                    } else {
                                        Poll::Ready(())
                                    }
                                })
                                .await;
                                let (tx, rx) = tokio::sync::oneshot::channel();
                                drop(context.child("cross").dedicated().spawn(
                                    move |_| async move {
                                        tx.send(()).unwrap();
                                    },
                                ));
                                rx.await.unwrap();
                            })
                        },
                    )?;
                assert_eq!(
                    step.events
                        .iter()
                        .filter(|e| matches!(e, Event::Poll { task: 1, .. }))
                        .count(),
                    3
                );
                assert!(step.events.iter().any(|e| matches!(
                    e,
                    Event::TaskRegistered {
                        task: 2,
                        placement: poc_native_runtime::Placement::Dedicated,
                        ..
                    }
                )));
                Ok(serde_json::to_vec(&driver.events()).unwrap())
            })
            .unwrap()
        };
    assert_eq!(execute(false), execute(true));
}

#[test]
fn same_cid_sibling_can_poll_transferred_sleep_with_original_deadline() {
    let dir = TestDir::new();
    let profile = profile();
    let observed = Arc::new(AtomicUsize::new(0));
    let probe = observed.clone();
    ControlledDriver::run(dir.config(), profile.clone(), move |driver| {
        calibrate(driver, &profile, START + 500);
        let start = driver.grant(
            Grant {
                id: 3,
                tick_us: START,
            },
            Input::None,
            |driver| {
                driver.spawn_session(CID, STOP, move |context| async move {
                    use std::future::Future;
                    let (tx, rx) = tokio::sync::oneshot::channel();
                    let producer = context.child("producer").spawn(|context| async move {
                        let mut sleep = Box::pin(context.sleep(Duration::from_micros(9)));
                        poll_fn(|cx| {
                            assert!(sleep.as_mut().poll(cx).is_pending());
                            Poll::Ready(())
                        })
                        .await;
                        assert!(tx.send(sleep).is_ok());
                    });
                    let consumer = context.child("consumer").spawn(|context| async move {
                        rx.await.unwrap().await;
                        assert_eq!(context.current(), at(1_001_009));
                        probe.fetch_add(1, Ordering::SeqCst);
                    });
                    producer.await.unwrap();
                    consumer.await.unwrap();
                })
            },
        )?;
        let timer = arm(&start);
        assert_eq!(timer.deadline_us, 1_001_009);
        assert!(start.events.contains(&Event::TimerTransferred {
            timer: timer.clone(),
            from: 2,
            to: 3
        }));
        let fire = apply(driver, 4, START + 9, Input::Timer(timer));
        assert!(fire.events.iter().any(|e| matches!(
            e,
            Event::Poll {
                task: 3,
                grant: 4,
                ..
            }
        )));
        assert!(!fire.events.iter().any(|e| matches!(
            e,
            Event::Poll {
                task: 2,
                grant: 4,
                ..
            }
        )));
        assert_eq!(observed.load(Ordering::SeqCst), 1);
        Ok(())
    })
    .unwrap();
}
