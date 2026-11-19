#!/usr/bin/env python3
"""Mide proporciones, contornos, profundidad de perfil, máscara de la 'W' y paleta
desde `refs/raqueta.webp` (vista de perfil a la izquierda, vista frontal a la derecha)
y genera `src/racket_data.rs` + `docs/medidas-raqueta.md`.

Uso:
    python3 tools/measure_racket.py
"""

from __future__ import annotations

import math
import statistics
from collections import deque
from pathlib import Path
from PIL import Image

ROOT = Path(__file__).resolve().parent.parent
REF_PATH = ROOT / "refs" / "raqueta.webp"
OUT_RS = ROOT / "src" / "racket_data.rs"
OUT_MD = ROOT / "docs" / "medidas-raqueta.md"


def is_ink(r: int, g: int, b: int) -> bool:
    """Pixel de raqueta (excluye fondo blanco y lecho de cuerdas casi blanco)."""
    # Fondo blanco puro o casi blanco.
    if r > 240 and g > 240 and b > 240:
        return False
    # Cuerdas rojas de la W dentro del lecho: las tratamos aparte.
    return True


def is_frame_ink(r: int, g: int, b: int) -> bool:
    """Pixel sólido del marco/mango (excluye fondo blanco y cuerdas del lecho)."""
    if r > 235 and g > 235 and b > 235:
        return False
    # Excluir rojo vivo de la W para no romper los agujeros del lecho de cuerdas.
    if r > 140 and (r - g) > 45 and (r - b) > 45:
        return False
    # Excluir gris muy claro de las líneas de cuerdas blancas.
    if min(r, g, b) > 185 and max(r, g, b) - min(r, g, b) < 20:
        return False
    return True


def connected_components_mask(mask: list[list[bool]]) -> list[list[tuple[int, int]]]:
    h = len(mask)
    w = len(mask[0])
    seen = [[False] * w for _ in range(h)]
    comps: list[list[tuple[int, int]]] = []
    for y in range(h):
        for x in range(w):
            if mask[y][x] and not seen[y][x]:
                comp = []
                q = deque([(x, y)])
                seen[y][x] = True
                while q:
                    cx, cy = q.popleft()
                    comp.append((cx, cy))
                    for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
                        nx, ny = cx + dx, cy + dy
                        if 0 <= nx < w and 0 <= ny < h and mask[ny][nx] and not seen[ny][nx]:
                            seen[ny][nx] = True
                            q.append((nx, ny))
                comps.append(comp)
    comps.sort(key=len, reverse=True)
    return comps


