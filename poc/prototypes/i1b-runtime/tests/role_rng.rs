mod support;

use commonware_runtime::Supervisor as _;
use poc_native_runtime::{ControlledDriver, Event, Grant, Input, RngCheckpoint, RoleRng};
use rand_core::TryRng;
use std::sync::{Arc, Mutex};
use support::*;

fn hex32(hex: &str) -> [u8; 32] {
    assert_eq!(hex.len(), 64);
    std::array::from_fn(|i| u8::from_str_radix(&hex[2 * i..2 * i + 2], 16).unwrap())
}

fn zero_at(stream: u64, offset: u64) -> RngCheckpoint {
    RngCheckpoint {
        seed: [0; 32],
        stream,
        word_position_hi: 0,
        word_position_lo: offset,
    }
}

#[test]
fn chacha20_literal_mixed_fill_and_seek_vectors() {
    let mut rng = RoleRng::restore(zero_at(0, 0)).unwrap();
    assert_eq!(rng.try_next_u32().unwrap(), 0xade0b876);
    assert_eq!(rng.checkpoint(), zero_at(0, 1));
    assert_eq!(rng.try_next_u64().unwrap(), 0xe56a5d40903df1a0);
    assert_eq!(rng.checkpoint(), zero_at(0, 3));
    let mut three = [0; 3];
    rng.try_fill_bytes(&mut three).unwrap();
    assert_eq!(three, [0x53, 0x86, 0xbd]);
    assert_eq!(rng.checkpoint(), zero_at(0, 4));
    assert_eq!(rng.try_next_u32().unwrap(), 0xb819d2bd);
    let mut five = [0; 5];
    rng.try_fill_bytes(&mut five).unwrap();
    assert_eq!(five, [0xa0, 0x8d, 0xed, 0x1a, 0xa8]);
    assert_eq!(rng.checkpoint(), zero_at(0, 7));
    rng.try_fill_bytes(&mut []).unwrap();
    assert_eq!(rng.checkpoint(), zero_at(0, 7));
    assert_eq!(rng.try_next_u64().unwrap(), 0x7c5941dac70d778b);
    assert_eq!(rng.checkpoint(), zero_at(0, 9));
    let mut restored = RoleRng::restore(rng.checkpoint()).unwrap();
    assert_eq!(restored.try_next_u64().unwrap(), 0x3fe024778d485751);
    assert_eq!(restored.checkpoint(), zero_at(0, 11));

    for (stream, initial, expected) in [
        (0, 15, 0xbee7079f8665eeb2),
        (0, 63, 0x7488a6e57e166731),
        (51, 15, 0x922704dfb05e7395),
        (51, 63, 0x9782fb529d5bfa29),
    ] {
        let mut rng = RoleRng::restore(zero_at(stream, initial)).unwrap();
        assert_eq!(rng.try_next_u64().unwrap(), expected);
        assert_eq!(rng.checkpoint(), zero_at(stream, initial + 2));
    }
}

