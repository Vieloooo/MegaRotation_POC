//! Independent source-rule projection for the bounded M2 live run and M1 smoke.
//! Uses public 03 typed codecs/PoP and upstream native signatures. Does not call GFR, Merge,
//! Publisher, Manager, production FinalVerifier or candidate execution routines.
use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs,
    path::Path,
};

use commonware_codec::{Copying, Decode as CwDecode, Encode as CwEncode};
use commonware_consensus::simplex::{scheme::bls12381_multisig::Scheme, types::Finalization};
use commonware_cryptography::{
    bls12381::primitives::variant::MinSig, ed25519::PublicKey, sha256::Digest,
};
use commonware_parallel::Sequential;
use poc_authorization::{StaticScheduleProvider, ValidatedRegistry};
use poc_types::domain::{
    Account, Anchor, Attributes, BlockRef, BoundaryCause, CanonicalObject, Cid, DepFixture,
    Descriptor, ExecutionAnchor, ExecutionState, Hash32, L1Fixture, Metadata, NativeBlock,
    PhysicalBlock, PublicConfig, RegistryRecord, Transfer,
};
use rand_chacha::ChaCha20Rng;
use rand_core::SeedableRng;
use serde::{Deserialize, de::DeserializeOwned};
use serde_json::json;
use sha2::{Digest as _, Sha256};

#[path = "../../../m3-auto-force/historical_plan.rs"]
mod historical_plan;
#[path = "../../../m4-resume/replay_resume.rs"]
mod replay_resume;
use historical_plan::{ActivePolicy, Header, Policy};

const PUBLICATION_CODEC: u32 = 0x47465201;
type NativeScheme = Scheme<PublicKey, MinSig>;
type NativeFinal = Finalization<NativeScheme, Digest>;

fn framed_hash(parts: &[&[u8]]) -> [u8; 32] {
    let mut hash = Sha256::new();
    for bytes in parts {
        hash.update((bytes.len() as u64).to_be_bytes());
        hash.update(bytes);
    }
    hash.finalize().into()
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|x| format!("{x:02x}")).collect()
}

fn read<T: CanonicalObject + DeserializeOwned>(path: &str) -> T {
    let bytes = fs::read(path).unwrap_or_else(|e| panic!("{path}: {e}"));
    if path.ends_with(".json") {
        serde_json::from_slice(&bytes).unwrap_or_else(|e| panic!("{path}: {e}"))
    } else {
        T::decode(&bytes).unwrap_or_else(|e| panic!("{path}: {e}"))
    }
}

fn take<'a>(input: &mut &'a [u8], count: usize) -> &'a [u8] {
    let (value, tail) = input
        .split_at_checked(count)
        .expect("truncated GFRP object");
    *input = tail;
    value
}

fn frame<'a>(input: &mut &'a [u8]) -> &'a [u8] {
    let count = u32::from_be_bytes(take(input, 4).try_into().unwrap());
    take(input, count as usize)
}

#[derive(Clone)]
struct Proof {
    desc: Descriptor,
    target: Hash32,
    final_: NativeFinal,
    signers: Vec<u8>,
    q: u64,
    message: usize,
}

