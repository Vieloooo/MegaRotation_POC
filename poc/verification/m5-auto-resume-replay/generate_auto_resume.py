#!/usr/bin/env python3
"""Render a self-contained M5 offline page from sample or sealed observations."""

import argparse
import json
import os
from pathlib import Path
from tempfile import NamedTemporaryFile

from model import project
from sample import scenarios
from sealed import load

PAGE = Path(__file__).resolve().parents[2] / "web" / "m1-gfr-replay"
if not PAGE.is_dir():
    PAGE = Path(__file__).resolve().parents[1] / "web"


def compact(dataset):
    """Intern repeated fields while retaining every timeline observation."""
    shared = ("duty", "pc", "d_position", "comparison", "blocked", "watermark", "stopped")
    signer_detail = ("published", "verified", "previous", "motion_basis", "pc", "heads",
                     "clock_sample", "d", "common_comparison", "guard_parts", "payload",
                     "growth_basis", "inclusion")
    pools = {"chains": [], "values": [], "refs": []}
    indexes = {field: {} for field in pools}

    def intern(field, value):
        key = json.dumps(value, sort_keys=True, separators=(",", ":"))
        index = indexes[field]
        if key not in index:
            index[key] = len(pools[field])
            pools[field].append(value)
        return index[key]

    views = {}
    frames = []
    for index, frame in enumerate(dataset["frames"]):
        data = {**frame, **{name: intern("values", frame[name]) for name in shared},
                "chains": {name: intern("chains", chain) for name, chain in frame["chains"].items()},
                "refs": [intern("refs", ref) for ref in frame["refs"]]}
        if frame["kind"] == "signer_view":
            who = frame["detail"]["signer"]
            view = frame["signer_views"][who]
            data["observation"] = {name: view[name] for name in ("q_advanced", "theta_advanced", "solver")}
            data["observation"]["solver"] = intern("values", data["observation"]["solver"])
            data["detail"] = {name: intern("values", value) if name in signer_detail else value
                              for name, value in frame["detail"].items()}
            views[who] = index
        data["signer_views"] = {who: views[who] for who in frame["signer_views"]}
        frames.append(data)
    return {**dataset, "frames":frames, "pools":pools, "shared":shared, "signer_detail":signer_detail}


def render(datasets):
    data = json.dumps([compact(case) for case in datasets], ensure_ascii=False, sort_keys=True, separators=(",", ":"))
    data = data.replace("<", "\\u003c").replace(">", "\\u003e").replace("&", "\\u0026")

    def read(name):
        return (PAGE / name).read_text(encoding="utf-8")

    css = "\n".join(read(name) for name in ("replay.css", "resume.css", "auto-resume.css"))
    return (read("auto-resume-template.html").replace("/* INLINE_CSS */", css)
            .replace("/* INLINE_JS */", read("auto-resume.js")).replace("INLINE_JSON", data)).encode("utf-8")


def publish(path, content):
    temporary = None
    try:
        with NamedTemporaryFile(dir=path.parent, prefix=".m5-replay-", delete=False) as stream:
            temporary = Path(stream.name)
            stream.write(content)
        temporary.chmod(0o644)
        os.replace(temporary, path)
    finally:
        if temporary is not None:
            temporary.unlink(missing_ok=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--sample", action="store_true")
    mode.add_argument("--bundle", type=Path, action="append")
    parser.add_argument("--independent", type=Path, action="append")
    parser.add_argument("--output", type=Path)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    if args.sample and args.independent or args.bundle and len(args.bundle) != len(args.independent or []):
        parser.error("provide one --independent DIR per --bundle DIR")
    output = args.output or PAGE / ("auto-resume-sample.html" if args.sample else "auto-resume.html")
    try:
        if args.sample:
            if "auto-resume.html" in (output.name, output.resolve().name):
                raise ValueError("sample must use its own output")
            datasets = [project(tape, "sample", {
                "baseline": "e44b6e8e77aeb1aa3895c14109a57ff3894d7252",
                "contract": "M5-AUTO-RESUME-1 · 712c018",
                "observations": "合同计划值、合成时点和标识、从样本输入复算的公开键与最小目标。",
            }) for tape in scenarios()]
        else:
            datasets = [load(bundle, independent) for bundle, independent in zip(args.bundle, args.independent)]
            if len({case["provenance"]["candidate"] for case in datasets}) != 1:
                raise ValueError("combined page requires one exact candidate")
        html = render(datasets)
        if args.check:
            if output.read_bytes() != html:
                raise ValueError("output differs from deterministic generation")
        else:
            publish(output, html)
    except (IndexError, KeyError, OSError, OverflowError, TypeError, ValueError) as error:
        parser.exit(1, f"M5 replay: {error}\n")
    print(f"PASS: M5 {datasets[0]['mode']} replay, {sum(len(case['frames']) for case in datasets)} frames · {output}")


if __name__ == "__main__":
    main()
