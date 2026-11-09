#!/usr/bin/env python3
"""Check first-party Rust with the pinned Rust grammar, including crate roots."""

from pathlib import Path
import subprocess


def main():
    root = Path(__file__).resolve().parent.parent
    command = ["cargo", "run", "--locked", "--quiet", "-p", "rev",
               "--example", "source-check", "--", "--workspace"]
    subprocess.run(command, cwd=root, check=True)


if __name__ == "__main__":
    main()
