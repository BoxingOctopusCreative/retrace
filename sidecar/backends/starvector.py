"""StarVector backend (1B and 8B models).

Requires model weights downloaded via Settings → Enhanced Backends in the Re:Trace app.
The Rust host sets RETRACE_MODEL_DIR to the directory containing the downloaded weights.
"""

from __future__ import annotations

import os
import sys

MODEL_IDS = {
    "starvector-1b": "StarVector/starvector-1b-svg",
    "starvector-8b": "StarVector/starvector-8b-svg",
}


def trace(input_path: str, backend: str) -> str:
    """Run StarVector inference on *input_path*, returning an SVG string."""
    try:
        import torch
        from transformers import AutoModelForCausalLM, AutoProcessor
    except ImportError as e:
        raise RuntimeError(
            f"Missing dependency: {e}. "
            "Download the model via Settings → Enhanced Backends."
        ) from e

    from PIL import Image

    repo_id = MODEL_IDS.get(backend)
    if not repo_id:
        raise ValueError(f"Unknown StarVector variant: {backend!r}")

    # RETRACE_MODEL_DIR is set by the Rust host to the local weights directory.
    model_dir = os.environ.get("RETRACE_MODEL_DIR")
    load_from = model_dir if (model_dir and os.path.isdir(model_dir)) else repo_id

    print("progress:init", file=sys.stderr, flush=True)

    device = (
        "cuda"
        if torch.cuda.is_available()
        else "mps"
        if torch.backends.mps.is_available()
        else "cpu"
    )
    dtype = torch.float16 if device != "cpu" else torch.float32

    processor = AutoProcessor.from_pretrained(load_from, trust_remote_code=True)
    model = AutoModelForCausalLM.from_pretrained(
        load_from,
        trust_remote_code=True,
        torch_dtype=dtype,
    ).to(device)
    model.eval()

    print("progress:loaded", file=sys.stderr, flush=True)

    image = Image.open(input_path).convert("RGB")
    inputs = processor(images=image, return_tensors="pt").to(device)

    with torch.no_grad():
        output_ids = model.generate(**inputs, max_new_tokens=4096, do_sample=False)

    svg = processor.batch_decode(output_ids, skip_special_tokens=True)[0].strip()

    # StarVector may prepend reasoning text before the SVG; strip it.
    start = svg.find("<svg")
    if start > 0:
        svg = svg[start:]

    return svg
