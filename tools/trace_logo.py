#!/usr/bin/env python3
"""Herramienta offline de trazado de contornos y medición de paleta para logos en refs/.

Cómo regenerar src/logo_data.rs desde la raíz del repo:
    python3 tools/trace_logo.py
"""

from collections import deque
import math
from pathlib import Path
from PIL import Image, ImageDraw


def median_rgb(pixels):
    if not pixels:
        return (1.0, 1.0, 1.0)
    rs = sorted(p[0] for p in pixels)
    gs = sorted(p[1] for p in pixels)
    bs = sorted(p[2] for p in pixels)
    mid = len(pixels) // 2
    return (rs[mid] / 255.0, gs[mid] / 255.0, bs[mid] / 255.0)


def connected_components(mask, w, h, min_area=20):
    visited = bytearray(w * h)
    comps = []
    for y in range(h):
        for x in range(w):
            idx = y * w + x
            if not mask[idx] or visited[idx]:
                continue
            q = deque([(x, y)])
            visited[idx] = 1
            pts = []
            while q:
                cx, cy = q.popleft()
                pts.append((cx, cy))
                for nx, ny in ((cx + 1, cy), (cx - 1, cy), (cx, cy + 1), (cx, cy - 1)):
                    if 0 <= nx < w and 0 <= ny < h:
                        nidx = ny * w + nx
                        if mask[nidx] and not visited[nidx]:
                            visited[nidx] = 1
                            q.append((nx, ny))
            if len(pts) >= min_area:
                comps.append(pts)
    comps.sort(key=len, reverse=True)
    return comps


def trace_outer_contour(pts_set, w, h):
    """Seguimiento de contorno exterior (Moore-neighbor) sobre un componente conexo."""
    start = min(pts_set, key=lambda p: (p[1], p[0]))
    dirs = [(1, 0), (1, 1), (0, 1), (-1, 1), (-1, 0), (-1, -1), (0, -1), (1, -1)]
    contour = [start]
    cur = start
    back_dir = 4  # venimos desde la izquierda (-1, 0)
    max_steps = len(pts_set) * 4 + 16
    for _ in range(max_steps):
        found = None
        for step in range(8):
            d = (back_dir + 1 + step) % 8
            nx, ny = cur[0] + dirs[d][0], cur[1] + dirs[d][1]
            if 0 <= nx < w and 0 <= ny < h and (nx, ny) in pts_set:
                found = (nx, ny, d)
                break
        if found is None:
            break
        nx, ny, d = found
        if (nx, ny) == start and len(contour) > 2:
            break
        contour.append((nx, ny))
        cur = (nx, ny)
        back_dir = (d + 4) % 8
    return contour


def point_seg_dist(p, a, b):
    dx, dy = b[0] - a[0], b[1] - a[1]
    l2 = dx * dx + dy * dy
    if l2 < 1e-12:
        return math.hypot(p[0] - a[0], p[1] - a[1])
    t = max(0.0, min(1.0, ((p[0] - a[0]) * dx + (p[1] - a[1]) * dy) / l2))
    return math.hypot(p[0] - (a[0] + t * dx), p[1] - (a[1] + t * dy))


def rdp(pts, eps):
    if len(pts) <= 2:
        return pts
    a, b = pts[0], pts[-1]
    best_i = 0
    best_d = -1.0
    for i in range(1, len(pts) - 1):
        d = point_seg_dist(pts[i], a, b)
        if d > best_d:
            best_d = d
            best_i = i
    if best_d > eps:
        left = rdp(pts[: best_i + 1], eps)
        right = rdp(pts[best_i:], eps)
        return left[:-1] + right
    return [a, b]


def simplify_closed(contour, eps, min_pts=5):
    if len(contour) <= min_pts:
        return contour
    # Partir por los dos puntos más lejanos para simplificar la curva cerrada.
    p0 = contour[0]
    far_i = max(range(len(contour)), key=lambda i: (contour[i][0] - p0[0]) ** 2 + (contour[i][1] - p0[1]) ** 2)
    if far_i == 0:
        return contour
    c1 = rdp(contour[: far_i + 1], eps)
    c2 = rdp(contour[far_i:] + [contour[0]], eps)
    out = c1[:-1] + c2[:-1]
    # Eliminar duplicados consecutivos.
    cleaned = []
    for p in out:
        if not cleaned or (abs(p[0] - cleaned[-1][0]) > 1e-5 or abs(p[1] - cleaned[-1][1]) > 1e-5):
            cleaned.append(p)
    if len(cleaned) >= 2 and abs(cleaned[0][0] - cleaned[-1][0]) < 1e-5 and abs(cleaned[0][1] - cleaned[-1][1]) < 1e-5:
        cleaned.pop()
    return cleaned