fn result(
    authorization: &StaticScheduleProvider,
    mut input: &[u8],
    q: u64,
    message: usize,
    historical: bool,
) -> Proof {
    let desc = Descriptor::decode(frame(&mut input)).expect("canonical descriptor");
    if historical && desc.plan.leader_rule.rule_id == 2 {
        historical_plan::check_descriptor_base(&desc, authorization);
    } else {
        authorization
            .validate_descriptor(&desc)
            .expect("config, Plan and cid binding");
    }
    let target = Hash32::try_from(take(&mut input, 32)).unwrap();
    let proof_bytes = frame(&mut input);
    assert!(input.is_empty(), "trailing GFRP proof bytes");
    assert!(proof_bytes.len() <= NativeFinal::max_encode_size(5));
    let final_ = NativeFinal::decode_cfg(Copying(proof_bytes), &5).expect("native Finalization");
    assert!(final_.encode().as_ref() == proof_bytes);
    assert_eq!(final_.proposal.payload.0, target.into_bytes());
    assert_eq!(final_.round().epoch().get(), 1);
    assert!((1..=128).contains(&final_.round().view().get()));
    assert!(final_.proposal.parent < final_.round().view());
    let config = authorization.config();
    let namespace = framed_hash(&[
        b"PoC/RegionalBFT/Simplex/v1",
        desc.plan.config_ref.as_bytes(),
        desc.cid.as_bytes(),
        &1u64.to_be_bytes(),
        &config.rule_version.to_be_bytes(),
        &config.verifier_version.to_be_bytes(),
    ]);
    let registry = authorization.registry();
    let committee = registry.participants(desc.plan.region).clone();
    let verifier = NativeScheme::verifier(&namespace, committee);
    let mut rng = ChaCha20Rng::from_seed(namespace);
    assert_eq!(final_.certificate.signers.len(), 5);
    let signers: Vec<_> = final_
        .certificate
        .signers
        .iter()
        .map(|index| {
            let index =
                poc_types::domain::SignerIndex::new(index.get().try_into().unwrap()).unwrap();
            registry.pid_at(desc.plan.region, index).get()
        })
        .collect();
    assert!((4..=5).contains(&signers.len()));
    assert!(
        final_.verify(&mut rng, &verifier, &Sequential),
        "invalid BLS FINAL for q={q} message={message}"
    );
    Proof {
        desc,
        target,
        final_,
        signers,
        q,
        message,
    }
}

fn update_account(state: &mut ExecutionState, pid: u8, account: Account) {
    if account.balance == 0 && account.nonce == 0 {
        state.accounts.remove(&pid);
    } else {
        state.accounts.insert(pid, account);
    }
}

fn account(state: &ExecutionState, pid: u8) -> Account {
    state.accounts.get(&pid).cloned().unwrap_or(Account {
        balance: 0,
        nonce: 0,
    })
}

fn transfer(state: &mut ExecutionState, raw: &[u8]) -> bool {
    let transaction = Transfer::decode_raw(raw).expect("typed transfer");
    let mut payer = account(state, transaction.from);
    if payer.nonce != transaction.nonce || payer.balance < transaction.amount {
        return false;
    }
    let Some(new_nonce) = payer.nonce.checked_add(1) else {
        return false;
    };
    payer.nonce = new_nonce;
    if transaction.from != transaction.to {
        let mut payee = account(state, transaction.to);
        let Some(new_balance) = payee.balance.checked_add(transaction.amount) else {
            return false;
        };
        payee.balance = new_balance;
        payer.balance -= transaction.amount;
        update_account(state, transaction.to, payee);
    }
    update_account(state, transaction.from, payer);
    true
}

#[derive(Clone, Copy, Deserialize)]
struct Control {
    position: [u64; 3],
    seq: u64,
    act_us: u64,
    target: u64,
    l1_floor: u64,
    policy: Option<Policy>,
}

#[derive(Clone)]
struct Duty {
    beta: u64,
    r: u64,
    cause: BoundaryCause,
    region: u64,
    source_region: Option<u64>,
    min: u64,
    max: u64,
    anchor: Anchor,
    reference: BlockRef,
    cid: Option<Cid>,
    active_policy: Option<ActivePolicy>,
}

