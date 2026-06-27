"""
Download ML model weights for a Re:Trace backend.

Protocol (stderr lines):
  progress:start:TOTAL      — total number of files to download
  progress:file:N           — file N just completed
  progress:done             — all files done

Usage:
  python download_model.py --backend (starvector-1b|starvector-8b) --model-dir /path/to/dir
"""

from __future__ import annotations
import argparse
import sys
import os

MODEL_IDS = {
    "starvector-1b": "StarVector/starvector-1b-svg",
    "starvector-8b": "StarVector/starvector-8b-svg",
}

# File patterns to skip (not needed for inference; saves bandwidth).
SKIP_PREFIXES = ("flax_model", "tf_model", "rust_model", "onnx")
SKIP_EXTENSIONS = (".msgpack", ".ot", ".h5")


def _should_skip(filename: str) -> bool:
    return any(filename.startswith(p) for p in SKIP_PREFIXES) or any(
        filename.endswith(e) for e in SKIP_EXTENSIONS
    )


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--backend", required=True)
    ap.add_argument("--model-dir", required=True)
    args = ap.parse_args()

    repo_id = MODEL_IDS.get(args.backend)
    if not repo_id:
        print(f"Unknown backend: {args.backend!r}", file=sys.stderr)
        sys.exit(1)

    model_dir = args.model_dir
    os.makedirs(model_dir, exist_ok=True)

    try:
        from huggingface_hub import list_repo_files, hf_hub_download
    except ImportError:
        print(
            "huggingface_hub not installed — install the Python environment first",
            file=sys.stderr,
        )
        sys.exit(1)

    all_files = list(list_repo_files(repo_id))
    files = [f for f in all_files if not _should_skip(f)]

    total = len(files)
    print(f"progress:start:{total}", file=sys.stderr, flush=True)

    for i, filename in enumerate(files):
        hf_hub_download(
            repo_id=repo_id,
            filename=filename,
            local_dir=model_dir,
        )
        print(f"progress:file:{i + 1}", file=sys.stderr, flush=True)

    print("progress:done", file=sys.stderr, flush=True)


main()
