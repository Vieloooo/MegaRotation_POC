#!/usr/bin/env python3
"""Verify or acquire the pinned official GHCR image pieces used for native launch."""

import argparse
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import shutil
import tarfile
import tempfile
import urllib.parse
import urllib.request

REPOSITORY = "espressosystems/espresso-network/espresso-dev-node"
COMMIT = "fd221e8e4b51f5ecec0ae44bc99f6b7f82cd1171"
INDEX = "sha256:af73f55630690caa35aa0bc015d60dc03333791f966fdcc07b0798e33d4f768e"
MANIFEST = "sha256:cfb845e49cb5369f77cd8bbda0b4fe4aad340793cfd130b7f2bb631e59d2db66"
METADATA_FILES = ("index.json", "manifest-amd64.json", "config.json")
LIBS = (
    "ld-linux-x86-64.so.2", "libc.so.6", "libcrypto.so.3",
    "libgcc_s.so.1", "libm.so.6", "libssl.so.3",
)
SRS = "kzg10-aztec20-srs-1048584.bin"
CONTENTS = {
    0: {f"usr/lib/x86_64-linux-gnu/{name}": f"lib/{name}" for name in LIBS},
    2: {SRS: SRS},
    4: {"usr/bin/espresso-dev-node": "espresso-dev-node"},
    5: {"usr/bin/anvil": "anvil"},
}


def digest_stream(stream):
    digest = hashlib.sha256()
    for chunk in iter(lambda: stream.read(1024 * 1024), b""):
        digest.update(chunk)
    return "sha256:" + digest.hexdigest()


def digest_file(path):
    if not path.is_file() or path.is_symlink():
        raise ValueError(f"expected regular image file {path}")
    with path.open("rb") as stream:
        return digest_stream(stream)


class SecureRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, request, fp, code, msg, headers, newurl):
        parsed = urllib.parse.urlsplit(newurl)
        if parsed.scheme != "https":
            raise ValueError("registry redirect is not HTTPS")
        redirect = super().redirect_request(request, fp, code, msg, headers, newurl)
        if redirect and parsed.hostname != "ghcr.io":
            redirect.remove_header("Authorization")
        return redirect


class Registry:
    def __init__(self):
        self.opener = urllib.request.build_opener(SecureRedirect())
        query = urllib.parse.urlencode({
            "service": "ghcr.io", "scope": f"repository:{REPOSITORY}:pull",
        })
        with self.opener.open("https://ghcr.io/token?" + query, timeout=30) as reply:
            self.token = json.load(reply)["token"]

    def get(self, kind, digest, accept):
        request = urllib.request.Request(
            f"https://ghcr.io/v2/{REPOSITORY}/{kind}/{digest}",
            headers={"Accept": accept, "Authorization": "Bearer " + self.token},
        )
        return self.opener.open(request, timeout=60)


def receive(path, expected_digest, size, opener):
    """Verify first, then install a new cache file. Existing files are immutable."""
    if path.exists():
        if (size is not None and path.stat().st_size != size) or digest_file(path) != expected_digest:
            raise ValueError(f"image digest or size mismatch for {path}")
        return
    if opener is None:
        raise FileNotFoundError(f"missing {path}, run prepare-image.py prepare")
    limit = size if size is not None else 1024 * 1024
    with opener() as response, tempfile.NamedTemporaryFile(dir=path.parent) as out:
        length = 0
        while True:
            chunk = response.read(1024 * 1024)
            if not chunk:
                break
            length += len(chunk)
            if length > limit:
                raise ValueError(f"official image item is larger than expected: {path}")
            out.write(chunk)
        out.seek(0)
        if (size is not None and length != size) or digest_stream(out) != expected_digest:
            raise ValueError(f"official image item checksum failed: {path}")
        out.seek(0)
        with path.open("xb") as target:
            shutil.copyfileobj(out, target)


def populate_native(root, path, member_names, prepare):
    hashes = {}
    with tarfile.open(path, "r|gz") as archive:
        for member in archive:
            tar_name = str(PurePosixPath(member.name))
            if tar_name not in member_names:
                continue
            name = member_names[tar_name]
            if name in hashes or not member.isfile():
                raise ValueError(f"expected exactly one regular layer member {tar_name}")
            target = root / "native" / name
            with archive.extractfile(member) as body:
                if target.exists():
                    sha256 = digest_stream(body)
                    if digest_file(target) != sha256:
                        raise ValueError(f"native file differs from official image layer: {target}")
                elif prepare:
                    target.parent.mkdir(exist_ok=True, parents=True)
                    with target.open("xb") as out:
                        shutil.copyfileobj(body, out)
                    os.chmod(target, member.mode & 0o755)
                    sha256 = digest_file(target)
                else:
                    raise FileNotFoundError(f"missing {target}, run prepare-image.py prepare")
            hashes[name] = sha256
    if set(hashes) != set(member_names.values()):
        raise ValueError(f"required runtime member missing from {path}")
    return hashes


def image(root, prepare):
    if prepare:
        root.mkdir(parents=True, exist_ok=True)
    registry = None

    def fetch(kind, digest, accept):
        def open_response():
            nonlocal registry
            if registry is None:
                registry = Registry()
            return registry.get(kind, digest, accept)
        return open_response if prepare else None

    index_path, manifest_path, config_path = (root / name for name in METADATA_FILES)
    receive(index_path, INDEX, None, fetch("manifests", INDEX, "application/vnd.oci.image.index.v1+json"))
    index = json.loads(index_path.read_bytes())
    descriptor = next(
        item for item in index["manifests"]
        if item["digest"] == MANIFEST and item["platform"]["os"] == "linux"
        and item["platform"]["architecture"] == "amd64"
    )
    receive(
        manifest_path, MANIFEST, descriptor["size"],
        fetch("manifests", MANIFEST, "application/vnd.oci.image.manifest.v1+json"),
    )
    manifest = json.loads(manifest_path.read_bytes())
    config_digest = manifest["config"]["digest"]
    receive(
        config_path, config_digest, manifest["config"]["size"],
        fetch("blobs", config_digest, "application/octet-stream"),
    )
    config = json.loads(config_path.read_bytes())
    if config["config"]["Labels"]["org.opencontainers.image.revision"] != COMMIT:
        raise ValueError("official image source commit label differs from the pinned commit")

    layers = {}
    native = {}
    for position, targets in CONTENTS.items():
        layer = manifest["layers"][position]
        digest = layer["digest"]
        path = root / (digest.removeprefix("sha256:") + ".tar.gz")
        receive(path, digest, layer["size"], fetch("blobs", digest, "application/octet-stream"))
        layers[str(position)] = {"sha256": digest, "size": layer["size"]}
        native.update(populate_native(root, path, targets, prepare))
    return {
        "repository": REPOSITORY,
        "official_commit": COMMIT,
        "index": INDEX,
        "manifest_amd64": MANIFEST,
        "image_config": config_digest,
        "used_layers": layers,
        "native_sha256": native,
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("action", choices=["prepare", "verify"])
    parser.add_argument("cache", type=Path)
    args = parser.parse_args()
    info = image(args.cache.resolve(), args.action == "prepare")
    print(json.dumps(info, indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
