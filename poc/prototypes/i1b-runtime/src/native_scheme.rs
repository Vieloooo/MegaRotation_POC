use crate::{
    Id,
    driver::Authority,
    native_config::NativeScheme,
    native_wire::{SignAction, SignAudit, SignKind},
};
use commonware_codec::{Encode, Read};
use commonware_consensus::simplex::types::Subject;
use commonware_cryptography::{
    Digest,
    certificate::{
        AssemblyError, Attestation, Membership, Scheme, Subject as _, Verification, Verifier,
    },
    ed25519::PublicKey,
};
use commonware_parallel::Strategy;
use commonware_utils::{N3f1, iter::NonEmpty, ordered::Set};
use rand_core::CryptoRng;
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

type Coordinates = (i32, u64, u64);
type Signed = (Vec<u8>, Attestation<NativeScheme>);
#[derive(Clone)]
pub(crate) struct GuardedScheme {
    native: NativeScheme,
    scope: Authority,
    cid: Id,
    epoch: u64,
    signer: u32,
    cache: Arc<Mutex<BTreeMap<Coordinates, Signed>>>,
    audits: Arc<Mutex<Vec<SignAudit>>>,
}

impl std::fmt::Debug for GuardedScheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GuardedScheme")
            .field("cid", &self.cid)
            .field("signer", &self.signer)
            .finish()
    }
}

impl GuardedScheme {
    pub fn new(
        native: NativeScheme,
        scope: Authority,
        cid: Id,
        epoch: u64,
        signer: u32,
        audits: Arc<Mutex<Vec<SignAudit>>>,
    ) -> Self {
        Self {
            native,
            scope,
            cid,
            epoch,
            signer,
            cache: Arc::default(),
            audits,
        }
    }
    fn native_attestation(a: &Attestation<Self>) -> Attestation<NativeScheme> {
        Attestation {
            signer: a.signer,
            signature: a.signature.clone(),
        }
    }
    fn guarded(a: Attestation<NativeScheme>) -> Attestation<Self> {
        Attestation {
            signer: a.signer,
            signature: a.signature,
        }
    }
    fn verification(v: Verification<NativeScheme>) -> Verification<Self> {
        Verification::new(
            v.verified.into_iter().map(Self::guarded).collect(),
            v.invalid,
        )
    }
}

impl Verifier for GuardedScheme {
    type Subject<'a, D: Digest> = Subject<'a, D>;
    type Faults = N3f1;
    type PublicKey = PublicKey;
    type Certificate = <NativeScheme as Verifier>::Certificate;
    fn verify_certificate<R: CryptoRng, D: Digest>(
        &self,
        rng: &mut R,
        subject: Subject<'_, D>,
        certificate: &Self::Certificate,
        strategy: &impl Strategy,
    ) -> bool {
        self.native
            .verify_certificate(rng, subject, certificate, strategy)
    }
    fn verify_certificates<'a, R, D, I>(
        &self,
        rng: &mut R,
        certificates: NonEmpty<I>,
        strategy: &impl Strategy,
    ) -> bool
    where
        R: CryptoRng,
        D: Digest,
        I: Iterator<Item = (Subject<'a, D>, &'a Self::Certificate)>,
    {
        self.native.verify_certificates(rng, certificates, strategy)
    }
    fn is_batchable() -> bool {
        NativeScheme::is_batchable()
    }
    fn max_certificate_size(n: u32) -> usize {
        NativeScheme::max_certificate_size(n)
    }
    fn certificate_codec_config(&self) -> <Self::Certificate as Read>::Cfg {
        self.native.certificate_codec_config()
    }
    fn certificate_codec_config_unbounded() -> <Self::Certificate as Read>::Cfg {
        NativeScheme::certificate_codec_config_unbounded()
    }
}