def median_rgb(pixels: list[tuple[int, int, int]]) -> tuple[int, int, int]:
    rs = sorted(p[0] for p in pixels)
    gs = sorted(p[1] for p in pixels)
    bs = sorted(p[2] for p in pixels)
    n = len(pixels)
    return (rs[n // 2], gs[n // 2], bs[n // 2])


def rgb_to_hex(rgb: tuple[int, int, int]) -> str:
    return f"#{rgb[0]:02X}{rgb[1]:02X}{rgb[2]:02X}"


def rgb_to_f32(rgb: tuple[int, int, int]) -> str:
    return f"[{rgb[0] / 255.0:.4f}, {rgb[1] / 255.0:.4f}, {rgb[2] / 255.0:.4f}]"


def main() -> None:
    img = Image.open(REF_PATH).convert("RGB")
    w, h = img.size
    px = img.load()

    frame_mask = [[is_frame_ink(px[x, y][0], px[x, y][1], px[x, y][2]) for x in range(w)] for y in range(h)]
    comps = connected_components_mask(frame_mask)
    # Dos componentes principales: perfil (izquierda) y frente (derecha).
    main_two = comps[:2]
    main_two.sort(key=lambda c: sum(p[0] for p in c) / len(c))
    prof_comp, front_comp = main_two

    prof_xs = [p[0] for p in prof_comp]
    prof_ys = [p[1] for p in prof_comp]
    front_xs = [p[0] for p in front_comp]
    front_ys = [p[1] for p in front_comp]

    fx0, fx1 = min(front_xs), max(front_xs)
    fy0, fy1 = min(front_ys), max(front_ys)
    px0, px1 = min(prof_xs), max(prof_xs)
    py0, py1 = min(prof_ys), max(prof_ys)

    total_h_px = float(fy1 - fy0)
    scale = 2.0 / total_h_px  # longitud total normalizada = 2.0

    # Eje vertical de simetría de la vista frontal (mediana de centros X del mango).
    grip_y_start = fy0 + int(0.76 * total_h_px)
    grip_centers = []
    front_set = set(front_comp)
    for y in range(grip_y_start, fy1 - 10):
        row_xs = [x for x in range(fx0, fx1 + 1) if (x, y) in front_set]
        if row_xs:
            grip_centers.append(0.5 * (min(row_xs) + max(row_xs)))
    axis_x = statistics.median(grip_centers)
    mid_y = 0.5 * (fy0 + fy1)

    def to_norm(x: float, y: float) -> tuple[float, float]:
        return ((x - axis_x) * scale, (mid_y - y) * scale)

    # Detectar agujeros dentro de la caja frontal (fondo = not frame_mask).
    fw = fx1 - fx0 + 1
    fh = fy1 - fy0 + 1
    front_local = [[(fx0 + dx, fy0 + dy) in front_set for dx in range(fw)] for dy in range(fh)]
    bg_local = [[not front_local[dy][dx] for dx in range(fw)] for dy in range(fh)]

    # Flood fill desde el borde de la caja frontal para marcar el fondo exterior.
    ext_bg = [[False] * fw for _ in range(fh)]
    q = deque()
    for dx in range(fw):
        for dy in (0, fh - 1):
            if bg_local[dy][dx] and not ext_bg[dy][dx]:
                ext_bg[dy][dx] = True
                q.append((dx, dy))
    for dy in range(fh):
        for dx in (0, fw - 1):
            if bg_local[dy][dx] and not ext_bg[dy][dx]:
                ext_bg[dy][dx] = True
                q.append((dx, dy))
    while q:
        cx, cy = q.popleft()
        for ddx, ddy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
            nx, ny = cx + ddx, cy + ddy
            if 0 <= nx < fw and 0 <= ny < fh and bg_local[ny][nx] and not ext_bg[ny][nx]:
                ext_bg[ny][nx] = True
                q.append((nx, ny))

    hole_mask = [[bg_local[dy][dx] and not ext_bg[dy][dx] for dx in range(fw)] for dy in range(fh)]
    holes = connected_components_mask(hole_mask)
    # Mayor agujero = lecho de cuerdas; segundo mayor = triángulo de la garganta.
    bed_hole = holes[0]
    throat_hole = holes[1]

    bed_xs = [fx0 + p[0] for p in bed_hole]
    bed_ys = [fy0 + p[1] for p in bed_hole]
    bx0, bx1 = min(bed_xs), max(bed_xs)
    by0, by1 = min(bed_ys), max(bed_ys)
    bed_cy_px = 0.5 * (by0 + by1)
    bed_cx_px = axis_x

    # Medidas del aro de cabeza.
    head_outer_w_px = 0
    for y in range(by0, by1 + 1):
        row_xs = [x for x in range(fx0, fx1 + 1) if (x, y) in front_set]
        if row_xs:
            head_outer_w_px = max(head_outer_w_px, max(row_xs) - min(row_xs))
    head_outer_h_px = by1 - fy0 + (by0 - fy0)  # aprox hasta debajo del puente
    # Grosor arriba del aro y en los costados.
    rim_top_px = by0 - fy0
    row_mid = [x for x in range(fx0, fx1 + 1) if (x, int(bed_cy_px)) in front_set]
    rim_left_px = bx0 - min(row_mid)
    rim_right_px = max(row_mid) - bx1
    rim_side_px = 0.5 * (rim_left_px + rim_right_px)

    # Medidas de la garganta y puente.
    th_xs = [fx0 + p[0] for p in throat_hole]
    th_ys = [fy0 + p[1] for p in throat_hole]
    th_y0, th_y1 = min(th_ys), max(th_ys)
    bridge_thickness_px = th_y0 - by1
    throat_open_w_px = max(th_xs) - min(th_xs)
    throat_len_px = th_y1 - by1

    # Mango: debajo de th_y1 donde el ancho se estabiliza hasta el tapón.
    grip_widths = []
    for y in range(th_y1 + int(0.05 * total_h_px), fy1 - int(0.05 * total_h_px)):
        row_xs = [x for x in range(fx0, fx1 + 1) if (x, y) in front_set]
        if row_xs:
            grip_widths.append((y, max(row_xs) - min(row_xs)))
    grip_w_top_px = statistics.median([w_ for _, w_ in grip_widths[:20]])
    grip_w_bot_px = statistics.median([w_ for _, w_ in grip_widths[-20:]])
    # Ancho máximo del tapón al final.
    butt_widths = []
    for y in range(fy1 - int(0.03 * total_h_px), fy1):
        row_xs = [x for x in range(fx0, fx1 + 1) if (x, y) in front_set]
        if row_xs:
            butt_widths.append(max(row_xs) - min(row_xs))
    butt_w_px = max(butt_widths) if butt_widths else grip_w_bot_px

    # Profundidad del perfil (vista izquierda): espesor sólido de la pared del marco (no el hueco del aro).
    prof_set = set(prof_comp)
    def profile_wall_thickness(y_a: int, y_b: int) -> float:
        ws = []
        for y in range(y_a, y_b):
            rxs = sorted(x for x in range(px0, px1 + 1) if (x, y) in prof_set)
            if not rxs:
                continue
            # Contar el primer segmento sólido continuo desde la izquierda en esa fila.
            run_len = 1
            for k in range(1, len(rxs)):
                if rxs[k] == rxs[k - 1] + 1:
                    run_len += 1
                else:
                    break
            ws.append(run_len)
        return statistics.median(ws) if ws else 0.0

    prof_head_px = profile_wall_thickness(by0 + 30, by1 - 30)
    prof_throat_px = profile_wall_thickness(th_y0 + 15, th_y1 - 15)
    prof_grip_px = profile_wall_thickness(th_y1 + 40, fy1 - 40)

    # Remuestreo angular de la línea media y grosor del aro (64 muestras).
    n_arc = 64
    bed_set = set((fx0 + p[0], fy0 + p[1]) for p in bed_hole)
    midline_pts = []
    rim_widths = []
    for i in range(n_arc):
        theta = 2.0 * math.pi * i / n_arc
        ca, sa = math.cos(theta), math.sin(theta)
        r_in = None
        r_out = None
        for r in range(10, int(head_outer_w_px)):
            sx = int(round(bed_cx_px + r * ca))
            sy = int(round(bed_cy_px - r * sa))
            if not (fx0 <= sx <= fx1 and fy0 <= sy <= fy1):
                break
            if (sx, sy) in bed_set:
                r_in = r
            if (sx, sy) in front_set:
                if r_in is None:
                    r_in = r
                r_out = r
        if r_in is None:
            r_in = 0.4 * (bx1 - bx0)
        if r_out is None:
            r_out = r_in + rim_side_px
        r_mid = 0.5 * (r_in + r_out)
        mx = bed_cx_px + r_mid * ca
        my = bed_cy_px - r_mid * sa
        nx, ny = to_norm(mx, my)
        midline_pts.append((nx, ny))
        rim_widths.append((r_out - r_in) * scale)

    # Máscara de baja resolución (32x40) de la W roja normalizada a la caja del lecho.
    mw, mh = 32, 40
    w_mask = []
    red_pixels = []
    string_pixels = []
    for gy in range(mh):
        row = []
        y0_b = by0 + gy * (by1 - by0) / mh
        y1_b = by0 + (gy + 1) * (by1 - by0) / mh
        for gx in range(mw):
            x0_b = bx0 + gx * (bx1 - bx0) / mw
            x1_b = bx0 + (gx + 1) * (bx1 - bx0) / mw
            red_hits = 0
            total_hits = 0
            for yy in range(int(y0_b), max(int(y0_b) + 1, int(y1_b))):
                for xx in range(int(x0_b), max(int(x0_b) + 1, int(x1_b))):
                    r, g, b = px[xx, yy]
                    total_hits += 1
                    if r > 140 and (r - g) > 55 and (r - b) > 55:
                        red_hits += 1
                        if (r - g) > 80 and (r - b) > 80:
                            red_pixels.append((r, g, b))
                    elif 145 < min(r, g, b) < 228 and (max(r, g, b) - min(r, g, b)) < 22:
                        string_pixels.append((r, g, b))
            row.append(1 if red_hits >= 1 else 0)
        w_mask.append(row)

    # Cerrar huecos de 1 celda entre cuerdas rojas vecinas para que la 'W' sea continua.
    filled = [r[:] for r in w_mask]
    for gy in range(1, mh - 1):
        for gx in range(1, mw - 1):
            if w_mask[gy][gx] == 0 and (
                (w_mask[gy][gx - 1] and w_mask[gy][gx + 1])
                or (w_mask[gy - 1][gx] and w_mask[gy + 1][gx])
            ):
                filled[gy][gx] = 1
    w_mask = filled

    # Mediana de colores por zonas medidas en refs/raqueta.webp:
    # - turquesa: banda metálica izquierda (vista de perfil y costado izquierdo)
    # - azul: costado derecho del aro y garganta (vista frontal)
    # - marino oscuro: tercio superior del aro y puente bajo el lecho
    teal_pixels = []
    blue_pixels = []
    navy_pixels = []
    grip_pixels = []
    for x, y in prof_comp:
        r, g, b = px[x, y]
        if int(h * 0.18) <= y <= int(h * 0.72) and b > r + 45 and g > 110:
            teal_pixels.append((r, g, b))
    for x, y in front_comp:
        r, g, b = px[x, y]
        if y < by0 + int(0.18 * (by1 - by0)) and b > r + 8 and max(r, g, b) < 120:
            navy_pixels.append((r, g, b))
        elif by1 <= y <= th_y0 + 4 and bx0 <= x <= bx1 and max(r, g, b) < 140:
            navy_pixels.append((r, g, b))
        elif by0 + int(0.28 * (by1 - by0)) <= y <= th_y1 and x > axis_x + 4 and b > r + 45 and (b - g) > 28:
            blue_pixels.append((r, g, b))
        elif y > th_y1 + int(0.08 * total_h_px) and y < fy1 - int(0.06 * total_h_px):
            grip_pixels.append((r, g, b))

    teal_rgb = median_rgb(teal_pixels)
    blue_rgb = median_rgb(blue_pixels)
    navy_rgb = median_rgb(navy_pixels)
    red_rgb = median_rgb(red_pixels) if red_pixels else (200, 38, 30)
    string_rgb = median_rgb(string_pixels) if string_pixels else (215, 219, 226)
    grip_sorted = sorted(grip_pixels, key=lambda c: c[0] + c[1] + c[2])
    grip_a = grip_sorted[int(0.32 * len(grip_sorted))]
    grip_b = grip_sorted[int(0.68 * len(grip_sorted))]

    # Valores normalizados clave.
    _, head_cy_norm = to_norm(bed_cx_px, bed_cy_px)
    head_inner_w = (bx1 - bx0) * scale
    head_inner_h = (by1 - by0) * scale
    head_outer_w = head_outer_w_px * scale
    head_outer_h = (by1 + bridge_thickness_px - fy0) * scale
    rim_thick_side = rim_side_px * scale
    rim_thick_top = rim_top_px * scale
    bridge_y = to_norm(axis_x, 0.5 * (by1 + th_y0))[1]
    bridge_h = bridge_thickness_px * scale
    throat_join_y = to_norm(axis_x, th_y1)[1]
    throat_len = throat_len_px * scale
    throat_open_w = throat_open_w_px * scale
    grip_top_y = throat_join_y - 0.04
    grip_len = (grip_top_y - (-1.0))
    grip_w_top = grip_w_top_px * scale
    grip_w_bot = grip_w_bot_px * scale
    butt_w = butt_w_px * scale
    depth_head = prof_head_px * scale
    depth_throat = prof_throat_px * scale
    depth_grip = prof_grip_px * scale

    # Escribir src/racket_data.rs
    lines = [
        "#![allow(dead_code)]",
        "//! Parámetros geométricos, contorno medio, máscara de la 'W' y paleta medidos",
        "//! desde `refs/raqueta.webp` (vista de perfil izquierda + vista frontal derecha).",
        "//!",
        "//! Cómo regenerar:",
        "//!     python3 tools/measure_racket.py",
        "",
        f"// Longitud total normalizada: 2.0 (escala = {scale:.6f} u/px sobre {int(total_h_px)} px).",
        f"pub const RACKET_TOTAL_LENGTH: f32 = 2.0;",
        f"pub const RACKET_HEAD_CENTER_Y: f32 = {head_cy_norm:.5f};",
        f"pub const RACKET_HEAD_OUTER_WIDTH: f32 = {head_outer_w:.5f};",
        f"pub const RACKET_HEAD_OUTER_HEIGHT: f32 = {head_outer_h:.5f};",
        f"pub const RACKET_HEAD_INNER_WIDTH: f32 = {head_inner_w:.5f};",
        f"pub const RACKET_HEAD_INNER_HEIGHT: f32 = {head_inner_h:.5f};",
        f"pub const RACKET_RIM_THICK_SIDE: f32 = {rim_thick_side:.5f};",
        f"pub const RACKET_RIM_THICK_TOP: f32 = {rim_thick_top:.5f};",
        f"pub const RACKET_BRIDGE_Y: f32 = {bridge_y:.5f};",
        f"pub const RACKET_BRIDGE_HEIGHT: f32 = {bridge_h:.5f};",
        f"pub const RACKET_THROAT_JOIN_Y: f32 = {throat_join_y:.5f};",
        f"pub const RACKET_THROAT_LENGTH: f32 = {throat_len:.5f};",
        f"pub const RACKET_THROAT_OPEN_WIDTH: f32 = {throat_open_w:.5f};",
        f"pub const RACKET_GRIP_TOP_Y: f32 = {grip_top_y:.5f};",
        f"pub const RACKET_GRIP_LENGTH: f32 = {grip_len:.5f};",
        f"pub const RACKET_GRIP_WIDTH_TOP: f32 = {grip_w_top:.5f};",
        f"pub const RACKET_GRIP_WIDTH_BOT: f32 = {grip_w_bot:.5f};",
        f"pub const RACKET_BUTTCAP_WIDTH: f32 = {butt_w:.5f};",
        "",
        "// Profundidades medidas en la vista de perfil (eje Z).",
        f"pub const RACKET_DEPTH_HEAD: f32 = {depth_head:.5f};",
        f"pub const RACKET_DEPTH_THROAT: f32 = {depth_throat:.5f};",
        f"pub const RACKET_DEPTH_GRIP: f32 = {depth_grip:.5f};",
        "",
        "// Paleta medida por mediana de píxeles en zonas de `refs/raqueta.webp`.",
        f"pub const RACKET_TEAL_RGB: [f32; 3] = {rgb_to_f32(teal_rgb)}; // {rgb_to_hex(teal_rgb)}",
        f"pub const RACKET_BLUE_RGB: [f32; 3] = {rgb_to_f32(blue_rgb)}; // {rgb_to_hex(blue_rgb)}",
        f"pub const RACKET_NAVY_RGB: [f32; 3] = {rgb_to_f32(navy_rgb)}; // {rgb_to_hex(navy_rgb)}",
        f"pub const RACKET_GRIP_A_RGB: [f32; 3] = {rgb_to_f32(grip_a)}; // {rgb_to_hex(grip_a)}",
        f"pub const RACKET_GRIP_B_RGB: [f32; 3] = {rgb_to_f32(grip_b)}; // {rgb_to_hex(grip_b)}",
        f"pub const RACKET_STRING_RGB: [f32; 3] = {rgb_to_f32(string_rgb)}; // {rgb_to_hex(string_rgb)}",
        f"pub const RACKET_LOGO_RED_RGB: [f32; 3] = {rgb_to_f32(red_rgb)}; // {rgb_to_hex(red_rgb)}",
        "",
        "/// Línea media del aro (x, y) y ancho local en el plano por ángulo desde el centro de la cabeza.",
        "pub const RACKET_HOOP_MIDLINE: &[[f32; 3]] = &[",
    ]
    for (nx, ny), rw in zip(midline_pts, rim_widths):
        lines.append(f"    [{nx:.5f}, {ny:.5f}, {rw:.5f}],")
    lines.append("];")
    lines.append("")
    lines.append(f"pub const RACKET_W_MASK_COLS: usize = {mw};")
    lines.append(f"pub const RACKET_W_MASK_ROWS: usize = {mh};")
    lines.append("/// Máscara 32×40 de la 'W' roja normalizada a la caja interior del lecho (fila 0 = arriba).")
    lines.append("pub const RACKET_W_MASK: &[u32] = &[")
    for row in w_mask:
        bits = 0
        for gx, val in enumerate(row):
            if val:
                bits |= 1 << gx
        lines.append(f"    0x{bits:08X},")
    lines.append("];")
    OUT_RS.write_text("\n".join(lines) + "\n", encoding="utf-8")

    # Documentar en docs/medidas-raqueta.md
    md = f"""# Medidas de la raqueta Wilson Ultra (`refs/raqueta.webp`)

## Método de medición
1. Se cargó `refs/raqueta.webp` ({w}×{h} px) con `tools/measure_racket.py`.
2. Se separaron las dos vistas por componentes conexas de píxeles del marco: perfil izquierdo (`x ∈ [{px0}, {px1}]`) y frente derecho (`x ∈ [{fx0}, {fx1}]`, altura total `{int(total_h_px)} px`).
3. Todas las dimensiones lineales se normalizaron a `longitud total = 2.0` (`escala = {scale:.6f} u/px`) con origen en el centro vertical de la vista frontal (`y ∈ [-1.0, +1.0]`) y eje de simetría sobre la mediana horizontal del mango.
4. Se detectaron los agujeros internos de la vista frontal (componentes de fondo desconectadas del borde): el lecho de cuerdas y la abertura triangular de la garganta en Y.
5. Se remuestreó el aro en {n_arc} ángulos desde el centro de la cabeza para obtener la línea media y el ancho local en el plano, y se midió la profundidad (eje Z) sobre el perfil izquierdo.
6. Se calculó la mediana RGB por zona (costado izquierdo turquesa, derecho azul, tercio superior/puente marino, mango en percentiles 32/68, cuerdas grises y 'W' roja) y se discretizó la 'W' en una máscara `32×40`.

## Tabla de medidas normalizadas

| Parámetro | Valor normalizado | Píxeles (aprox.) |
|---|---:|---:|
| Longitud total | `{2.0:.4f}` | `{int(total_h_px)}` |
| Centro vertical de cabeza (`Y`) | `{head_cy_norm:.4f}` | `{bed_cy_px:.1f}` |
| Ancho exterior de cabeza | `{head_outer_w:.4f}` | `{head_outer_w_px}` |
| Alto exterior de cabeza | `{head_outer_h:.4f}` | `{int(head_outer_h / scale)}` |
| Ancho interior de cabeza (lecho) | `{head_inner_w:.4f}` | `{bx1 - bx0}` |
| Alto interior de cabeza (lecho) | `{head_inner_h:.4f}` | `{by1 - by0}` |
| Grosor del aro (costado) | `{rim_thick_side:.4f}` | `{rim_side_px:.1f}` |
| Grosor del aro (arriba) | `{rim_thick_top:.4f}` | `{rim_top_px}` |
| Profundidad del marco (cabeza, Z) | `{depth_head:.4f}` | `{prof_head_px:.1f}` |
| Profundidad del marco (garganta, Z) | `{depth_throat:.4f}` | `{prof_throat_px:.1f}` |
| Profundidad del mango (Z) | `{depth_grip:.4f}` | `{prof_grip_px:.1f}` |
| Largo de la garganta | `{throat_len:.4f}` | `{throat_len_px}` |
| Apertura superior de la Y | `{throat_open_w:.4f}` | `{throat_open_w_px}` |
| Unión inferior de brazos (`Y`) | `{throat_join_y:.4f}` | `{th_y1}` |
| Largo del mango | `{grip_len:.4f}` | `{int(grip_len / scale)}` |
| Ancho del mango (arriba / abajo) | `{grip_w_top:.4f}` / `{grip_w_bot:.4f}` | `{grip_w_top_px:.1f}` / `{grip_w_bot_px:.1f}` |
| Ancho del tapón | `{butt_w:.4f}` | `{butt_w_px}` |

## Paleta medida (mediana RGB)

| Zona | Hex | RGB `[0..1]` |
|---|---|---|
| Marco izquierdo (turquesa) | `{rgb_to_hex(teal_rgb)}` | `{rgb_to_f32(teal_rgb)}` |
| Marco derecho (azul) | `{rgb_to_hex(blue_rgb)}` | `{rgb_to_f32(blue_rgb)}` |
| Tercio superior y puente (marino) | `{rgb_to_hex(navy_rgb)}` | `{rgb_to_f32(navy_rgb)}` |
| Mango tono A / tono B | `{rgb_to_hex(grip_a)}` / `{rgb_to_hex(grip_b)}` | `{rgb_to_f32(grip_a)}` / `{rgb_to_f32(grip_b)}` |
| Cuerdas | `{rgb_to_hex(string_rgb)}` | `{rgb_to_f32(string_rgb)}` |
| Logo W rojo | `{rgb_to_hex(red_rgb)}` | `{rgb_to_f32(red_rgb)}` |
"""
    OUT_MD.write_text(md, encoding="utf-8")
    print(f"OK: head=({head_outer_w:.3f}x{head_outer_h:.3f}), depth_head={depth_head:.3f} > rim_side={rim_thick_side:.3f}")
    print(f"Colors: teal={rgb_to_hex(teal_rgb)} blue={rgb_to_hex(blue_rgb)} navy={rgb_to_hex(navy_rgb)} red={rgb_to_hex(red_rgb)} string={rgb_to_hex(string_rgb)}")


if __name__ == "__main__":
    main()