impl Duty {
    fn check_descriptor(
        &mut self,
        descriptor: &Descriptor,
        authorization: &StaticScheduleProvider,
        historical_headers: Option<&[Header]>,
    ) {
        let plan = &descriptor.plan;
        assert_eq!((plan.session.beta, plan.session.r), (self.beta, self.r));
        assert_eq!(plan.cause, self.cause);
        assert_eq!(
            plan.source_region.map(|r| u64::from(r.get())),
            self.source_region
        );
        assert_eq!(u64::from(plan.region.get()), self.region);
        assert_eq!(plan.reference_block, self.reference);
        assert_eq!((plan.window.h_min, plan.window.h_max), (self.min, self.max));
        assert_eq!(descriptor.anchor, self.anchor);
        if let Some(policy) = self.active_policy {
            policy.check_plan(descriptor, authorization, historical_headers.unwrap());
        } else {
            assert_eq!(
                plan.leader_rule,
                poc_types::domain::RuleSpec::static_schedule()
            );
        }
        assert_eq!(*self.cid.get_or_insert(descriptor.cid), descriptor.cid);
    }

    fn regular(&self, anchor: &Anchor) -> Self {
        Self {
            beta: self.beta,
            r: self.r + 1,
            cause: BoundaryCause::Handoff,
            region: match self.active_policy {
                Some(active) => active
                    .policy
                    .select(Some(self.region))
                    .expect("evidence profile requires an eligible historical handoff"),
                None => (self.beta + self.r + 1) % 3,
            },
            source_region: Some(self.region),
            min: self.max + 1,
            max: self.max + 10,
            anchor: anchor.clone(),
            reference: self.anchor.block.clone(),
            cid: None,
            active_policy: self.active_policy,
        }
    }
}

#[derive(Clone)]
struct Chain {
    head: Anchor,
    state: ExecutionState,
    native: Hash32,
}

struct Prepared {
    proof: Proof,
    blocks: Vec<Hash32>,
    q_avail: u64,
}

fn complete(proof: Proof, bodies: &BTreeMap<Hash32, (PhysicalBlock, u64)>) -> Option<Prepared> {
    let mut blocks = Vec::new();
    let anchor = &proof.desc.anchor.block;
    let mut id = proof.target;
    let mut max_height = u64::MAX;
    let mut q_avail = proof.q;
    while id != anchor.block_id {
        let (body, q) = bodies.get(&id)?;
        let h = body.metadata.height;
        assert!(
            anchor.height < h && h < max_height,
            "invalid ancestry order"
        );
        q_avail = q_avail.max(*q);
        blocks.push(id);
        id = body.parent;
        max_height = h;
    }
    assert!(!blocks.is_empty());
    blocks.reverse();
    Some(Prepared {
        proof,
        blocks,
        q_avail,
    })
}

fn timestamp(config: &PublicConfig, height: u64) -> u64 {
    config.genesis.timestamp_us + (height - config.genesis.block.height) * config.l2_slot_us
}

fn append(
    chain: &mut Chain,
    duty: &Duty,
    config: &PublicConfig,
    l1: &L1Fixture,
    dep: &DepFixture,
    height: u64,
    block: Option<&PhysicalBlock>,
) -> (Hash32, Vec<Vec<u8>>) {
    let slot_time = timestamp(config, height);
    let scheduled =
        (slot_time - config.l1_genesis_timestamp_us - config.t_dep_us) / config.l1_block_us;
    let origin = (duty.anchor.block.origin + height - duty.anchor.block.height)
        .min(duty.anchor.block.origin.max(scheduled));
    let metadata = Metadata {
        height,
        timestamp_us: slot_time,
        origin,
    };
    let header = &l1.rows[origin as usize].header;
    assert_eq!(header.height, origin);
    let origin_ts = header.timestamp_us().unwrap();
    assert!(origin_ts <= slot_time && slot_time - origin_ts <= config.t_drift_us);
    assert!(origin >= chain.state.metadata.origin && origin <= chain.state.metadata.origin + 1);
    let mut kept = Vec::new();
    if let Some(block) = block {
        assert_eq!(block.metadata, metadata);
        assert_eq!(block.pre_state_root, chain.state.root().unwrap());
        assert!(
            block.transactions.iter().map(Vec::len).sum::<usize>() <= config.byte_limit as usize
        );
    }
    chain.state.metadata = metadata.clone();
    let mut dep_raw = vec![
        Attributes {
            metadata: metadata.clone(),
        }
        .encode_raw(),
    ];
    for credit in &dep.rows[(height - dep.first_height) as usize] {
        dep_raw.push(credit.encode_raw());
        let mut balance = account(&chain.state, credit.to);
        balance.balance = balance.balance.checked_add(credit.amount).unwrap();
        update_account(&mut chain.state, credit.to, balance);
    }
    if let Some(block) = block {
        for tx in &block.transactions {
            if transfer(&mut chain.state, tx) {
                kept.push(tx.clone());
            }
        }
    }
    let root = chain.state.root().unwrap();
    let native = NativeBlock {
        parent: chain.native,
        metadata,
        dep: dep_raw,
        kept,
        post_state_root: root,
    };
    chain.native = native.digest().unwrap();
    if let Some(block) = block {
        chain.head = Anchor {
            block: BlockRef {
                height,
                origin,
                block_id: block.digest().unwrap(),
            },
            timestamp_us: slot_time,
        };
    }
    (root, native.kept)
}

