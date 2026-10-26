#!/usr/bin/env python3
"""Check first-party Rust with the pinned Rust grammar, including crate roots."""

import json
from pathlib import Path
import subprocess


def main():
    root = Path(__file__).resolve().parent.parent
    metadata = subprocess.check_output(
        ["cargo", "metadata", "--no-deps", "--format-version", "1"], cwd=root
    )
    package = json.loads(metadata)["packages"][0]
    roots = {Path(target["src_path"]) for target in package["targets"]}
    files = {path for folder in ("src", "tests", "examples", "benches")
             for path in (root / folder).rglob("*.rs")}
    files.update(roots)
    command = ["cargo", "run", "--quiet", "--example", "source-check", "--"]
    for path in sorted(files):
        command.extend(["--root" if path in roots else "--source", str(path)])
    subprocess.run(command, cwd=root, check=True)


if __name__ == "__main__":
    main()
