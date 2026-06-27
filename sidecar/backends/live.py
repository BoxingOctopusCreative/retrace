"""LIVE (Layer-wise Image Vectorization) backend.

Implements artistic layered vectorization: the image is colour-quantized into
N layers, each layer's connected regions are boundary-traced and smoothed into
cubic-Bézier SVG paths.

Slower than vtracer by design — this is an iterative, multi-pass process.
Best for artistic illustrations and complex colourful images.

Requires: Pillow, numpy (via torch), scipy — all installed by the Re:Trace
sidecar environment setup.
"""

from __future__ import annotations

import math
import sys

import numpy as np
from PIL import Image
from scipy import ndimage as ndi

# Raise the recursion limit so RDP can handle large contours.
sys.setrecursionlimit(50_000)

# ── Entry point ───────────────────────────────────────────────────────────────


def trace(input_path: str) -> str:
    """Vectorize *input_path* using layered colour decomposition.

    Emits ``progress:init`` and ``progress:layer:N/M`` lines to stderr so
    the host process can surface tracing progress in the UI.
    Returns an SVG string written to stdout by the caller.
    """
    img = Image.open(input_path).convert("RGB")

    # Cap resolution so worst-case runtime stays under ~60 s on a modern CPU.
    MAX_DIM = 800
    w, h = img.size
    if max(w, h) > MAX_DIM:
        scale = MAX_DIM / max(w, h)
        w, h = max(1, int(w * scale)), max(1, int(h * scale))
        img = img.resize((w, h), Image.LANCZOS)

    N_LAYERS = 8
    print("progress:init", file=sys.stderr, flush=True)

    quantized = img.quantize(colors=N_LAYERS, method=Image.Quantize.MEDIANCUT)
    palette = quantized.getpalette()  # flat [R, G, B, R, G, B, …]
    q_arr = np.asarray(quantized, dtype=np.uint8)

    svg_paths: list[str] = []

    for layer_idx in range(N_LAYERS):
        print(f"progress:layer:{layer_idx + 1}/{N_LAYERS}", file=sys.stderr, flush=True)

        r = palette[layer_idx * 3]
        g = palette[layer_idx * 3 + 1]
        b = palette[layer_idx * 3 + 2]
        color_hex = f"#{r:02x}{g:02x}{b:02x}"

        mask = (q_arr == layer_idx).astype(np.uint8)
        if mask.sum() < 16:
            continue

        for path_d in _layer_paths(mask):
            svg_paths.append(f'  <path fill="{color_hex}" d="{path_d}"/>')

    body = "\n".join(svg_paths)
    return (
        f'<svg xmlns="http://www.w3.org/2000/svg" '
        f'width="{w}" height="{h}" viewBox="0 0 {w} {h}">\n'
        f"{body}\n</svg>\n"
    )


# ── Layer → SVG paths ─────────────────────────────────────────────────────────


def _layer_paths(mask: np.ndarray) -> list[str]:
    """Return one SVG path string per connected component in *mask*."""
    # Fill holes so each component has a simple outer boundary.
    filled = ndi.binary_fill_holes(mask).astype(np.uint8)
    # Slight closing to merge near-touching pixels and smooth 1-pixel noise.
    closed = ndi.binary_closing(filled, iterations=1).astype(np.uint8)

    labeled, n_comps = ndi.label(closed)
    paths: list[str] = []

    for comp_id in range(1, n_comps + 1):
        comp = (labeled == comp_id).astype(np.uint8)
        if comp.sum() < 9:
            continue
        contour = _trace_boundary(comp)
        if len(contour) < 4:
            continue
        simplified = _rdp(contour, epsilon=2.0)
        if len(simplified) < 3:
            continue
        paths.append(_smooth_path(simplified))

    return paths


# ── Boundary tracing ──────────────────────────────────────────────────────────