fn main() {
    let args: Vec<_> = env::args().skip(1).collect();
    assert!(
        (6..=8).contains(&args.len())
            || (args.len() == 9 && args[8] == "--timer-history"),
        "usage: RAW CONTROLS_JSON ANNOUNCEMENT_US GRACE_US EXPIRY_US INIT_US [ACTUAL_L1_JSON [VERIFIED_RESUME_JSON [--timer-history]]]"
    );
    let track_timers = args.len() == 9;
    let headers: Option<Vec<Header>> = args.get(6).map(|path| {
        serde_json::from_slice(&fs::read(path).unwrap()).expect("actual L1 header projection")
    });
    let historical_headers = headers.as_deref();
    let mut resume_inputs = args.get(7).map(|path| {
        replay_resume::ResumeReplay::from_controls(
            serde_json::from_slice(&fs::read(path).unwrap()).expect("independently verified cuts"),
        )
    });
    let raw = Path::new(&args[0]);
    let path = |name: &str| raw.join(name).to_str().unwrap().to_owned();
    let config: PublicConfig = read(&path("config.bin"));
    let registry: RegistryRecord = read(&path("registry.bin"));
    let genesis: ExecutionAnchor = read(&path("genesis.bin"));
    let dep: DepFixture = read(&path("dep.bin"));
    let l1: L1Fixture = read(&path("l1.bin"));
    let ma: u64 = args[2].parse().unwrap();
    let grace: u64 = args[3].parse().unwrap();
    let expiry: u64 = args[4].parse().unwrap();
    let init: u64 = args[5].parse().unwrap();
    let mut controls: Vec<Control> = serde_json::from_slice(&fs::read(&args[1]).unwrap()).unwrap();
    assert!(
        controls
            .iter()
            .all(|c| c.policy.is_some() == headers.is_some())
    );
    if let Some(headers) = historical_headers {
        assert!(!headers.is_empty());
        for (index, header) in headers.iter().enumerate() {
            assert_eq!(header.height, index as u64);
            let nominal = &l1.rows[index].header;
            assert_eq!(
                (header.height, header.timestamp_s),
                (nominal.height, nominal.timestamp_s)
            );
        }
    }
    controls.sort_by_key(|c| (c.act_us, c.position));
    assert!(
        controls
            .windows(2)
            .all(|a| a[0].position < a[1].position && a[0].seq < a[1].seq)
    );
    let authorization = StaticScheduleProvider::new(
        config.clone(),
        ValidatedRegistry::load(registry).expect("registered keys and PoPs"),
    )
    .unwrap();
    assert_eq!(config.genesis, genesis.anchor);
    assert_eq!(config.genesis_state_root, genesis.state.root().unwrap());
    assert_eq!(config.genesis_native_hash, genesis.native_hash);
    assert_eq!(config.dep_fixture_hash, dep.digest().unwrap());
    assert_eq!(config.l1_fixture_hash, l1.digest().unwrap());
    assert_eq!(genesis.anchor.block.height, 0);
    genesis.validate().unwrap();

    let mut bodies = BTreeMap::new();
    let mut proofs = Vec::new();
    let mut rows = BTreeMap::new();
    let mut espresso = Vec::new();
    for file in fs::read_dir(raw.join("espresso")).unwrap() {
        let file = file.unwrap().path();
        if file.extension().is_none_or(|e| e != "bin") {
            continue;
        }
        let bytes = fs::read(&file).unwrap();
        let (source, row) = sim_espresso::decode_finalized(&bytes).expect("canonical PELG bytes");
        let q = row.proposal.q;
        assert!(
            rows.insert(q, (source, row)).is_none(),
            "duplicate public block at q={q}"
        );
    }
    let mut previous = None;
    for (q, (source, row)) in rows {
        let theta = row.proposal.theta_us.0;
        let producer = row.proposal.proposed_at_us.0;
        let finalize = row.finalized_at_us.0;
        if !row.proposal.live_at_proposal {
            assert!(
                track_timers && row.proposal.messages.is_empty()
                    && previous.as_ref().is_some_and(|(_, old, _, _)| *old == theta),
                "non-live M5 rows must be empty at the unchanged public key"
            );
        }
        assert!(producer <= finalize);
        if let Some((last_q, last_theta, last_finalize, last_source)) = &previous {
            assert_eq!(q, last_q + 1);
            assert!(theta >= *last_theta && finalize >= *last_finalize);
            assert_eq!(&source, last_source);
        } else {
            assert_eq!(q, 0);
        }
        previous = Some((q, theta, finalize, source));
        let mut row_evidence =
            json!({"q":q, "theta_us":theta, "proposed_us":producer, "finalized_us":finalize});
        if track_timers {
            row_evidence["messages"] = json!(row.proposal.messages.len());
            row_evidence["live_at_proposal"] = json!(row.proposal.live_at_proposal);
        }
        espresso.push(row_evidence);
        for (message, log_message) in row.proposal.messages.into_iter().enumerate() {
            assert_eq!(log_message.codec, PUBLICATION_CODEC);
            let mut input = log_message.bytes.as_slice();
            assert_eq!(take(&mut input, 6), b"GFRP\0\x01");
            match take(&mut input, 1)[0] {
                0 => {
                    let body =
                        PhysicalBlock::decode(frame(&mut input)).expect("canonical physical body");
                    assert!(input.is_empty());
                    let digest = body.digest().unwrap();
                    assert_eq!(
                        digest.into_bytes(),
                        framed_hash(&[b"PoC/PhysicalBlock/v1", &body.encode_checked().unwrap(),])
                    );
                    let recorded = bodies.entry(digest).or_insert_with(|| (body.clone(), q));
                    assert_eq!(recorded.0, body);
                }
                1 => proofs.push(result(&authorization, input, q, message, headers.is_some())),
                kind => panic!("unexpected GFRP kind {kind}"),
            }
        }
    }

    let q_thetas: Vec<_> = espresso
        .iter()
        .map(|row| row["theta_us"].as_u64().unwrap())
        .collect();
    let mut eligible: Vec<_> = proofs
        .iter()
        .cloned()
        .filter_map(|p| complete(p, &bodies))
        .collect();
    let public_finals: Vec<_> = eligible.iter().map(|done| {
        let p = &done.proof;
        json!({
            "cid": hex(p.desc.cid.as_bytes()), "target": hex(p.target.as_bytes()),
            "height": bodies[&p.target].0.metadata.height, "session": p.desc.plan.session,
            "view": p.final_.round().view().get(), "signers": p.signers,
            "proof": hex(p.final_.encode().as_ref()), "subject": hex(p.final_.proposal.encode().as_ref()),
            "q": p.q, "q_avail": done.q_avail, "message": p.message,
            "q_avail_theta_us":q_thetas[done.q_avail as usize],
        })
    }).collect();
    eligible.sort_by_key(|e| (e.proof.target, e.q_avail, e.proof.q, e.proof.message));

    let mut global = Chain {
        head: genesis.anchor.clone(),
        state: genesis.state,
        native: genesis.native_hash,
    };
    let mut duty = Duty {
        beta: 0,
        r: 0,
        cause: BoundaryCause::Genesis,
        region: 0,
        source_region: None,
        min: 1,
        max: 10,
        anchor: genesis.anchor.clone(),
        reference: genesis.anchor.block,
        cid: None,
        active_policy: None,
    };
    let mut floor = 0;
    let mut halted = false;
    let mut controls = controls.into_iter().peekable();
    let mut force = Vec::new();
    let mut late = Vec::new();
    let mut regular = Vec::new();
    let mut expected = Vec::new();
    let mut final_accepted = BTreeSet::new();
    let mut resume_events = Vec::new();
    let mut regional_entries = Vec::new();
    let mut timers = Vec::new();

    for (q, &theta) in q_thetas.iter().enumerate() {
        let q = q as u64;
        let covering_tip_us = historical_headers.map_or_else(
            || l1.rows.last().unwrap().header.timestamp_us().unwrap(),
            |headers| {
                headers
                    .last()
                    .unwrap()
                    .timestamp_s
                    .checked_mul(1_000_000)
                    .unwrap()
            },
        );
        assert!(covering_tip_us >= theta.saturating_sub(ma));
        loop {
            let end = if halted {
                u64::MAX
            } else {
                timestamp(&config, duty.max + 1) + grace
            };
            let expire = if halted {
                u64::MAX
            } else {
                timestamp(&config, global.head.block.height + 1) + expiry
            };
            let force_position = controls.peek().map(|c| (c.act_us, c.position));
            let resume_position = resume_inputs.as_ref().and_then(|rows| rows.next());
            let control_position = match (force_position, resume_position) {
                (Some(force), Some(resume)) => Some(force.min(resume)),
                (force, resume) => force.or(resume),
            };
            let control = control_position.map_or(u64::MAX, |event| event.0);
            let next = control.min(expire).min(end);
            if next > theta {
                break;
            }
            if next == control && resume_position == control_position {
                let (event, stopped, new_floor) = resume_inputs.as_mut().unwrap().apply(
                    &mut global,
                    &mut duty,
                    &mut expected,
                    &config,
                    historical_headers.expect("RESUME requires verified real headers"),
                    q,
                    ma,
                    grace,
                    expiry,
                );
                halted = stopped;
                floor = new_floor;
                resume_events.push(event);
            } else if next == control {
                let c = controls.next().unwrap();
                let result = if halted || c.target != duty.beta {
                    json!({"seq":c.seq,"position":c.position,"act_us":c.act_us,"valid":false})
                } else {
                    for item in &eligible {
                        let desc = &item.proof.desc;
                        if item.q_avail < q
                            || !matches!(&desc.plan.session, s if s.beta == duty.beta && s.r == duty.r)
                            || desc.anchor != duty.anchor
                        {
                            continue;
                        }
                        let target = &bodies[&item.proof.target].0;
                        if target.metadata.height <= global.head.block.height {
                            continue;
                        }
                        let Some(first) = item
                            .blocks
                            .iter()
                            .position(|id| bodies[id].0.parent == global.head.block.block_id)
                        else {
                            continue;
                        };
                        let mut closed_duty = duty.clone();
                        closed_duty.check_descriptor(desc, &authorization, historical_headers);
                        let mut old_chain = global.clone();
                        let mut height = old_chain.head.block.height + 1;
                        for id in &item.blocks[first..] {
                            let block = &bodies[id].0;
                            assert!(
                                (closed_duty.min..=closed_duty.max)
                                    .contains(&block.metadata.height)
                            );
                            while height <= block.metadata.height {
                                let physical = (height == block.metadata.height).then_some(block);
                                append(
                                    &mut old_chain,
                                    &closed_duty,
                                    &config,
                                    &l1,
                                    &dep,
                                    height,
                                    physical,
                                );
                                height += 1;
                            }
                        }
                        late.push(json!({"cid":hex(desc.cid.as_bytes()),"physical":hex(item.proof.target.as_bytes()),
                            "q_avail":item.q_avail,"height":target.metadata.height}));
                    }
                    let beta = duty.beta + 1;
                    let r = duty.r + u64::from(global.head.block.height > duty.anchor.block.height);
                    let source_region = duty.region;
                    let active_policy = c.policy.map(|policy| {
                        let headers = historical_headers.expect("missing M3 history");
                        let inclusion = &headers[usize::try_from(c.position[0]).unwrap()];
                        assert_eq!(
                            inclusion.timestamp_s.checked_mul(1_000_000).unwrap() + ma,
                            c.act_us
                        );
                        ActivePolicy {
                            policy,
                            position: c.position,
                            control_seq: c.seq,
                            signed_floor: c.l1_floor,
                            effective_floor: floor.max(c.position[0]),
                        }
                    });
                    let region = match active_policy {
                        Some(active) => active
                            .policy
                            .select(Some(source_region))
                            .expect("evidence profile requires an eligible historical Force"),
                        None => (beta + r) % 3,
                    };
                    let min = (global.head.block.height + 1).max(
                        (c.act_us + init - config.genesis.timestamp_us) / config.l2_slot_us + 1,
                    );
                    floor = active_policy.map_or_else(
                        || floor.max(c.position[0]).max(c.l1_floor),
                        |active| active.effective_floor,
                    );
                    let prior_beta = duty.beta;
                    duty = Duty {
                        beta,
                        r,
                        cause: BoundaryCause::Force,
                        region,
                        source_region: Some(source_region),
                        min,
                        max: min + 9,
                        anchor: global.head.clone(),
                        reference: global.head.block.clone(),
                        cid: None,
                        active_policy,
                    };
                    let mut projection = json!({
                        "seq":c.seq,"position":c.position,"act_us":c.act_us,"valid":true,
                        "q_star":q, "closed_beta":prior_beta, "anchor":global.head,
                        "anchor_root":global.state.root().unwrap(), "native_hash":global.native,
                        "beta":beta,"r":r,"region":region,"source_region":source_region,
                        "h_min":min,"h_max":min+9,"floor":floor,
                        "latest_ready_us":timestamp(&config, min)-1000,
                        "expiry_key_us":timestamp(&config, global.head.block.height + 1)+expiry,
                    });
                    if let Some(active) = active_policy {
                        projection["historical_plan"] = active
                            .projection(global.head.block.origin, historical_headers.unwrap());
                    }
                    projection
                };
                force.push(result);
            } else if next == expire {
                if track_timers {
                    timers.push(json!({
                        "kind": "Expire",
                        "key_us": expire,
                        "height": global.head.block.height + 1,
                        "branch": duty.beta,
                        "q_star": q,
                    }));
                }
                halted = true;
            } else {
                if track_timers {
                    timers.push(json!({
                        "kind": "SessionEnd",
                        "key_us": end,
                        "branch": duty.beta,
                        "r": duty.r,
                        "q_star": q,
                    }));
                }
                regular.push(json!({"cause":"SessionEnd","key_us":end,"beta":duty.beta,
                    "r":duty.r,"anchor":global.head,"old_max":duty.max}));
                duty = duty.regular(&global.head);
            }
        }
        if halted {
            continue;
        }

        loop {
            let mut chosen = None;
            for item in &eligible {
                if item.q_avail > q {
                    continue;
                }
                let Proof { desc, target, .. } = &item.proof;
                let plan = &desc.plan;
                if plan.session.beta != duty.beta
                    || plan.session.r != duty.r
                    || desc.anchor != duty.anchor
                {
                    continue;
                }
                let high = bodies[target].0.metadata.height;
                if high <= global.head.block.height {
                    continue;
                }
                let Some(first) = item
                    .blocks
                    .iter()
                    .position(|id| bodies[id].0.parent == global.head.block.block_id)
                else {
                    continue;
                };
                let stop = timestamp(&config, duty.max + 1) + grace;
                if q_thetas[item.q_avail as usize] >= stop
                    || theta >= stop
                    || theta >= timestamp(&config, global.head.block.height + 1) + expiry
                {
                    continue;
                }
                duty.check_descriptor(desc, &authorization, historical_headers);
                chosen = Some((item, first));
                break;
            }
            let Some((item, first)) = chosen else {
                break;
            };
            let mut next_height = global.head.block.height + 1;
            for id in &item.blocks[first..] {
                let block = &bodies[id].0;
                let high = block.metadata.height;
                assert_eq!(block.parent, global.head.block.block_id);
                assert!((duty.min..=duty.max).contains(&high));
                while next_height <= high {
                    let physical = (next_height == high).then_some(block);
                    let (root, kept) = append(
                        &mut global,
                        &duty,
                        &config,
                        &l1,
                        &dep,
                        next_height,
                        physical,
                    );
                    let mut accepted = json!({
                        "height":next_height,"physical":physical.map(|_|hex(id.as_bytes())),
                        "parent":physical.map(|b|hex(b.parent.as_bytes())),
                        "native":hex(global.native.as_bytes()),"state_root":hex(root.as_bytes()),
                        "cid":hex(item.proof.desc.cid.as_bytes()),"beta":duty.beta,"r":duty.r,
                        "region":duty.region,"anchor":hex(duty.anchor.block.block_id.as_bytes()),
                        "q_avail":item.q_avail,"q":q,"theta_us":theta,
                        "target":hex(item.proof.target.as_bytes()),"kept":kept.len(),
                    });
                    if resume_inputs.is_some() {
                        accepted["metadata"] = json!(global.state.metadata);
                        accepted["kept_bytes"] = json!(kept.iter().map(|bytes| hex(bytes)).collect::<Vec<_>>());
                        regional_entries.push(accepted.clone());
                    }
                    expected.push(accepted);
                    next_height += 1;
                }
            }
            final_accepted.insert(item.proof.target);
            if global.head.block.height == duty.max {
                regular.push(
                    json!({"cause":"FullWindow","q":q,"key_us":theta,"beta":duty.beta,
                    "r":duty.r,"anchor":global.head,"old_max":duty.max}),
                );
                duty = duty.regular(&global.head);
            }
        }
    }
    let signer_indexes: Vec<_> = (0..15)
        .map(|pid| {
            authorization
                .registry()
                .signer_index(poc_types::domain::Pid::new(pid).unwrap())
                .get()
        })
        .collect();
    let l1_meta: Vec<_> = l1.rows.iter().map(|row| json!({
        "height":row.header.height, "timestamp_s":row.header.timestamp_s,
        "hash":hex(row.header.hash.as_bytes()), "parent_hash":hex(row.header.parent_hash.as_bytes()),
        "finalized_us":row.finalized_at_us,
    })).collect();
    let mut output = json!({
        "config_ref":hex(config.reference().unwrap().as_bytes()),
        "public_finals":public_finals,"signature_checks":proofs.len(),"signer_indexes":signer_indexes,
        "espresso":espresso,"expected":expected,"force":force,"regular":regular,"l1":l1_meta,"late":late,
        "accepted_final_count":final_accepted.len(),"halted":halted,
    });
    if let Some(resumes) = resume_inputs {
        assert!(resumes.next().is_none(), "Resume still missing Espresso coverage");
        output["resumes"] = json!(resume_events);
        output["regional_entries"] = json!(regional_entries);
    }
    if track_timers {
        output["timers"] = json!(timers);
    }
    println!("{}", serde_json::to_string_pretty(&output).unwrap());
}
