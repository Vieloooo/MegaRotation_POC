use std::{env, ffi::OsString, fs, io, path::Path, process::ExitCode, time::Duration};

use base64::{Engine, engine::general_purpose::STANDARD};
use espresso_compatibility::{CaptureBundle, FIXTURE_CODEC, ReplaySession, sha256_hex};
use serde_json::{Value, json};

fn main() -> ExitCode {
    if let Err(error) = run() {
        eprintln!("espresso-compatibility: {error}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<OsString> = env::args_os().skip(1).collect();
    match args.as_slice() {
        [command, bundle, reader, output] if command == "replay" => {
            replay(Path::new(bundle), reader_id(reader)?, Path::new(output), 0)?;
        }
        [command, bundle, reader, output, flag, ms]
            if command == "replay" && flag == "--pause-ms" =>
        {
            let pause = reader_id(ms)?.parse::<u64>()?;
            if pause > 10_000 {
                return Err(io::Error::other("replay pause must be at most 10000 ms").into());
            }
            replay(
                Path::new(bundle),
                reader_id(reader)?,
                Path::new(output),
                pause,
            )?;
        }
        [command, bundle, left, right] if command == "compare" => {
            let capture = CaptureBundle::open(Path::new(bundle))?;
            let left = capture.replay(reader_id(left)?)?;
            let right = capture.replay(reader_id(right)?)?;
            left.reader.compare_public_objects(&right.reader)?;
            let sha256 = left.reader.public_sha256(FIXTURE_CODEC)?;
            if sha256 != right.reader.public_sha256(FIXTURE_CODEC)? {
                return Err(io::Error::other("public audit digests differ").into());
            }
            println!(
                "{}",
                json!({
                    "source_id": capture.binding().source_id(),
                    "complete_blocks": left.reader.prefix().blocks().len(),
                    "capture_kind": capture.binding().capture_kind(),
                    "public_sha256": sha256,
                })
            );
        }
        _ => {
            return Err(io::Error::other(
                "usage: espresso-compatibility replay BUNDLE READER NEW_OUTPUT [--pause-ms N]\n       espresso-compatibility compare BUNDLE READER REQUERY_READER",
            )
            .into());
        }
    }
    Ok(())
}

fn reader_id(value: &OsString) -> Result<&str, io::Error> {
    value
        .to_str()
        .ok_or_else(|| io::Error::other("expected UTF-8 argument"))
}

fn replay(
    bundle: &Path,
    reader_id: &str,
    output: &Path,
    pause_ms: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    let capture = CaptureBundle::open(bundle)?;
    std::thread::sleep(Duration::from_millis(pause_ms));
    let session = capture.replay(reader_id)?;
    let public = session.reader.public_audit_bytes(FIXTURE_CODEC)?;
    let public_sha256 = sha256_hex(&public);
    let projection = projection(&session, &public_sha256)?;
    let binding = capture.binding();
    let receipt = json!({
        "capture_kind": binding.capture_kind(),
        "capture_manifest_sha256": binding.capture_manifest_sha256(),
        "reader_id": session.reader.reader_id(),
        "public_sha256": public_sha256,
        "wait": session.reader.wait().to_string(),
        "unavailable_records": session.unavailable_records,
        "rejected_submissions": session.rejected_submissions,
        "publications": session.publications,
        "pe_log_unavailable": session.reader.export_pe_log().unwrap_err().missing,
        "pe_reader_unavailable": session.reader.export_pe_reader().err().expect("missing timing").missing,
    });
    fs::create_dir(output)?;
    fs::write(output.join("public-audit.bin"), public)?;
    write_json(output.join("public-projection.json").as_path(), &projection)?;
    write_json(output.join("receipts.json").as_path(), &receipt)?;
    println!("{public_sha256}");
    Ok(())
}

fn projection(
    session: &ReplaySession,
    sha256: &str,
) -> Result<Value, espresso_compatibility::Error> {
    let reader = &session.reader;
    let mut blocks = Vec::new();
    for block in reader.prefix().blocks() {
        let native: Vec<_> = block
            .ordered_transactions
            .iter()
            .map(|tx| {
                json!({
                    "source_position": tx.position,
                    "namespace": tx.namespace,
                    "pos_in_namespace": tx.pos_in_namespace,
                    "transaction_hash": tx.transaction_hash,
                    "payload_base64": STANDARD.encode(&tx.payload),
                })
            })
            .collect();
        let selected: Vec<_> = reader
            .project_publication_messages(block.q, FIXTURE_CODEC)?
            .into_iter()
            .map(|item| {
                json!({
                    "message_index": item.position.message_index,
                    "source_position": item.source_position,
                    "codec": item.message.codec,
                    "payload_base64": STANDARD.encode(&item.message.bytes),
                })
            })
            .collect();
        blocks.push(json!({
            "q": block.q,
            "theta_us": block.theta.0,
            "block_hash": block.block_hash,
            "native_transactions": native,
            "publication_messages": selected,
            "timing": block.timing,
        }));
    }
    Ok(json!({
        "schema": "EC16-AUDIT-1",
        "source_id": reader.binding().source_id(),
        "public_sha256": sha256,
        "blocks": blocks,
    }))
}

fn write_json(path: &Path, value: &Value) -> Result<(), Box<dyn std::error::Error>> {
    let mut bytes = serde_json::to_vec_pretty(value)?;
    bytes.push(b'\n');
    fs::write(path, bytes)?;
    Ok(())
}