# 8-neighbourhood offsets, clockwise starting from East.
_DELTAS = [(0, 1), (1, 1), (1, 0), (1, -1), (0, -1), (-1, -1), (-1, 0), (-1, 1)]


def _trace_boundary(mask: np.ndarray) -> list[tuple[float, float]]:
    """Trace the outer boundary of a single connected binary region.

    Uses Moore-neighbourhood boundary tracing (Jacob's stopping criterion).
    Returns an ordered list of ``(x, y)`` pixel-centre coordinates.
    """
    rows, cols = np.where(mask)
    if len(rows) == 0:
        return []

    h, w = mask.shape
    # Start at the top-most, left-most foreground pixel.
    start_r = int(rows.min())
    start_c = int(cols[rows == start_r].min())

    contour: list[tuple[float, float]] = [(float(start_c), float(start_r))]
    r, c = start_r, start_c
    # We treat the start as if we arrived by moving East (direction index 0),
    # so the initial backtrack direction is West (index 4) — the first search
    # starts from there and sweeps clockwise, which is correct for the
    # top-left pixel of any foreground region.
    prev_d = 0

    for _ in range(min(h * w * 4, 200_000)):
        backtrack = (prev_d + 4) % 8
        moved = False
        for k in range(8):
            d = (backtrack + k) % 8
            nr = r + _DELTAS[d][0]
            nc = c + _DELTAS[d][1]
            if not (0 <= nr < h and 0 <= nc < w) or not mask[nr, nc]:
                continue
            # Jacob's stopping criterion: we've closed the loop.
            if (nr, nc) == (start_r, start_c) and len(contour) > 4:
                return contour
            r, c = nr, nc
            prev_d = d
            contour.append((float(c), float(r)))
            moved = True
            break
        if not moved:
            break

    return contour


# ── Path simplification & smoothing ──────────────────────────────────────────


def _rdp(
    pts: list[tuple[float, float]], epsilon: float
) -> list[tuple[float, float]]:
    """Ramer–Douglas–Peucker polyline simplification."""
    if len(pts) <= 2:
        return list(pts)

    sx, sy = pts[0]
    ex, ey = pts[-1]
    dx, dy = ex - sx, ey - sy
    length = math.hypot(dx, dy)

    max_d = 0.0
    max_i = 0
    for i in range(1, len(pts) - 1):
        px, py = pts[i]
        d = (
            abs(dy * px - dx * py + ex * sy - ey * sx) / length
            if length > 0
            else math.hypot(px - sx, py - sy)
        )
        if d > max_d:
            max_d, max_i = d, i

    if max_d <= epsilon:
        return [pts[0], pts[-1]]
    return _rdp(pts[: max_i + 1], epsilon)[:-1] + _rdp(pts[max_i:], epsilon)


def _smooth_path(pts: list[tuple[float, float]]) -> str:
    """Convert a polygon to a smooth closed SVG path.

    Uses Catmull-Rom → cubic Bézier conversion so the curve passes through
    every simplified vertex and is C¹-continuous at each one.
    """
    n = len(pts)
    if n < 3:
        return ""

    parts = [f"M {pts[0][0]:.1f},{pts[0][1]:.1f}"]
    for i in range(n):
        p0 = pts[(i - 1) % n]
        p1 = pts[i % n]
        p2 = pts[(i + 1) % n]
        p3 = pts[(i + 2) % n]

        # Catmull-Rom control points (tension = 1/6).
        c1x = p1[0] + (p2[0] - p0[0]) / 6
        c1y = p1[1] + (p2[1] - p0[1]) / 6
        c2x = p2[0] - (p3[0] - p1[0]) / 6
        c2y = p2[1] - (p3[1] - p1[1]) / 6

        parts.append(
            f"C {c1x:.1f},{c1y:.1f} {c2x:.1f},{c2y:.1f} {p2[0]:.1f},{p2[1]:.1f}"
        )

    parts.append("Z")
    return " ".join(parts)