#[test]
fn role_derivation_and_repeated_context_paths_have_literal_expected_order() {
    let root = RoleRng::root_role();
    let voter = RoleRng::child_role(root, "voter");
    let attribute = RoleRng::attribute_role(voter, "epoch", "5");
    assert_eq!(
        root,
        hex32("7e18f65d5736bd4967d4213d11aeab38e5fb52520f9fd3e625c6d476396d379c")
    );
    assert_eq!(
        voter,
        hex32("ccb1b4a0e0532ddc1d79bafea0f5feec8fcb2ef8f668b3c2df0a097f89f834d0")
    );
    assert_eq!(
        attribute,
        hex32("6eefcf1ca14a62f0266612905bb661f27397a0aeca6207bc6494c3b02d6ec9ff")
    );
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
            attribute,
            "6c07d3625500da2a95e23c6a9fbea867c5d6072f6d98e41c90e85a1ab12d1882",
        ),
    ] {
        assert_eq!(
            RoleRng::derive(7, 0, CID, role).checkpoint().seed,
            hex32(seed)
        );
    }
    let dir = TestDir::new();
    let profile = profile();
    let draws = Arc::new(Mutex::new(vec![]));
    let probe = draws.clone();
    let checkpoint = ControlledDriver::run(dir.config(), profile.clone(), move |driver| {
        calibrate(driver, &profile, START + 500);
        let grant = Grant {
            id: 3,
            tick_us: START,
        };
        let step = driver.grant(grant, Input::None, |driver| {
            driver.spawn_session(CID, STOP, move |context| async move {
                let mut first = context.child("voter");
                let mut second = context.child("voter");
                let mut other = context.child("voter").with_attribute("epoch", 5);
                let a = first.try_next_u32().unwrap();
                let _ = other.try_next_u64().unwrap();
                let b = second.try_next_u32().unwrap();
                let c = first.try_next_u32().unwrap();
                probe.lock().unwrap().extend([a, b, c]);
            })
        })?;
        assert_eq!(
            step.events
                .iter()
                .filter(|e| matches!(e, Event::RngDraw { role, .. } if *role == voter))
                .count(),
            3
        );
        let mut checkpoint = None;
        driver.grant(
            Grant {
                id: 4,
                tick_us: START,
            },
            Input::None,
            |driver| {
                checkpoint = Some(driver.rng_checkpoint(CID, voter)?);
                Ok(())
            },
        )?;
        Ok(checkpoint.unwrap())
    })
    .unwrap();
    assert_eq!(*draws.lock().unwrap(), [0x03815b53, 0xee086c42, 0xe997babc]);
    assert_eq!(checkpoint.word_position_lo, 3);
    let dir = TestDir::new();
    let mut profile = support::profile();
    profile.incarnation = 2;
    ControlledDriver::run(dir.config(), profile.clone(), move |driver| {
        calibrate(driver, &profile, START + 500);
        driver.grant(
            Grant {
                id: 3,
                tick_us: START,
            },
            Input::None,
            |driver| {
                assert!(driver.restore_rng(CID, root, checkpoint.clone()).is_err());
                driver.restore_rng(CID, voter, checkpoint)?;
                driver.spawn_session(CID, STOP, |context| async move {
                    assert_eq!(context.child("voter").try_next_u32().unwrap(), 0xb9cc119c);
                })
            },
        )?;
        Ok(())
    })
    .unwrap();
}

#[test]
fn exhaustion_rejects_before_drawing_and_reaches_driver_failure_cleanup() {
    assert!(
        RoleRng::restore(RngCheckpoint {
            word_position_hi: 16,
            ..zero_at(0, 0)
        })
        .is_err()
    );
    let checkpoint = RngCheckpoint {
        word_position_hi: 15,
        ..zero_at(0, u64::MAX)
    };
    let mut rng = RoleRng::restore(checkpoint.clone()).unwrap();
    rng.try_fill_bytes(&mut []).unwrap();
    let mut bytes = [0x9f; 1];
    assert!(rng.try_fill_bytes(&mut bytes).is_err());
    assert_eq!(bytes, [0x9f]);
    assert!(rng.try_next_u32().is_err());
    assert_eq!(rng.checkpoint(), checkpoint);

    let dir = TestDir::new();
    let profile = profile();
    let root = RoleRng::root_role();
    let seed = RoleRng::derive(7, 0, CID, root).checkpoint().seed;
    let result = ControlledDriver::run(dir.config(), profile.clone(), move |driver| {
        calibrate(driver, &profile, START + 500);
        let failed = driver.grant(
            Grant {
                id: 3,
                tick_us: START,
            },
            Input::None,
            |driver| {
                driver.restore_rng(CID, root, RngCheckpoint { seed, ..checkpoint })?;
                driver.spawn_session(CID, STOP, |mut context| async move {
                    let _ = context.try_next_u32();
                    panic!("infallible adapter fabricated a value");
                })
            },
        );
        assert!(failed.is_err());
        let events = driver.events();
        assert!(events.contains(&Event::Failure {
            reason: "RNG stream exhausted"
        }));
        assert!(!events.contains(&Event::Ack { grant: 3 }));
        assert!(events.contains(&Event::CleanupComplete));
        Ok(())
    });
    assert!(result.is_err());
}
