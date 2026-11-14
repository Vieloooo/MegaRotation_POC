#![allow(dead_code)]
use commonware_runtime::tokio::Config;
use poc_native_runtime::{ClockSample, ControlledDriver, Error, Grant, Input, Profile, Step};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

pub const CID: [u8; 32] = [0x33; 32];
pub const STOP: i64 = 2_000_000;
pub const START: u64 = 1_000_000;
static ID: AtomicU64 = AtomicU64::new(0);

pub struct TestDir(PathBuf);
impl TestDir {
    pub fn new() -> Self {
        let counter = ID.fetch_add(1, Ordering::Relaxed);
        let path =
            std::env::temp_dir().join(format!("i1b-runtime-{}-{counter}", std::process::id()));
        fs::create_dir(&path).unwrap();
        Self(path)
    }

    pub fn config(&self) -> Config {
        Config::default().with_storage_directory(&self.0)
    }
}
impl Drop for TestDir {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

pub fn profile() -> Profile {
    Profile {
        run_id: [0x11; 32],
        config_hash: [0x22; 32],
        run_seed: 7,
        node: 0,
        incarnation: 1,
        epsilon_us: 1000,
    }
}

pub fn sample(profile: &Profile, request: u64, sequence: u64, as_of: u64, end: u64) -> ClockSample {
    ClockSample {
        run_id: profile.run_id,
        node: profile.node,
        incarnation: profile.incarnation,
        clock_domain: profile.clock_domain(),
        rate_profile: Profile::rate_profile(),
        request_id: request,
        sequence,
        as_of_tick_us: as_of,
        node_time_at_as_of_us: i64::try_from(as_of).unwrap() + profile.epsilon_us,
        uncertainty_us: profile.epsilon_us.unsigned_abs(),
        lease_end_tick_us: end,
    }
}

pub fn idle(_: &mut ControlledDriver) -> Result<(), Error> {
    Ok(())
}

pub fn apply(driver: &mut ControlledDriver, id: u64, tick_us: u64, input: Input) -> Step {
    driver.grant(Grant { id, tick_us }, input, idle).unwrap()
}

pub fn calibrate(driver: &mut ControlledDriver, profile: &Profile, end: u64) {
    apply(driver, 1, START, Input::RequestSample);
    apply(
        driver,
        2,
        START,
        Input::DeliverSample(sample(profile, 1, 1, START, end)),
    );
}