def rasterize_polygons_iou(polys_px, ref_mask, w, h):
    img = Image.new("L", (w, h), 0)
    draw = ImageDraw.Draw(img)
    for poly in polys_px:
        if len(poly) >= 3:
            draw.polygon(poly, fill=1)
    pred = img.tobytes()
    inter = 0
    union = 0
    for i in range(w * h):
        a = pred[i] > 0
        b = ref_mask[i] > 0
        if a and b:
            inter += 1
        if a or b:
            union += 1
    return inter / union if union > 0 else 0.0


def trace_suns(path: Path):
    im = Image.open(path).convert("RGBA")
    w, h = im.size
    px = list(im.getdata())

    orange_pts = []
    orange_pixels = []
    shadow_pixels = []
    gold_pixels = []
    ink_mask = bytearray(w * h)
    gold_mask = bytearray(w * h)

    for y in range(h):
        for x in range(w):
            r, g, b, a = px[y * w + x]
            if a < 128 or (r > 238 and g > 238 and b > 238):
                continue
            ink_mask[y * w + x] = 1
            if r > 200 and g > 125 and b < 85:
                gold_pixels.append((r, g, b))
                gold_mask[y * w + x] = 1
            elif r > 180 and 60 < g < 135 and b < 65:
                orange_pixels.append((r, g, b))
                orange_pts.append((x, y))
            elif r > 120 and r > g * 1.8 and b < 50:
                shadow_pixels.append((r, g, b))
                orange_pts.append((x, y))

    # Centroide y radio del balón a partir de los píxeles del balón.
    cx = sum(p[0] for p in orange_pts) / len(orange_pts)
    cy = sum(p[1] for p in orange_pts) / len(orange_pts)
    # Incluir las costuras negras dentro del disco para el radio real.
    ball_r = max(
        math.hypot(p[0] - cx, p[1] - cy)
        for p in orange_pts
    )
    # Ajustar centroide por simetría del bounding box del balón naranja.
    xs = [p[0] for p in orange_pts]
    ys = [p[1] for p in orange_pts]
    cx = 0.5 * (min(xs) + max(xs))
    cy = 0.5 * (min(ys) + max(ys))
    ball_r = 0.25 * ((max(xs) - min(xs)) + (max(ys) - min(ys)))

    # Máscara de la placa solar: píxeles dorados + disco del balón (para cerrar huecos internos).
    plate_mask = bytearray(gold_mask)
    for y in range(h):
        for x in range(w):
            if math.hypot(x - cx, y - cy) <= ball_r * 0.96:
                plate_mask[y * w + x] = 1

    comps = connected_components(plate_mask, w, h, min_area=40)
    polys_norm = []
    polys_px = []
    for idx, comp in enumerate(comps):
        eps = 0.85 if idx == 0 else 0.75
        raw = trace_outer_contour(set(comp), w, h)
        simp = simplify_closed(raw, eps=eps, min_pts=4)
        if len(simp) >= 3:
            polys_px.append(simp)
            polys_norm.append([((x - cx) / ball_r, -(y - cy) / ball_r) for (x, y) in simp])

    iou = rasterize_polygons_iou(polys_px, plate_mask, w, h)
    return {
        "ball_center_px": (cx, cy),
        "ball_radius_px": ball_r,
        "gold_rgb": median_rgb(gold_pixels),
        "orange_rgb": median_rgb(orange_pixels),
        "shadow_rgb": median_rgb(shadow_pixels),
        "polys": polys_norm,
        "iou": iou,
    }