impl Scheme for GuardedScheme {
    type Signature = <NativeScheme as Scheme>::Signature;
    fn me(&self) -> Option<Membership> {
        self.native.me()
    }
    fn participants(&self) -> &Set<PublicKey> {
        self.native.participants()
    }
    fn sign<D: Digest>(&self, subject: Subject<'_, D>) -> Option<Attestation<Self>> {
        let (kind, round) = match subject {
            Subject::Notarize { proposal } => (SignKind::Notarize, proposal.round),
            Subject::Nullify { round } => (SignKind::Nullify, round),
            Subject::Finalize { proposal } => (SignKind::Finalize, proposal.round),
        };
        let key = (kind as i32, round.epoch().get(), round.view().get());
        let bytes = subject.message().to_vec();
        let access = self.scope.check();
        let snapshot = access
            .as_ref()
            .cloned()
            .unwrap_or_else(|_| self.scope.snapshot());
        let mut reason = "";
        let (action, value) = if access.is_err()
            || key.1 != self.epoch
            || self.scope.cid != self.cid
            || !(1..=8).contains(&key.2)
        {
            reason = "unsafe_clock_scope_or_round";
            (SignAction::Denied, None)
        } else {
            let mut cache = self.cache.lock().unwrap();
            if let Some((old, value)) = cache.get(&key) {
                if *old == bytes {
                    (SignAction::CacheHit, Some(value.clone()))
                } else {
                    reason = "conflicting_subject";
                    (SignAction::Denied, None)
                }
            } else if let Some(value) = self.native.sign(subject) {
                cache.insert(key, (bytes.clone(), value.clone()));
                (SignAction::Created, Some(value))
            } else {
                reason = "native_signer_refused";
                (SignAction::Denied, None)
            }
        };
        self.audits.lock().unwrap().push(SignAudit {
            cid: self.cid.to_vec(),
            epoch: key.1,
            view: key.2,
            kind: key.0,
            action: action as i32,
            subject_bytes: bytes,
            attestation_bytes: value
                .as_ref()
                .map_or_else(Vec::new, |v| v.encode().to_vec()),
            signer_index: self.signer,
            grant_id: snapshot.grant_id,
            task_id: snapshot.task_id,
            tick_us: snapshot.tick_us,
            local_time_us: snapshot.local_time_us.unwrap_or_default(),
            sample_sequence: snapshot.sample_sequence,
            lease_end_tick_us: snapshot.lease_end_tick_us,
            local_stop_us: snapshot.local_stop_us.unwrap_or_default(),
            reason: reason.into(),
        });
        if reason == "conflicting_subject" || !(1..=8).contains(&key.2) {
            panic!("native signing fixture violated: {reason}");
        }
        value.map(Self::guarded)
    }
    fn verify_attestation<R: CryptoRng, D: Digest>(
        &self,
        rng: &mut R,
        subject: Subject<'_, D>,
        attestation: &Attestation<Self>,
        strategy: &impl Strategy,
    ) -> bool {
        self.native.verify_attestation(
            rng,
            subject,
            &Self::native_attestation(attestation),
            strategy,
        )
    }
    fn verify_attestations<R, D, I>(
        &self,
        rng: &mut R,
        subject: Subject<'_, D>,
        attestations: I,
        strategy: &impl Strategy,
    ) -> Verification<Self>
    where
        R: CryptoRng,
        D: Digest,
        I: IntoIterator<Item = Attestation<Self>>,
        I::IntoIter: Send,
    {
        Self::verification(
            self.native.verify_attestations(
                rng,
                subject,
                attestations
                    .into_iter()
                    .map(|a| Self::native_attestation(&a)),
                strategy,
            ),
        )
    }
    fn optimistic_assemble<'a, R, D, I, J>(
        &self,
        rng: &mut R,
        subject: Subject<'_, D>,
        pending: I,
        verified: J,
        strategy: &impl Strategy,
    ) -> Result<Self::Certificate, Verification<Self>>
    where
        R: CryptoRng,
        D: Digest,
        I: IntoIterator<Item = Attestation<Self>>,
        I::IntoIter: ExactSizeIterator + Send,
        J: IntoIterator<Item = &'a Attestation<Self>>,
        J::IntoIter: Send,
    {
        let verified: Vec<_> = verified.into_iter().map(Self::native_attestation).collect();
        self.native
            .optimistic_assemble(
                rng,
                subject,
                pending.into_iter().map(|a| Self::native_attestation(&a)),
                verified.iter(),
                strategy,
            )
            .map_err(Self::verification)
    }
    fn assemble<I>(
        &self,
        attestations: NonEmpty<I>,
        strategy: &impl Strategy,
    ) -> Result<Self::Certificate, AssemblyError>
    where
        I: Iterator<Item = Attestation<Self>> + Send,
    {
        let mut input = attestations
            .into_iter()
            .map(|a| Self::native_attestation(&a));
        self.native.assemble(
            NonEmpty::new(input.next().expect("nonempty attestations"), input),
            strategy,
        )
    }
    fn is_attributable() -> bool {
        NativeScheme::is_attributable()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ClockSample, ControlledDriver, Grant, Input,
        native_config::{Resolved, START},
    };
    use commonware_consensus::{
        simplex::types::{Notarize, Proposal},
        types::{Epoch, Round, View},
    };
    use commonware_cryptography::sha256::Digest;
    use std::fs;

    #[test]
    fn native_cache_obeys_the_half_open_lease_and_permanent_close() {
        for expiry in [false, true] {
            let (config, mut keys) = Resolved::generate("healthy", 7).unwrap();
            let native = config.signer(0, keys.remove(0)).unwrap();
            let profile = config.profile(0);
            let lease_end = START + 10_000;
            let spec = ClockSample {
                run_id: profile.run_id,
                node: 0,
                incarnation: 1,
                clock_domain: profile.clock_domain(),
                rate_profile: crate::Profile::rate_profile(),
                request_id: 1,
                sequence: 1,
                as_of_tick_us: START,
                node_time_at_as_of_us: START as i64 + 1000,
                uncertainty_us: 1000,
                lease_end_tick_us: lease_end,
            };
            let root = (0..u64::MAX)
                .map(|i| {
                    std::env::temp_dir().join(format!("native-scheme-{}-{i}", std::process::id()))
                })
                .find(|path| fs::create_dir(path).is_ok())
                .unwrap();
            let audits = Arc::new(Mutex::new(Vec::new()));
            let collected = audits.clone();
            let saved = Arc::new(Mutex::new(None::<GuardedScheme>));
            let retained = saved.clone();
            let proposal = Proposal::new(
                Round::new(Epoch::new(1), View::new(1)),
                View::new(0),
                Digest([0x41; 32]),
            );
            let original = proposal.clone();
            ControlledDriver::run_async(
                commonware_runtime::tokio::Config::default().with_storage_directory(root.clone()),
                profile,
                async |driver| {
                    driver.enable_native_clock_audit();
                    driver.grant(
                        Grant {
                            id: 1,
                            tick_us: START,
                        },
                        Input::RequestSample,
                        |_| Ok(()),
                    )?;
                    driver.grant(
                        Grant {
                            id: 2,
                            tick_us: START,
                        },
                        Input::DeliverSample(spec),
                        |_| Ok(()),
                    )?;
                    driver
                        .grant_async(
                            Grant {
                                id: 3,
                                tick_us: START,
                            },
                            Input::None,
                            |driver| {
                                driver.spawn_session(
                                    config.cid,
                                    config.base.stop,
                                    move |context| async move {
                                        let guarded = GuardedScheme::new(
                                            native,
                                            context.authority("test-signer"),
                                            config.cid,
                                            1,
                                            config.members[0].signer_index,
                                            audits,
                                        );
                                        let one =
                                            Notarize::sign(&guarded, original.clone()).unwrap();
                                        let cached = Notarize::sign(&guarded, original).unwrap();
                                        assert_eq!(one.encode(), cached.encode());
                                        *retained.lock().unwrap() = Some(guarded);
                                        std::future::pending::<()>().await;
                                    },
                                )
                            },
                        )
                        .await?;
                    let (input, tick) = if expiry {
                        (Input::None, lease_end)
                    } else {
                        (Input::Close(config.cid), START + 10)
                    };
                    driver
                        .grant_async(
                            Grant {
                                id: 4,
                                tick_us: tick,
                            },
                            input,
                            |_| {
                                let guarded = saved.lock().unwrap();
                                assert!(
                                    Notarize::sign(guarded.as_ref().unwrap(), proposal).is_none()
                                );
                                Ok(())
                            },
                        )
                        .await?;
                    Ok(())
                },
            )
            .unwrap();
            let audits = collected.lock().unwrap();
            assert_eq!(
                audits.iter().map(|row| row.action).collect::<Vec<_>>(),
                [
                    SignAction::Created as i32,
                    SignAction::CacheHit as i32,
                    SignAction::Denied as i32
                ]
            );
            assert!(audits[2].attestation_bytes.is_empty());
            fs::remove_dir_all(root).unwrap();
        }
    }
}
