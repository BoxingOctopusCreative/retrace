"""StarVector backend (1B and 8B models).

Requires model weights downloaded via Settings → Enhanced Backends in the Re:Trace app.
"""


def trace(input_path: str, output_path: str, backend: str) -> None:
    # TODO (milestone 4): implement StarVector inference
    # model_id = "StarVector/starvector-1b-svg" if backend == "starvector-1b"
    #             else "StarVector/starvector-8b-svg"
    # from transformers import AutoModelForCausalLM, AutoProcessor
    # ...
    raise NotImplementedError(
        f"StarVector backend ({backend}) not yet available. "
        "Download model weights via Settings → Enhanced Backends."
    )
