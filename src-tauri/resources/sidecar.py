#!/usr/bin/env python3
"""Re:Trace sidecar — subprocess entrypoint invoked by the Rust backend.

Called as: <venv>/bin/python3 sidecar.py --backend <id> --input <path.png>
Writes SVG to stdout; errors go to stderr; exits non-zero on failure.
"""
import argparse
import os
import sys


def main() -> None:
    parser = argparse.ArgumentParser(prog="sidecar")
    parser.add_argument(
        "--backend",
        required=True,
        choices=["live", "starvector-1b", "starvector-8b"],
    )
    parser.add_argument("--input", required=True)
    args = parser.parse_args()

    if not os.path.exists(args.input):
        print(f"input not found: {args.input}", file=sys.stderr)
        sys.exit(1)

    if args.backend == "live":
        from backends.live import trace
        svg = trace(args.input)
    elif args.backend in ("starvector-1b", "starvector-8b"):
        from backends.starvector import trace
        svg = trace(args.input, args.backend)
    else:
        print(f"unknown backend: {args.backend}", file=sys.stderr)
        sys.exit(1)

    sys.stdout.write(svg)


if __name__ == "__main__":
    main()
