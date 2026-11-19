#!/usr/bin/env python3
"""Mide contornos, surcos (Zhang-Suen), visor, orejeras, mangueras y paleta de
`refs/samus_helmet.jpg`, así como proporciones de gradas y paleta de `refs/bombonera.webp`.
Genera `src/samus_bombonera_data.rs` y `docs/medidas-samus-bombonera.md`.

Uso:
    python3 tools/measure_samus_bombonera.py
"""

from __future__ import annotations

import math
import statistics
from collections import deque
from pathlib import Path
from PIL import Image

ROOT = Path(__file__).resolve().parent.parent
SAMUS_PATH = ROOT / "refs" / "samus_helmet.jpg"
BOCA_PATH = ROOT / "refs" / "bombonera.webp"
OUT_RS = ROOT / "src" / "samus_bombonera_data.rs"
OUT_MD = ROOT / "docs" / "medidas-samus-bombonera.md"


def median_rgb(pixels: list[tuple[int, int, int]]) -> tuple[int, int, int]:
    rs = sorted(p[0] for p in pixels)
    gs = sorted(p[1] for p in pixels)
    bs = sorted(p[2] for p in pixels)
    n = len(pixels)
    return (rs[n // 2], gs[n // 2], bs[n // 2])


def rgb_hex(c: tuple[int, int, int]) -> str:
    return f"#{c[0]:02X}{c[1]:02X}{c[2]:02X}"


def rgb_f32(c: tuple[int, int, int]) -> str:
    return f"[{c[0] / 255.0:.4f}, {c[1] / 255.0:.4f}, {c[2] / 255.0:.4f}]"


def rdp_open(pts: list[tuple[float, float]], eps: float) -> list[tuple[float, float]]:
    if len(pts) <= 2:
        return pts
    ax, ay = pts[0]
    bx, by = pts[-1]
    abx, aby = bx - ax, by - ay
    ab2 = abx * abx + aby * aby
    best_i, best_d = 0, -1.0
    for i in range(1, len(pts) - 1):
        px, py = pts[i]
        if ab2 < 1e-12:
            d = math.hypot(px - ax, py - ay)
        else:
            t = max(0.0, min(1.0, ((px - ax) * abx + (py - ay) * aby) / ab2))
            d = math.hypot(px - (ax + t * abx), py - (ay + t * aby))
        if d > best_d:
            best_d = d
            best_i = i
    if best_d > eps:
        left = rdp_open(pts[: best_i + 1], eps)
        right = rdp_open(pts[best_i:], eps)
        return left[:-1] + right
    return [pts[0], pts[-1]]


def rdp_closed(pts: list[tuple[float, float]], eps: float) -> list[tuple[float, float]]:
    if len(pts) <= 4:
        return pts
    p0 = pts[0]
    far_i = max(range(len(pts)), key=lambda i: (pts[i][0] - p0[0]) ** 2 + (pts[i][1] - p0[1]) ** 2)
    c1 = rdp_open(pts[: far_i + 1], eps)
    c2 = rdp_open(pts[far_i:] + [pts[0]], eps)
    out = c1[:-1] + c2[:-1]
    return out


def trace_contour(mask: list[list[bool]]) -> list[tuple[int, int]]:
    h, w = len(mask), len(mask[0])
    start = None
    for y in range(1, h - 1):
        for x in range(1, w - 1):
            if mask[y][x] and not mask[y][x - 1]:
                start = (x, y)
                break
        if start:
            break
    if not start:
        return []
    dirs = [(1, 0), (1, 1), (0, 1), (-1, 1), (-1, 0), (-1, -1), (0, -1), (1, -1)]
    contour = [start]
    cur = start
    prev_dir = 4
    for _ in range(w * h):
        found = None
        base = (prev_dir + 1) % 8
        for k in range(8):
            d = (base + k) % 8
            nx, ny = cur[0] + dirs[d][0], cur[1] + dirs[d][1]
            if 0 <= nx < w and 0 <= ny < h and mask[ny][nx]:
                found = (nx, ny)
                prev_dir = (d + 4) % 8
                break
        if not found or (found == start and len(contour) > 4):
            break
        contour.append(found)
        cur = found
    return contour


def zhang_suen_thin(grid: list[list[int]]) -> list[list[int]]:
    h, w = len(grid), len(grid[0])
    img = [r[:] for r in grid]
    changed = True
    while changed:
        changed = False
        for step in (0, 1):
            to_del = []
            for y in range(1, h - 1):
                for x in range(1, w - 1):
                    if img[y][x] != 1:
                        continue
                    p2, p3, p4 = img[y - 1][x], img[y - 1][x + 1], img[y][x + 1]
                    p5, p6, p7 = img[y + 1][x + 1], img[y + 1][x], img[y + 1][x - 1]
                    p8, p9 = img[y][x - 1], img[y - 1][x - 1]
                    nbrs = [p2, p3, p4, p5, p6, p7, p8, p9]
                    b = sum(nbrs)
                    if not (2 <= b <= 6):
                        continue
                    a = sum(1 for i in range(8) if nbrs[i] == 0 and nbrs[(i + 1) % 8] == 1)
                    if a != 1:
                        continue
                    if step == 0:
                        if p2 * p4 * p6 == 0 and p4 * p6 * p8 == 0:
                            to_del.append((x, y))
                    else:
                        if p2 * p4 * p8 == 0 and p2 * p6 * p8 == 0:
                            to_del.append((x, y))
            if to_del:
                changed = True
                for x, y in to_del:
                    img[y][x] = 0
    return img


def extract_polylines(skel: list[list[int]]) -> list[list[tuple[int, int]]]:
    h, w = len(skel), len(skel[0])
    seen = [[False] * w for _ in range(h)]
    lines = []
    for y in range(1, h - 1):
        for x in range(1, w - 1):
            if skel[y][x] == 1 and not seen[y][x]:
                comp = []
                q = deque([(x, y)])
                seen[y][x] = True
                while q:
                    cx, cy = q.popleft()
                    comp.append((cx, cy))
                    for dx in (-1, 0, 1):
                        for dy in (-1, 0, 1):
                            nx, ny = cx + dx, cy + dy
                            if 0 <= nx < w and 0 <= ny < h and skel[ny][nx] == 1 and not seen[ny][nx]:
                                seen[ny][nx] = True
                                q.append((nx, ny))
                if len(comp) >= 15:
                    comp.sort(key=lambda p: p[1])
                    lines.append(comp)
    lines.sort(key=lambda c: sum(p[0] for p in c) / len(c))
    return lines


def measure_samus() -> dict:
    img = Image.open(SAMUS_PATH).convert("RGB")
    w, h = img.size
    px = img.load()

    fg = [[not (px[x, y][0] > 232 and px[x, y][1] > 232 and px[x, y][2] > 232) for x in range(w)] for y in range(h)]
    fg_pts = [(x, y) for y in range(h) for x in range(w) if fg[y][x]]
    x0, x1 = min(p[0] for p in fg_pts), max(p[0] for p in fg_pts)
    y0, y1 = min(p[1] for p in fg_pts), max(p[1] for p in fg_pts)
    cx = 0.5 * (x0 + x1)
    cy = 0.5 * (y0 + y1)
    scale = 2.0 / float(y1 - y0)

    def to_norm(x: float, y: float) -> tuple[float, float]:
        return ((x - cx) * scale, (cy - y) * scale)

    # Perfil radial frontal de la cúpula (48 ángulos) medido desde (cx, cy).
    n_ang = 48
    radial_profile = []
    for i in range(n_ang):
        a = 2.0 * math.pi * i / n_ang
        ca, sa = math.cos(a), math.sin(a)
        max_r = 0.0
        for r in range(20, max(w, h)):
            sx = int(round(cx + r * ca))
            sy = int(round(cy - r * sa))
            if not (0 <= sx < w and 0 <= sy < h):
                break
            if fg[sy][sx]:
                max_r = r * scale
        radial_profile.append(max_r)

    # Visor verde claro: tomar la componente conexa más grande.
    visor_raw = [
        [
            fg[y][x] and px[x, y][1] > 185 and px[x, y][1] >= px[x, y][0] - 8 and px[x, y][1] > px[x, y][2] + 10
            for x in range(w)
        ]
        for y in range(h)
    ]
    seen_v = [[False] * w for _ in range(h)]
    best_v: list[tuple[int, int]] = []
    for y in range(h):
        for x in range(w):
            if visor_raw[y][x] and not seen_v[y][x]:
                comp = []
                q = deque([(x, y)])
                seen_v[y][x] = True
                while q:
                    cx_v, cy_v = q.popleft()
                    comp.append((cx_v, cy_v))
                    for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
                        nx, ny = cx_v + dx, cy_v + dy
                        if 0 <= nx < w and 0 <= ny < h and visor_raw[ny][nx] and not seen_v[ny][nx]:
                            seen_v[ny][nx] = True
                            q.append((nx, ny))
                if len(comp) > len(best_v):
                    best_v = comp
    visor_mask = [[False] * w for _ in range(h)]
    for vx, vy in best_v:
        visor_mask[vy][vx] = True
    visor_contour_px = trace_contour(visor_mask)
    visor_norm = [to_norm(x, y) for x, y in visor_contour_px]
    visor_poly = rdp_closed(visor_norm, 0.022)

    # Tres surcos oscuros en la parte superior de la cúpula.
    groove_grid = [
        [
            1
            if (
                fg[y][x]
                and y < int(y0 + 0.48 * (y1 - y0))
                and max(px[x, y]) < 95
            )
            else 0
            for x in range(w)
        ]
        for y in range(h)
    ]
    skel = zhang_suen_thin(groove_grid)
    groove_lines_px = extract_polylines(skel)[:3]
    grooves_norm = []
    for line in groove_lines_px:
        # Submuestrear ordenados por Y y simplificar con RDP.
        pts = [to_norm(x, y) for x, y in line[::3]]
        if len(pts) >= 2:
            grooves_norm.append(rdp_open(pts, 0.02))

    # Colores por mediana.
    armor_px = [
        px[x, y]
        for x, y in fg_pts
        if px[x, y][0] > 125 and px[x, y][0] > px[x, y][1] + 25 and px[x, y][1] > px[x, y][2]
    ]
    groove_px = [
        px[x, y]
        for x, y in fg_pts
        if y < int(y0 + 0.48 * (y1 - y0)) and max(px[x, y]) < 95
    ]
    visor_c_px = [
        px[x, y]
        for x, y in fg_pts
        if px[x, y][1] > 205 and px[x, y][1] >= px[x, y][0] and px[x, y][1] > px[x, y][2] + 15
    ]
    visor_e_px = [
        px[x, y]
        for x, y in fg_pts
        if px[x, y][1] > px[x, y][0] + 25 and px[x, y][1] > px[x, y][2] + 25 and px[x, y][1] < 200
    ]
    hose_px = [
        px[x, y]
        for x, y in fg_pts
        if y > int(y0 + 0.62 * (y1 - y0)) and abs(px[x, y][0] - px[x, y][2]) < 22 and max(px[x, y]) < 145
    ]
    hose_sorted = sorted(hose_px, key=lambda c: sum(c))
    hose_a = hose_sorted[int(0.28 * len(hose_sorted))]
    hose_b = hose_sorted[int(0.72 * len(hose_sorted))]

    # Orejera derecha (x > cx + 0.35*(x1-x0)/2, y en zona media).
    ear_y_min, ear_y_max = int(y0 + 0.34 * (y1 - y0)), int(y0 + 0.62 * (y1 - y0))
    ear_x_max = max(x for y in range(ear_y_min, ear_y_max) for x in range(int(cx), w) if fg[y][x])
    ear_center_x = (ear_x_max - cx) * scale * 0.78
    ear_center_y = to_norm(cx, 0.5 * (ear_y_min + ear_y_max))[1]
    ear_ry = 0.5 * (ear_y_max - ear_y_min) * scale * 0.92
    ear_rx = (ear_x_max - cx) * scale * 0.28

    # Mangueras: línea central aproximada medida de los píxeles oscuros inferiores.
    hose_r_pts = [(x, y) for x, y in fg_pts if x > cx + 0.12 * (x1 - x0) and y > int(y0 + 0.64 * (y1 - y0)) and max(px[x, y]) < 145]
    hose_centerline = []
    for k in range(5):
        yk0 = int(y0 + (0.66 + k * 0.06) * (y1 - y0))
        yk1 = yk0 + int(0.05 * (y1 - y0))
        band = [p for p in hose_r_pts if yk0 <= p[1] <= yk1]
        if band:
            mx = sum(p[0] for p in band) / len(band)
            my = sum(p[1] for p in band) / len(band)
            hose_centerline.append(to_norm(mx, my))

    return {
        "w": w,
        "h": h,
        "half_w": 0.5 * (x1 - x0) * scale,
        "radial": radial_profile,
        "visor_poly": visor_poly,
        "grooves": grooves_norm,
        "armor_rgb": median_rgb(armor_px),
        "groove_rgb": median_rgb(groove_px),
        "visor_c_rgb": median_rgb(visor_c_px),
        "visor_e_rgb": median_rgb(visor_e_px) if visor_e_px else (88, 184, 88),
        "hose_a_rgb": hose_a,
        "hose_b_rgb": hose_b,
        "ear_center": (ear_center_x, ear_center_y),
        "ear_radii": (ear_rx, ear_ry),
        "hose_line": hose_centerline,
    }


def measure_bombonera() -> dict:
    img = Image.open(BOCA_PATH).convert("RGB")
    w, h = img.size
    px = img.load()

    blue_px, gold_px, concrete_px, zocalo_px, grass_px = [], [], [], [], []
    for y in range(0, h, 2):
        for x in range(0, w, 2):
            r, g, b = px[x, y]
            # Excluir fondo gris muy uniforme claro (> 218).
            if r > 218 and g > 218 and b > 218:
                continue
            if b > r + 55 and b > g + 25 and max(r, g, b) < 200:
                blue_px.append((r, g, b))
            elif r > 175 and g > 135 and b < 70:
                gold_px.append((r, g, b))
            elif g > r + 8 and g > b + 20:
                grass_px.append((r, g, b))
            elif abs(r - g) < 18 and abs(g - b) < 18:
                if 125 <= r <= 185:
                    concrete_px.append((r, g, b))
                elif 185 < r <= 215:
                    zocalo_px.append((r, g, b))

    grass_sorted = sorted(grass_px, key=lambda c: c[1])
    grass_dark = grass_sorted[int(0.28 * len(grass_sorted))]
    grass_light = grass_sorted[int(0.72 * len(grass_sorted))]

    # Corte perpendicular en la grada curva superior-izquierda para medir proporción de 3 niveles.
    # Se escanea una línea vertical en x = 0.44 * w desde el borde superior (y=0.12*h..0.42*h).
    sx = int(0.44 * w)
    low_b, mid_y, top_b = 0, 0, 0
    seen_top_blue = False
    seen_mid_gold = False
    for y in range(int(0.12 * h), int(0.42 * h)):
        r, g, b = px[sx, y]
        is_blue = b > r + 30 and b > g + 10
        is_gold = r > 150 and g > 115 and b < 90
        if not seen_mid_gold and is_blue:
            top_b += 1
            seen_top_blue = True
        elif seen_top_blue and is_gold:
            mid_y += 1
            seen_mid_gold = True
        elif seen_mid_gold and is_blue:
            low_b += 1
    tot = max(1, low_b + mid_y + top_b)
    return {
        "w": w,
        "h": h,
        "blue_rgb": median_rgb(blue_px),
        "gold_rgb": median_rgb(gold_px),
        "concrete_rgb": median_rgb(concrete_px),
        "zocalo_rgb": median_rgb(zocalo_px),
        "grass_dark_rgb": grass_dark,
        "grass_light_rgb": grass_light,
        "tier_fracs": (low_b / tot, mid_y / tot, top_b / tot),
        "tier_px": (low_b, mid_y, top_b),
    }


def main() -> None:
    samus = measure_samus()
    boca = measure_bombonera()

    lines = [
        "#![allow(dead_code)]",
        "//! Datos geométricos y paleta medidos desde `refs/samus_helmet.jpg` y `refs/bombonera.webp`.",
        "//!",
        "//! Cómo regenerar:",
        "//!     python3 tools/measure_samus_bombonera.py",
        "",
        f"pub const SAMUS_HALF_WIDTH: f32 = {samus['half_w']:.5f};",
        f"pub const SAMUS_ARMOR_RGB: [f32; 3] = {rgb_f32(samus['armor_rgb'])}; // {rgb_hex(samus['armor_rgb'])}",
        f"pub const SAMUS_GROOVE_RGB: [f32; 3] = {rgb_f32(samus['groove_rgb'])}; // {rgb_hex(samus['groove_rgb'])}",
        f"pub const SAMUS_VISOR_CENTER_RGB: [f32; 3] = {rgb_f32(samus['visor_c_rgb'])}; // {rgb_hex(samus['visor_c_rgb'])}",
        f"pub const SAMUS_VISOR_EDGE_RGB: [f32; 3] = {rgb_f32(samus['visor_e_rgb'])}; // {rgb_hex(samus['visor_e_rgb'])}",
        f"pub const SAMUS_HOSE_A_RGB: [f32; 3] = {rgb_f32(samus['hose_a_rgb'])}; // {rgb_hex(samus['hose_a_rgb'])}",
        f"pub const SAMUS_HOSE_B_RGB: [f32; 3] = {rgb_f32(samus['hose_b_rgb'])}; // {rgb_hex(samus['hose_b_rgb'])}",
        f"pub const SAMUS_EAR_CENTER: [f32; 2] = [{samus['ear_center'][0]:.5f}, {samus['ear_center'][1]:.5f}];",
        f"pub const SAMUS_EAR_RADII: [f32; 2] = [{samus['ear_radii'][0]:.5f}, {samus['ear_radii'][1]:.5f}];",
        "",
        "pub const SAMUS_DOME_RADIAL_PROFILE: &[f32] = &[",
    ]
    for r in samus["radial"]:
        lines.append(f"    {r:.5f},")
    lines.append("];")
    lines.append("")
    lines.append("pub const SAMUS_VISOR_POLY: &[[f32; 2]] = &[")
    for x, y in samus["visor_poly"]:
        lines.append(f"    [{x:.5f}, {y:.5f}],")
    lines.append("];")
    lines.append("")
    for gi, g in enumerate(samus["grooves"]):
        lines.append(f"pub const SAMUS_GROOVE_{gi}: &[[f32; 2]] = &[")
        for x, y in g:
            lines.append(f"    [{x:.5f}, {y:.5f}],")
        lines.append("];")
    lines.append("pub const SAMUS_GROOVES: &[&[[f32; 2]]] = &[")
    for gi in range(len(samus["grooves"])):
        lines.append(f"    SAMUS_GROOVE_{gi},")
    lines.append("];")
    lines.append("")
    lines.append("pub const SAMUS_HOSE_RIGHT_LINE: &[[f32; 2]] = &[")
    for x, y in samus["hose_line"]:
        lines.append(f"    [{x:.5f}, {y:.5f}],")
    lines.append("];")
    lines.append("")
    lines.extend([
        f"pub const BOCA_BLUE_RGB: [f32; 3] = {rgb_f32(boca['blue_rgb'])}; // {rgb_hex(boca['blue_rgb'])}",
        f"pub const BOCA_GOLD_RGB: [f32; 3] = {rgb_f32(boca['gold_rgb'])}; // {rgb_hex(boca['gold_rgb'])}",
        f"pub const BOCA_CONCRETE_RGB: [f32; 3] = {rgb_f32(boca['concrete_rgb'])}; // {rgb_hex(boca['concrete_rgb'])}",
        f"pub const BOCA_ZOCALO_RGB: [f32; 3] = {rgb_f32(boca['zocalo_rgb'])}; // {rgb_hex(boca['zocalo_rgb'])}",
        f"pub const BOCA_GRASS_DARK_RGB: [f32; 3] = {rgb_f32(boca['grass_dark_rgb'])}; // {rgb_hex(boca['grass_dark_rgb'])}",
        f"pub const BOCA_GRASS_LIGHT_RGB: [f32; 3] = {rgb_f32(boca['grass_light_rgb'])}; // {rgb_hex(boca['grass_light_rgb'])}",
        f"pub const BOCA_TIER_PROPORTIONS: [f32; 3] = [{boca['tier_fracs'][0]:.4f}, {boca['tier_fracs'][1]:.4f}, {boca['tier_fracs'][2]:.4f}];",
    ])
    OUT_RS.write_text("\n".join(lines) + "\n", encoding="utf-8")

    md = f"""# Medidas de Samus (`refs/samus_helmet.jpg`) y La Bombonera (`refs/bombonera.webp`)

## Casco de Samus ({samus['w']}×{samus['h']} px, normalizado a alto = 2.0)
- **Perfil radial frontal**: 48 ángulos alrededor del centro de silueta; semiancho medido = `{samus['half_w']:.4f}`.
- **Visor**: polígono medido de `{len(samus['visor_poly'])}` vértices por componente verde claro.
- **Surcos de la cúpula**: `{len(samus['grooves'])}` polilíneas adelgazadas con Zhang–Suen y simplificadas con RDP.
- **Orejera derecha (espejada)**: centro `({samus['ear_center'][0]:.4f}, {samus['ear_center'][1]:.4f})`, semiejes `({samus['ear_radii'][0]:.4f}, {samus['ear_radii'][1]:.4f})`.
- **Paleta**: armadura `{rgb_hex(samus['armor_rgb'])}`, surcos `{rgb_hex(samus['groove_rgb'])}`, visor centro `{rgb_hex(samus['visor_c_rgb'])}`, visor borde `{rgb_hex(samus['visor_e_rgb'])}`, manguera `{rgb_hex(samus['hose_a_rgb'])}` / `{rgb_hex(samus['hose_b_rgb'])}`.

## La Bombonera ({boca['w']}×{boca['h']} px)
- **Proporción de las 3 bandejas** (corte perpendicular en `x = 0.44·W`, px `{boca['tier_px']}`): inferior azul `{boca['tier_fracs'][0]:.2%}`, media amarilla `{boca['tier_fracs'][1]:.2%}`, superior azul `{boca['tier_fracs'][2]:.2%}`.
- **Paleta**: azul `{rgb_hex(boca['blue_rgb'])}`, amarillo `{rgb_hex(boca['gold_rgb'])}`, concreto palcos `{rgb_hex(boca['concrete_rgb'])}`, zócalo `{rgb_hex(boca['zocalo_rgb'])}`, césped `{rgb_hex(boca['grass_light_rgb'])}` / `{rgb_hex(boca['grass_dark_rgb'])}`.
"""
    OUT_MD.write_text(md, encoding="utf-8")
    print("Samus:", len(samus["visor_poly"]), "visor pts,", len(samus["grooves"]), "grooves,", "armor=", rgb_hex(samus["armor_rgb"]), "visor=", rgb_hex(samus["visor_c_rgb"]))
    print("Bombonera:", "blue=", rgb_hex(boca["blue_rgb"]), "gold=", rgb_hex(boca["gold_rgb"]), "tiers=", boca["tier_px"])


if __name__ == "__main__":
    main()
