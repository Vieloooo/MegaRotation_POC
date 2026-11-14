use commonware_runtime::{
    Runner as _,
    tokio::{Config, Runner},
};
use std::{
    pin::pin,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    task::{Context, Poll, Waker},
};

#[test]
fn abort_before_first_poll_drops_captures_without_invoking_factory() {
    let path = support::unique_temp_dir("i1b-native-handoff");
    let outcome = Runner::new(
        Config::default()
            .with_storage_directory(&path)
            .with_catch_panics(true),
    )
    .start(|native| async move {
        let called = Arc::new(AtomicBool::new(false));
        let dropped = Arc::new(AtomicBool::new(false));
        let capture = MarkDrop(dropped.clone());
        let sentinel = called.clone();
        let flag = Arc::new(AtomicBool::new(false));
        let mut submitted = None;
        let handle = native.spawn_controlled(
            move |_| {
                sentinel.store(true, Ordering::SeqCst);
                async move {
                    drop(capture);
                }
            },
            flag.clone(),
            |job| {
                submitted = Some(job);
            },
        );
        let before = called.load(Ordering::SeqCst);
        handle.abort();
        let mut job = pin!(tokio::task::unconstrained(
            submitted.expect("registered job")
        ));
        let status = job.as_mut().poll(&mut Context::from_waker(Waker::noop()));
        (
            before,
            status,
            called.load(Ordering::SeqCst),
            dropped.load(Ordering::SeqCst),
            flag.load(Ordering::SeqCst),
        )
    });
    std::fs::remove_dir_all(path).unwrap();
    assert_eq!(outcome, (false, Poll::Ready(()), false, true, false));
}

struct MarkDrop(Arc<AtomicBool>);
impl Drop for MarkDrop {
    fn drop(&mut self) {
        self.0.store(true, Ordering::SeqCst);
    }
}
mod support;