def trace_spurs(path: Path):
    im = Image.open(path).convert("RGB")
    w, h = im.size
    px = list(im.getdata())

    ink_mask = bytearray(w * h)
    navy_pixels = []
    for i, (r, g, b) in enumerate(px):
        if r < 200 or g < 200 or b < 200:
            ink_mask[i] = 1
            if r < 100 and g < 100 and b > 45:
                navy_pixels.append((r, g, b))

    # Transformada de distancia aproximada (Chamfer 3-4) en el tercio inferior para hallar
    # el círculo inscrito máximo del balón.
    inf = 10_000_000
    dist = [0 if ink_mask[i] == 0 else inf for i in range(w * h)]
    for y in range(1, h - 1):
        for x in range(1, w - 1):
            i = y * w + x
            if dist[i] > 0:
                dist[i] = min(dist[i], dist[i - 1] + 3, dist[i - w] + 3, dist[i - w - 1] + 4, dist[i - w + 1] + 4)
    for y in range(h - 2, 0, -1):
        for x in range(w - 2, 0, -1):
            i = y * w + x
            if dist[i] > 0:
                dist[i] = min(dist[i], dist[i + 1] + 3, dist[i + w] + 3, dist[i + w + 1] + 4, dist[i + w - 1] + 4)

    # En la referencia refs/spurs_crest.jpg el balón tiene líneas blancas finas internas;
    # rellenamos el disco del balón usando el contorno exterior del tercio inferior.
    lower_pts = [(x, y) for y in range(int(h * 0.64), h) for x in range(w) if ink_mask[y * w + x]]
    lxs = [p[0] for p in lower_pts]
    lys = [p[1] for p in lower_pts]
    ball_cx = 0.5 * (min(lxs) + max(lxs))
    ball_r = 0.5 * (max(lxs) - min(lxs))
    ball_cy = max(lys) - ball_r

    # Separar el gallo (toda la tinta por encima del borde superior del balón).
    rooster_mask = bytearray(w * h)
    for y in range(h):
        for x in range(w):
            if ink_mask[y * w + x] and math.hypot(x - ball_cx, y - ball_cy) > ball_r * 0.94 and y < ball_cy:
                rooster_mask[y * w + x] = 1

    comps = connected_components(rooster_mask, w, h, min_area=50)
    rooster_comp = set(comps[0])
    raw = trace_outer_contour(rooster_comp, w, h)
    simp = simplify_closed(raw, eps=0.85, min_pts=10)
    rooster_norm = [((x - ball_cx) / ball_r, -(y - ball_cy) / ball_r) for (x, y) in simp]

    iou = rasterize_polygons_iou([simp], rooster_mask, w, h)
    return {
        "ball_center_px": (ball_cx, ball_cy),
        "ball_radius_px": ball_r,
        "navy_rgb": median_rgb(navy_pixels),
        "rooster_poly": rooster_norm,
        "iou": iou,
    }


def write_rust_data(out_path: Path, suns, spurs):
    lines = [
        "#![allow(dead_code)]",
        "//! Datos de contornos y paleta medidos desde `refs/suns_logo.png` y `refs/spurs_crest.jpg`.",
        "//!",
        "//! Cómo regenerar este archivo:",
        "//!     python3 tools/trace_logo.py",
        "",
        f"pub const SUNS_GOLD_RGB: [f32; 3] = [{suns['gold_rgb'][0]:.4}, {suns['gold_rgb'][1]:.4}, {suns['gold_rgb'][2]:.4}];",
        f"pub const SUNS_ORANGE_RGB: [f32; 3] = [{suns['orange_rgb'][0]:.4}, {suns['orange_rgb'][1]:.4}, {suns['orange_rgb'][2]:.4}];",
        f"pub const SUNS_SHADOW_RGB: [f32; 3] = [{suns['shadow_rgb'][0]:.4}, {suns['shadow_rgb'][1]:.4}, {suns['shadow_rgb'][2]:.4}];",
        "",
        f"pub const SPURS_NAVY_RGB: [f32; 3] = [{spurs['navy_rgb'][0]:.4}, {spurs['navy_rgb'][1]:.4}, {spurs['navy_rgb'][2]:.4}];",
        "",
        "pub const SPURS_ROOSTER_CONTOUR: &[[f32; 2]] = &[",
    ]
    for x, y in spurs["rooster_poly"]:
        lines.append(f"    [{x:.5}, {y:.5}],")
    lines.append("];")
    lines.append("")
    for idx, poly in enumerate(suns["polys"]):
        lines.append(f"pub const SUNS_RAY_{idx}: &[[f32; 2]] = &[")
        for x, y in poly:
            lines.append(f"    [{x:.5}, {y:.5}],")
        lines.append("];")
    lines.append("pub const SUNS_CONTOURS: &[&[[f32; 2]]] = &[")
    for idx in range(len(suns["polys"])):
        lines.append(f"    SUNS_RAY_{idx},")
    lines.append("];")
    out_path.write_text("\n".join(lines) + "\n", encoding="utf-8")


def main():
    root = Path(__file__).resolve().parent.parent
    suns = trace_suns(root / "refs" / "suns_logo.png")
    spurs = trace_spurs(root / "refs" / "spurs_crest.jpg")
    out_path = root / "src" / "logo_data.rs"
    write_rust_data(out_path, suns, spurs)
    print(f"SUNS: {len(suns['polys'])} componentes, puntos={ [len(p) for p in suns['polys']] }, IoU={suns['iou']:.4f}")
    print(f"  gold={suns['gold_rgb']} orange={suns['orange_rgb']} shadow={suns['shadow_rgb']}")
    print(f"SPURS: puntos={len(spurs['rooster_poly'])}, IoU={spurs['iou']:.4f}, navy={spurs['navy_rgb']}")


if __name__ == "__main__":
    main()
