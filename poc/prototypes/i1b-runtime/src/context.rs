use crate::{
    Id, RoleRng,
    clock::{local_time, system_time},
    driver::{Event, Placement, Shared, TaskPhase},
};
use commonware_runtime::{
    Clock, Error, Handle, Metrics, Name, Spawner, Supervisor,
    signal::{Signal, Stopper},
    telemetry::metrics::{Metric, Registered},
    tokio::Context as NativeContext,
};
use rand_core::{TryCryptoRng, TryRng};
use std::{
    convert::Infallible,
    future::{Future, pending},
    pin::Pin,
    sync::{Arc, Mutex, Weak},
    task::{Context, Poll},
    time::{Duration, SystemTime},
};

pub struct SimContext {
    native: NativeContext,
    shared: Weak<Shared>,
    cid: Id,
    role: Id,
    stopper: Arc<Mutex<Stopper>>,
    placement: Placement,
    scope: u64,
}

impl SimContext {
    pub(crate) fn new(
        native: NativeContext,
        shared: Weak<Shared>,
        cid: Id,
        role: Id,
        stopper: Arc<Mutex<Stopper>>,
        scope: u64,
    ) -> Self {
        Self {
            native,
            shared,
            cid,
            role,
            stopper,
            placement: Placement::Shared,
            scope,
        }
    }

    fn runtime(&self) -> Arc<Shared> {
        self.shared.upgrade().expect("driver owns protocol jobs")
    }

    pub fn publication_allowed(&self) -> bool {
        self.shared
            .upgrade()
            .is_some_and(|shared| shared.can_publish(self.cid))
    }
}

impl Supervisor for SimContext {
    fn child(&self, label: &'static str) -> Self {
        self.runtime().ordinary(self.cid);
        Self::new(
            self.native.child(label),
            self.shared.clone(),
            self.cid,
            RoleRng::child_role(self.role, label),
            self.stopper.clone(),
            self.scope,
        )
    }

    fn with_attribute(mut self, key: &'static str, value: impl std::fmt::Display) -> Self {
        self.runtime().ordinary(self.cid);
        let value = value.to_string();
        self.role = RoleRng::attribute_role(self.role, key, &value);
        self.native = self.native.with_attribute(key, value);
        self
    }

    fn name(&self) -> Name {
        self.runtime().ordinary(self.cid);
        self.native.name()
    }
}

impl Metrics for SimContext {
    fn register<N: Into<String>, H: Into<String>, M: Metric>(
        &self,
        name: N,
        help: H,
        metric: M,
    ) -> Registered<M> {
        self.runtime().ordinary(self.cid);
        self.native.register(name, help, metric)
    }

    fn encode(&self) -> String {
        self.runtime().ordinary(self.cid);
        self.native.encode()
    }
}

impl Spawner for SimContext {
    fn dedicated(mut self) -> Self {
        self.runtime().ordinary(self.cid);
        self.placement = Placement::Dedicated;
        self
    }

    fn shared(mut self, blocking: bool) -> Self {
        self.runtime().ordinary(self.cid);
        self.placement = if blocking {
            Placement::Blocking
        } else {
            Placement::Shared
        };
        self
    }

    fn spawn<F, Fut, T>(self, f: F) -> Handle<T>
    where
        F: FnOnce(Self) -> Fut + Send + 'static,
        Fut: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        let shared = self.runtime();
        let id = shared.reserve_task(self.cid);
        let Self {
            native,
            shared: weak,
            cid,
            role,
            stopper,
            placement,
            scope,
        } = self;
        let phase = Arc::new(Mutex::new(TaskPhase::Enrolled));
        let job_phase = phase.clone();
        let handle = native.spawn_controlled(
            move |native| {
                let context = Self::new(native, weak, cid, role, stopper, id);
                let shared = context.runtime();
                shared.ordinary(cid);
                *job_phase.lock().unwrap() = TaskPhase::Started;
                shared.event(Event::Construct { task: id });
                let future = f(context);
                async move {
                    let result = future.await;
                    *job_phase.lock().unwrap() = TaskPhase::Completed;
                    result
                }
            },
            shared.failure.clone(),
            |native| shared.enroll(id, scope, cid, role, placement, phase, native),
        );
        handle
    }

    async fn stop(self, value: i32, timeout: Option<Duration>) -> Result<(), Error> {
        self.runtime().ordinary(self.cid);
        let completion = self.stopper.lock().unwrap().stop(value);
        let timeout = async {
            match timeout {
                Some(duration) => self.sleep(duration).await,
                None => pending().await,
            }
        };
        tokio::select! {
            biased;
            result = completion => result.map_err(|_| Error::Closed),
            _ = timeout => Err(Error::Timeout),
        }
    }

    fn stopped(&self) -> Signal {
        self.runtime().ordinary(self.cid);
        self.stopper.lock().unwrap().stopped()
    }
}

impl governor::clock::Clock for SimContext {
    type Instant = SystemTime;
    fn now(&self) -> SystemTime {
        self.current()
    }
}

impl governor::clock::ReasonablyRealtime for SimContext {}

impl Clock for SimContext {
    fn current(&self) -> SystemTime {
        system_time(self.runtime().local(self.cid))
    }

    fn sleep(&self, duration: Duration) -> impl Future<Output = ()> + Send + 'static + use<> {
        self.sleep_until(
            self.current()
                .checked_add(duration)
                .expect("sleep deadline overflow"),
        )
    }

    fn sleep_until(
        &self,
        deadline: SystemTime,
    ) -> impl Future<Output = ()> + Send + 'static + use<> {
        self.runtime().ordinary(self.cid);
        Sleep {
            shared: self.shared.clone(),
            cid: self.cid,
            deadline_us: local_time(deadline),
            id: None,
        }
    }
}

impl TryRng for SimContext {
    type Error = Infallible;

    fn try_next_u32(&mut self) -> Result<u32, Infallible> {
        Ok(self
            .runtime()
            .draw(self.cid, self.role, 1, RoleRng::try_next_u32))
    }

    fn try_next_u64(&mut self) -> Result<u64, Infallible> {
        Ok(self
            .runtime()
            .draw(self.cid, self.role, 2, RoleRng::try_next_u64))
    }

    fn try_fill_bytes(&mut self, bytes: &mut [u8]) -> Result<(), Infallible> {
        let words = u64::try_from(bytes.len().div_ceil(4)).expect("RNG request too large");
        self.runtime()
            .draw(self.cid, self.role, words, |rng| rng.try_fill_bytes(bytes));
        Ok(())
    }
}
impl TryCryptoRng for SimContext {}

struct Sleep {
    shared: Weak<Shared>,
    cid: Id,
    deadline_us: i64,
    id: Option<u64>,
}

impl Future for Sleep {
    type Output = ();
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        let shared = self.shared.upgrade().expect("timer polled after driver");
        if let Some(id) = self.id {
            if shared.timer_polled(self.cid, id, cx.waker()) {
                self.id = None;
                return Poll::Ready(());
            }
        } else {
            self.id = Some(shared.arm(self.cid, self.deadline_us, cx.waker()));
        }
        Poll::Pending
    }
}

impl Drop for Sleep {
    fn drop(&mut self) {
        if let (Some(shared), Some(id)) = (self.shared.upgrade(), self.id) {
            shared.cancel_timer(id);
        }
    }
}
