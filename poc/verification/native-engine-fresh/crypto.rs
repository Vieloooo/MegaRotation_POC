//! Public-key-only native codec/cryptography oracle. No builder native_* helpers.
#[path = "control.rs"]
mod control;
use commonware_codec::{Copying, Decode, DecodeExt, Encode};
use commonware_consensus::{
    Epochable, Viewable,
    simplex::{
        scheme::bls12381_multisig::Scheme,
        types::{Artifact, Certificate, Finalization, Proposal, Subject, Vote},
    },
    types::Round,
};
use commonware_cryptography::{
    bls12381::primitives::{
        ops,
        variant::{MinSig, Variant},
    },
    certificate::{Attestation, Scheme as _, Verifier as _},
    ed25519::PublicKey,
    sha256::Digest,
};
use commonware_parallel::Sequential;
use commonware_utils::ordered::BiMap;
use poc_native_runtime::native_wire as p;
use prost::Message;
use rand_chacha::ChaCha20Rng;
use rand_core::SeedableRng;
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest as _, Sha256};
use std::io::{self, BufRead, Write};
type S = Scheme<PublicKey, MinSig>;
type V = Vote<S, Digest>;
type C = Certificate<S, Digest>;
type A = Artifact<S, Digest>;
type R<T> = Result<T, String>;
fn need(ok: bool, why: &str) -> R<()> {
    if ok { Ok(()) } else { Err(why.into()) }
}
fn err(e: impl std::fmt::Debug) -> String {
    format!("{e:?}")
}
pub fn hash(domain: &str, parts: &[&[u8]]) -> [u8; 32] {
    let mut h = Sha256::new();
    for p in std::iter::once(domain.as_bytes()).chain(parts.iter().copied()) {
        h.update((p.len() as u64).to_be_bytes());
        h.update(p);
    }
    h.finalize().into()
}
fn bytes(v: &Value, key: &str) -> R<Vec<u8>> {
    serde_json::from_value(v[key].clone()).map_err(err)
}
#[derive(Deserialize)]
struct Member {
    member: u32,
    identity: Vec<u8>,
    bls_key: Vec<u8>,
    pop: Vec<u8>,
    signer_index: u32,
}
pub struct Oracle {
    scheme: Option<S>,
    epoch: u64,
}
impl Oracle {
    pub fn new() -> Self {
        Self {
            scheme: None,
            epoch: 0,
        }
    }
    pub fn dispatch(&mut self, q: &Value) -> R<Value> {
        match q["op"].as_str().ok_or("missing operation")? {
            "init" => {
                let members: Vec<Member> =
                    serde_json::from_value(q["members"].clone()).map_err(err)?;
                need(members.len() == 5, "registry length")?;
                let mut keys = Vec::new();
                let mut fields = Vec::new();
                for (i, m) in members.iter().enumerate() {
                    need(m.member == i as u32, "registry member order")?;
                    let key = PublicKey::decode(Copying(&m.identity)).map_err(err)?;
                    let bls =
                        <MinSig as Variant>::Public::decode(Copying(&m.bls_key)).map_err(err)?;
                    let pop =
                        <MinSig as Variant>::Signature::decode(Copying(&m.pop)).map_err(err)?;
                    ops::verify_proof_of_possession::<MinSig>(&bls, b"I1b/storage/pop/v1", &pop)
                        .map_err(err)?;
                    fields.extend([
                        m.member.to_be_bytes().to_vec(),
                        m.identity.clone(),
                        m.bls_key.clone(),
                        m.pop.clone(),
                    ]);
                    keys.push((key, bls));
                }
                let registry = BiMap::try_from(keys).map_err(err)?;
                for m in &members {
                    need(
                        registry
                            .keys()
                            .get(m.signer_index as usize)
                            .is_some_and(|k| k.encode().as_ref() == m.identity),
                        "signer index map",
                    )?;
                }
                let ns = bytes(q, "namespace")?;
                need(ns.len() == 32, "namespace length")?;
                self.epoch = q["epoch"].as_u64().ok_or("epoch")?;
                self.scheme = Some(S::verifier(&ns, registry));
                Ok(
                    json!({"registry_hash":hash("I1b/voter-registry/v1",&fields.iter().map(Vec::as_slice).collect::<Vec<_>>()),"quorum":4}),
                )
            }
            "vote" => {
                let raw = bytes(q, "bytes")?;
                self.vote(&raw)
            }
            "certificate" => {
                let raw = bytes(q, "bytes")?;
                self.certificate(&raw)
            }
            "finalization" => {
                let raw = bytes(q, "bytes")?;
                let scheme = self.scheme.as_ref().ok_or("uninitialized")?;
                let f = Finalization::<S, Digest>::decode_cfg(
                    Copying(&raw),
                    &scheme.certificate_codec_config(),
                )
                .map_err(err)?;
                need(f.encode().as_ref() == raw, "noncanonical finalization")?;
                let mut out = self.certificate(&C::Finalization(f).encode())?;
                out["inner_bytes"] = json!(raw);
                Ok(out)
            }
            "attestation" => {
                let scheme = self.scheme.as_ref().ok_or("uninitialized")?;
                let raw = bytes(q, "bytes")?;
                let subject = bytes(q, "subject")?;
                let a = Attestation::<S>::decode(Copying(&raw)).map_err(err)?;
                need(a.encode().as_ref() == raw, "noncanonical attestation")?;
                let mut rng = ChaCha20Rng::from_seed([107; 32]);
                let kind = q["kind"].as_u64().ok_or("sign kind")?;
                let (valid, epoch, view, parent, payload) = if kind == 2 {
                    let round = Round::decode(Copying(&subject)).map_err(err)?;
                    need(round.encode().as_ref() == subject, "noncanonical round")?;
                    (
                        scheme.verify_attestation::<_, Digest>(
                            &mut rng,
                            Subject::Nullify { round },
                            &a,
                            &Sequential,
                        ),
                        round.epoch().get(),
                        round.view().get(),
                        None,
                        None,
                    )
                } else {
                    let proposal = Proposal::<Digest>::decode(Copying(&subject)).map_err(err)?;
                    need(
                        proposal.encode().as_ref() == subject,
                        "noncanonical proposal",
                    )?;
                    let sub = match kind {
                        1 => Subject::Notarize {
                            proposal: &proposal,
                        },
                        3 => Subject::Finalize {
                            proposal: &proposal,
                        },
                        _ => return Err("bad sign kind".into()),
                    };
                    (
                        scheme.verify_attestation(&mut rng, sub, &a, &Sequential),
                        proposal.round.epoch().get(),
                        proposal.round.view().get(),
                        Some(proposal.parent.get()),
                        Some(proposal.payload.0),
                    )
                };
                Ok(
                    json!({"valid":valid&&epoch==self.epoch,"kind":kind,"epoch":epoch,"view":view,"parent_view":parent,"payload":payload,"signer_index":a.signer.get()}),
                )
            }
            "artifact" => {
                let scheme = self.scheme.as_ref().ok_or("uninitialized")?;
                let raw = bytes(q, "bytes")?;
                let a = A::decode_cfg(Copying(&raw), &scheme.certificate_codec_config())
                    .map_err(err)?;
                need(a.encode().as_ref() == raw, "noncanonical artifact")?;
                let epoch = a.epoch().get();
                let view = a.view().get();
                let wire = match a {
                    A::Notarize(v) => Some((1, V::Notarize(v).encode().to_vec())),
                    A::Nullify(v) => Some((1, V::Nullify(v).encode().to_vec())),
                    A::Finalize(v) => Some((1, V::Finalize(v).encode().to_vec())),
                    A::Notarization(v) => Some((2, C::Notarization(v).encode().to_vec())),
                    A::Nullification(v) => Some((2, C::Nullification(v).encode().to_vec())),
                    A::Finalization(v) => Some((2, C::Finalization(v).encode().to_vec())),
                    A::Certification(..) => None,
                };
                let proof = match &wire {
                    Some((1, b)) => self.vote(b)?,
                    Some((2, b)) => self.certificate(b)?,
                    _ => json!({"valid":epoch==self.epoch}),
                };
                Ok(
                    json!({"epoch":epoch,"view":view,"valid":proof["valid"],"wire":wire.map(|(channel,bytes)|json!({"channel":channel,"bytes":bytes})),"proof":proof}),
                )
            }
            "body" => {
                let raw = bytes(q, "bytes")?;
                let v = p::BodyPacket::decode(raw.as_slice()).map_err(err)?;
                Ok(serde_json::to_value(v).map_err(err)?)
            }
            "proto" => match q["type"].as_str().ok_or("proto type")? {
                "grant" => {
                    let mut v: p::Grant =
                        serde_json::from_value(q["value"].clone()).map_err(err)?;
                    v.grant_hash.clear();
                    let bytes = v.encode_to_vec();
                    Ok(json!({"bytes":bytes,"hash":hash("I1b/fresh-grant/v1",&[&bytes])}))
                }
                "ack" => {
                    let mut v: p::Ack = serde_json::from_value(q["value"].clone()).map_err(err)?;
                    v.logical_ack_hash.clear();
                    v.durable_step_hash.clear();
                    let bytes = v.encode_to_vec();
                    Ok(json!({"bytes":bytes,"hash":hash("I1b/fresh-ack/v1",&[&bytes])}))
                }
                _ => Err("unknown proto hash type".into()),
            },
            "frame-encode" => {
                let encoded = match q["direction"].as_u64() {
                    Some(0) => serde_json::from_value::<p::HubFrame>(q["value"].clone())
                        .map_err(err)?
                        .encode_to_vec(),
                    Some(1) => serde_json::from_value::<p::WorkerFrame>(q["value"].clone())
                        .map_err(err)?
                        .encode_to_vec(),
                    _ => return Err("frame direction".into()),
                };
                Ok(json!(encoded))
            }
            "frames" => {
                let raw = std::fs::read(q["path"].as_str().ok_or("frame path")?).map_err(err)?;
                let mut at = 0;
                let mut rows = Vec::new();
                while at < raw.len() {
                    need(at + 10 <= raw.len(), "partial frame header")?;
                    let direction = raw[at];
                    let worker = raw[at + 1];
                    let len = u64::from_be_bytes(raw[at + 2..at + 10].try_into().unwrap()) as usize;
                    at += 10;
                    need(
                        len <= 1048576 && at + len <= raw.len(),
                        "invalid frame size",
                    )?;
                    let body = &raw[at..at + len];
                    let value = match direction {
                        0 => serde_json::to_value(p::HubFrame::decode(body).map_err(err)?)
                            .map_err(err)?,
                        1 => serde_json::to_value(p::WorkerFrame::decode(body).map_err(err)?)
                            .map_err(err)?,
                        _ => return Err("frame direction".into()),
                    };
                    rows.push(json!({"direction":direction,"worker":worker,"frame":value}));
                    at += len;
                }
                Ok(json!(rows))
            }
            _ => Err("unknown oracle operation".into()),
        }
    }
    fn vote(&self, raw: &[u8]) -> R<Value> {
        let scheme = self.scheme.as_ref().ok_or("uninitialized")?;
        let v = V::decode_cfg(Copying(raw), &()).map_err(err)?;
        need(v.encode().as_ref() == raw, "noncanonical vote")?;
        let mut rng = ChaCha20Rng::from_seed([109; 32]);
        let valid = match &v {
            V::Notarize(v) => v.verify(&mut rng, scheme, &Sequential),
            V::Nullify(v) => v.verify::<_, Digest>(&mut rng, scheme, &Sequential),
            V::Finalize(v) => v.verify(&mut rng, scheme, &Sequential),
        };
        let epoch = v.epoch().get();
        let view = v.view().get();
        let (kind, subject, attestation, parent, payload) = match v {
            V::Notarize(v) => (
                1,
                v.proposal.encode().to_vec(),
                v.attestation.encode().to_vec(),
                Some(v.proposal.parent.get()),
                Some(v.proposal.payload.0),
            ),
            V::Nullify(v) => (
                2,
                v.round.encode().to_vec(),
                v.attestation.encode().to_vec(),
                None,
                None,
            ),
            V::Finalize(v) => (
                3,
                v.proposal.encode().to_vec(),
                v.attestation.encode().to_vec(),
                Some(v.proposal.parent.get()),
                Some(v.proposal.payload.0),
            ),
        };
        let a = Attestation::<S>::decode(Copying(&attestation)).map_err(err)?;
        Ok(
            json!({"valid":valid&&epoch==self.epoch,"kind":kind,"epoch":epoch,"view":view,"subject":subject,"attestation":attestation,"signer_index":a.signer.get(),"parent_view":parent,"payload":payload}),
        )
    }
    fn certificate(&self, raw: &[u8]) -> R<Value> {
        let scheme = self.scheme.as_ref().ok_or("uninitialized")?;
        let c = C::decode_cfg(Copying(raw), &scheme.certificate_codec_config()).map_err(err)?;
        need(c.encode().as_ref() == raw, "noncanonical certificate")?;
        let mut rng = ChaCha20Rng::from_seed([113; 32]);
        let valid = c.verify(&mut rng, scheme, &Sequential);
        let epoch = c.epoch().get();
        let view = c.view().get();
        let (kind, signers, parent, payload) = match c {
            C::Notarization(c) => (
                1,
                c.certificate
                    .signers
                    .iter()
                    .map(|s| s.get())
                    .collect::<Vec<_>>(),
                Some(c.proposal.parent.get()),
                Some(c.proposal.payload.0),
            ),
            C::Nullification(c) => (
                2,
                c.certificate
                    .signers
                    .iter()
                    .map(|s| s.get())
                    .collect::<Vec<_>>(),
                None,
                None,
            ),
            C::Finalization(c) => (
                3,
                c.certificate
                    .signers
                    .iter()
                    .map(|s| s.get())
                    .collect::<Vec<_>>(),
                Some(c.proposal.parent.get()),
                Some(c.proposal.payload.0),
            ),
        };
        Ok(
            json!({"valid":valid&&epoch==self.epoch,"kind":kind,"epoch":epoch,"view":view,"signers":signers,"parent_view":parent,"payload":payload}),
        )
    }
}
async fn fake_worker(mode: &str, endpoint: &str, launch: &str, original: &str) -> R<()> {
    use p::{effect, hub_frame, input, native_run_client::NativeRunClient, worker_frame};
    let launch_value: Value =
        serde_json::from_slice(&std::fs::read(launch).map_err(err)?).map_err(err)?;
    let node = launch_value["member"].as_u64().ok_or("member")? as u32;
    if node != 0 {
        use std::os::unix::process::CommandExt;
        let error = std::process::Command::new(original)
            .args([endpoint, launch])
            .exec();
        return Err(err(error));
    }
    let c = &launch_value["resolved"];
    let binding = p::Binding {
        schema_version: 1,
        run_id: bytes(c, "run_id")?,
        config_hash: bytes(c, "config_hash")?,
        worker_id: node,
        member_id: node,
        incarnation: 1,
    };
    let hello = p::RegisterRequest {
        binding: Some(binding.clone()),
        registry_hash: bytes(&c["base"], "registry_hash")?,
        namespace_hash: bytes(c, "namespace")?,
        profile: "native-engine-fresh-v1".into(),
        recovery: None,
    };
    let mut client = NativeRunClient::connect(endpoint.to_owned())
        .await
        .map_err(err)?
        .max_decoding_message_size(1048576)
        .max_encoding_message_size(1049600);
    client.register(hello.clone()).await.map_err(err)?;
    let (tx, rx) = tokio::sync::mpsc::channel(4);
    tx.send(p::WorkerFrame {
        frame: Some(worker_frame::Frame::Hello(hello)),
    })
    .await
    .map_err(err)?;
    let mut stream = client
        .run(tokio_stream::wrappers::ReceiverStream::new(rx))
        .await
        .map_err(err)?
        .into_inner();
    let mut sequence = 0;
    let mut request = 0;
    if mode == "unsolicited" {
        let a = p::Ack {
            binding: Some(binding.clone()),
            logical_step_id: 999,
            status: 1,
            ..Default::default()
        };
        tx.send(p::WorkerFrame {
            frame: Some(worker_frame::Frame::Ack(a)),
        })
        .await
        .map_err(err)?;
    }
    while let Some(frame) = stream.message().await.map_err(err)? {
        let grant = match frame.frame {
            Some(hub_frame::Frame::Shutdown(_)) => return Ok(()),
            Some(hub_frame::Frame::Grant(g)) => g,
            _ => return Err("empty frame".into()),
        };
        sequence += 1;
        let mut effects = vec![];
        if matches!(
            grant.input.as_ref().and_then(|i| i.kind.as_ref()),
            Some(input::Kind::RequestSample(_))
        ) {
            request += 1;
            effects.push(p::Effect {
                kind: Some(effect::Kind::ClockRequest(p::ClockRequest {
                    request_id: request,
                })),
            });
        }
        if mode == "invalid-atomic" && sequence == 1 {
            effects.push(p::Effect {
                kind: Some(effect::Kind::TimerArm(p::Timer {
                    id: 0,
                    cid: bytes(c, "cid")?,
                    local_deadline_us: 1000000,
                })),
            });
        }
        if sequence == 1 && matches!(mode, "payload-exact" | "payload-over") {
            let cid = bytes(c, "cid")?;
            let content = hash("independent/body-limit/v1", &[&cid]);
            let request = p::BodyPacket {
                kind: Some(p::body_packet::Kind::Request(p::BodyRequest {
                    request_id: 1,
                    cid: cid.clone(),
                    wanted_digest: content.to_vec(),
                })),
            };
            let mut payload = request.encode_to_vec();
            let target = 65536 + usize::from(mode == "payload-over");
            let pad = target - payload.len() - 5;
            payload.extend_from_slice(&[0x9a, 0x06]);
            let mut length = pad as u32;
            while length >= 128 {
                payload.push((length as u8 & 127) | 128);
                length >>= 7;
            }
            payload.push(length as u8);
            payload.resize(target, 0);
            let payload_hash = hash("I1b/native-wire/v1", &[&4u32.to_be_bytes(), &payload]);
            let message_id = hash(
                "I1b/native-message/v1",
                &[
                    &binding.run_id,
                    &binding.config_hash,
                    &cid,
                    &1u64.to_be_bytes(),
                    &0u32.to_be_bytes(),
                    &1u32.to_be_bytes(),
                    &4u32.to_be_bytes(),
                    &payload_hash,
                ],
            );
            effects.push(p::Effect {
                kind: Some(effect::Kind::Send(p::Envelope {
                    run_id: binding.run_id.clone(),
                    cid,
                    epoch: 1,
                    src_member_id: 0,
                    dst_member_id: 1,
                    channel: 4,
                    message_id: message_id.to_vec(),
                    payload_hash: payload_hash.to_vec(),
                    payload_bytes: payload,
                    attempt: 1,
                    priority: false,
                    provenance: Some(p::envelope::Provenance::Body(p::BodyProvenance {
                        content_id: content.to_vec(),
                        request_id: 1,
                    })),
                })),
            });
        }
        let mut ack = p::Ack {
            binding: Some(binding.clone()),
            logical_step_id: grant.logical_step_id,
            event_id: grant.event_id,
            grant_hash: grant.grant_hash.clone(),
            status: 1,
            effects,
            runtime_events_json: b"[]".to_vec(),
            application_events_json: b"{}".to_vec(),
            witness_receipt: Some(p::Receipt {
                sequence: 1,
                hash: vec![0; 32],
            }),
            durable_step_sequence: sequence,
            durable_step_hash: vec![0; 32],
            logical_ack_hash: vec![],
            recovery_receipt: None,
        };
        if sequence == 1 && matches!(mode, "frame-exact" | "frame-over") {
            let target = 1048576 + usize::from(mode == "frame-over");
            ack.logical_ack_hash = vec![0; 32];
            ack.runtime_events_json = vec![b'x'; target];
            loop {
                let frame = p::WorkerFrame {
                    frame: Some(worker_frame::Frame::Ack(ack.clone())),
                };
                let n = frame.encoded_len();
                if n == target {
                    break;
                }
                let next = (ack.runtime_events_json.len() as isize + target as isize - n as isize)
                    as usize;
                ack.runtime_events_json.resize(next, b'x');
            }
            let n = ack.runtime_events_json.len();
            ack.runtime_events_json[0] = b'"';
            ack.runtime_events_json[n - 1] = b'"';
        }
        let mut logical = ack.clone();
        logical.logical_ack_hash.clear();
        logical.durable_step_hash.clear();
        ack.logical_ack_hash = hash("I1b/fresh-ack/v1", &[&logical.encode_to_vec()]).to_vec();
        tx.send(p::WorkerFrame {
            frame: Some(worker_frame::Frame::Ack(ack.clone())),
        })
        .await
        .map_err(err)?;
        if mode == "exact-duplicate" {
            tx.send(p::WorkerFrame {
                frame: Some(worker_frame::Frame::Ack(ack.clone())),
            })
            .await
            .map_err(err)?;
        }
        if mode == "altered-duplicate" && sequence == 1 {
            ack.application_events_json = b"[]".to_vec();
            let mut logical = ack.clone();
            logical.logical_ack_hash.clear();
            logical.durable_step_hash.clear();
            ack.logical_ack_hash = hash("I1b/fresh-ack/v1", &[&logical.encode_to_vec()]).to_vec();
            tx.send(p::WorkerFrame {
                frame: Some(worker_frame::Frame::Ack(ack)),
            })
            .await
            .map_err(err)?;
        }
    }
    Err("stream ended".into())
}
fn main() {
    let args: Vec<_> = std::env::args().collect();
    if args.get(1).is_some_and(|s| s == "drive-worker") {
        let result = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(control::run(&args[2], &args[3], &args[4], &args[5]));
        if let Err(error) = result {
            eprintln!("verifier authority: {error}");
            std::process::exit(1);
        }
        return;
    }
    if args.get(1).is_some_and(|s| s == "fake-worker") {
        let result = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(fake_worker(&args[2], &args[3], &args[4], &args[5]));
        if let Err(error) = result {
            eprintln!("verifier fixture: {error}");
            std::process::exit(1);
        }
        return;
    }
    let mut oracle = Oracle::new();
    let stdin = io::stdin();
    let mut out = io::BufWriter::new(io::stdout());
    for line in stdin.lock().lines() {
        let result = line
            .map_err(err)
            .and_then(|s| serde_json::from_str::<Value>(&s).map_err(err))
            .and_then(|q| oracle.dispatch(&q));
        let value = match result {
            Ok(v) => json!({"ok":v}),
            Err(e) => json!({"error":e}),
        };
        writeln!(out, "{}", value).unwrap();
        out.flush().unwrap();
    }
}
