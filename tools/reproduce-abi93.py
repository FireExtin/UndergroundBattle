#!/usr/bin/env python3
"""Reproduce the five pinned ABI93 files from the authorized bb77 source.

This recipe never reads the historical /workspace path, an old binary, or a
credential. The old Cargo prefix is an explicit compiler path-map destination.
Run with a new normal-disk output directory and the existing storage guard.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess

SOURCE = "bb77eb8ad69738ebecad860d60df05a9276efd00"
RUSTC = "rustc 1.90.0 (1159e78c4 2025-09-14)"
NORMALIZED_CARGO = "/workspace/audit-underground-100161b/cargo"
EXPECTED = {
    "hegemony_wasm_bg.wasm": "098986aff958d97b5fb5f49339daf07586a200997bf49c179f424ea51dd04e7e",
    "hegemony_wasm.js": "fb262978c41802322caec9fa9dd38ff6e273ff019865fd401a82afd78c1cfebb",
    "hegemony_wasm.d.ts": "c957ff06db12352cfc9ebb92a45f1707432e53972efcbee7df24c8b9c211a758",
    "hegemony_wasm_bg.wasm.d.ts": "5ebdfb8ff652624f96dfba3d1dcecf1ec3db65c65846a3579ad1aa94372dbdf3",
    "package.json": "89c47db7851f89caec72da4f215ebc21cf1e294aeef3b45d2583562d846ab035",
}


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ["source", "output", "node", "cargo", "bindgen", "cargo-home", "rustup-home", "cc"]:
        parser.add_argument("--" + name, required=True, type=Path)
    args = parser.parse_args()
    source = args.source.resolve()
    output = args.output.resolve()
    # Rustup selects the tool from argv[0]; keep the cargo/rustc proxy names.
    # Resolving cargo's symlink would invoke rustup itself with `build`.
    cargo = args.cargo.absolute()
    node = args.node.absolute()
    bindgen_path = args.bindgen.absolute()
    cc = args.cc.absolute()
    if output.exists() or output.is_relative_to(source) or source.is_relative_to(output):
        raise SystemExit("Use a new output directory outside the source; nothing overwritten.")
    git = lambda *a: subprocess.check_output(["git", *a], cwd=source, text=True).strip()
    if git("rev-parse", "HEAD") != SOURCE or git("status", "--porcelain"):
        raise SystemExit("The recipe requires the clean, fixed bb77 source checkout.")
    env = dict(os.environ)
    env.update({
        "PATH": str(cargo.parent) + ":" + str(node.parent) + ":" + env.get("PATH", ""),
        "CARGO_HOME": str(args.cargo_home.resolve()),
        "RUSTUP_HOME": str(args.rustup_home.resolve()),
        "RUSTUP_TOOLCHAIN": "1.90.0",
        "CC": str(cc),
        "CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER": str(cc),
        "CARGO_TARGET_DIR": str(output / "build/target"),
        "UNDERGROUNDBATTLE_BUILD_ROOT": str(output / "build"),
        "RUSTFLAGS": "--remap-path-prefix=" + str(args.cargo_home.resolve()) + "=" + NORMALIZED_CARGO,
    })
    rustc = subprocess.check_output([str(cargo.parent / "rustc"), "--version"], env=env, text=True).strip()
    bindgen = subprocess.check_output([str(bindgen_path), "--version"], env=env, text=True).strip()
    if rustc != RUSTC or bindgen != "wasm-bindgen 0.2.104":
        raise SystemExit("Use Rust 1.90.0 and the locked crates.io 0.2.104 CLI without a Git suffix.")
    output.mkdir(parents=True)
    package = output / "pkg"
    # The guard enforces normal disk, >=1GiB headroom and one Cargo job.
    command = [str(node), str(source / "tools/build-storage.mjs"),
               "--output-dir", str(package), "--", str(cargo), "build",
               "--offline", "--locked", "--release", "-p", "hegemony-wasm",
               "--target", "wasm32-unknown-unknown"]
    subprocess.run(command, cwd=source, env=env, check=True)
    raw = output / "build/target/wasm32-unknown-unknown/release/hegemony_wasm.wasm"
    subprocess.run([str(bindgen_path), str(raw), "--target", "web", "--out-dir",
                    str(package), "--out-name", "hegemony_wasm"], cwd=source, env=env, check=True)
    (package / "package.json").write_text('{"type":"module","private":true}\n')
    files = [{"file": name, "bytes": (package / name).stat().st_size,
              "sha256": sha(package / name), "expectedSha256": expected}
             for name, expected in EXPECTED.items()]
    if any(f["sha256"] != f["expectedSha256"] for f in files):
        raise SystemExit("Rebuilt bytes differ; outputs retained without relabeling anything.")
    result = {"sourceCommit": SOURCE, "rustc": rustc, "bindgen": bindgen,
              "bindgenExecutableSha256": sha(bindgen_path),
              "cargoLockSha256": sha(source / "Cargo.lock"), "pathMap": env["RUSTFLAGS"],
              "rawWasmSha256": sha(raw), "files": files, "allFiveFilesByteIdentical": True,
              "oldBinaryUsedAsInput": False, "directMetadataRewriting": False,
              "sourceUnchanged": git("rev-parse", "HEAD") == SOURCE and not git("status", "--porcelain"),
              "scope": "Author execution of this explicit ABI93 recipe; independent review remains pending."}
    (output / "reproduction.json").write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps(result))


if __name__ == "__main__":
    main()
