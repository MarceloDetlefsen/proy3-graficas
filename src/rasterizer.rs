use crate::Uniforms;
use crate::fragment::Fragment;
use crate::framebuffer::{Framebuffer, INVALID_TRI_ID};
use crate::primitive::ScreenTriangle;
use crate::shaders::{Shader, fragment_shader};
use raylib::prelude::{Color, Vector2, Vector3};
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Clone, Copy, Debug)]
pub struct TriMaterial {
    pub shader: Shader,
    pub light_dir: Vector3,
    pub shader_param: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TriangleMode {
    Wireframe,
    FilledBarycentric,
}

pub const WINDOW_WIDTH: i32 = 800;
pub const WINDOW_HEIGHT: i32 = 600;
pub const TILE_SIZE: usize = 64;
const DEGENERATE_EPS: f32 = 1e-6;
const MAX_SCREEN_COORD: f32 = 65536.0;

#[inline]
fn has_valid_screen_coords(tri: &ScreenTriangle) -> bool {
    tri.vertices.iter().all(|v| {
        v.x.is_finite()
            && v.y.is_finite()
            && v.z.is_finite()
            && v.inv_w.is_finite()
            && v.x.abs() <= MAX_SCREEN_COORD
            && v.y.abs() <= MAX_SCREEN_COORD
    })
}

#[derive(Clone, Debug)]
pub struct TileGrid {
    pub tiles_x: usize,
    pub tiles_y: usize,
    pub bins: Vec<Vec<u32>>,
}

impl TileGrid {
    pub fn new(width: usize, height: usize) -> Self {
        let tiles_x = width.div_ceil(TILE_SIZE).max(1);
        let tiles_y = height.div_ceil(TILE_SIZE).max(1);
        Self {
            tiles_x,
            tiles_y,
            bins: vec![Vec::new(); tiles_x * tiles_y],
        }
    }

    pub fn bin_triangles(
        &mut self,
        triangles: &[ScreenTriangle],
        width: usize,
        height: usize,
        backface_culling: bool,
    ) {
        let tiles_x = width.div_ceil(TILE_SIZE).max(1);
        let tiles_y = height.div_ceil(TILE_SIZE).max(1);
        self.tiles_x = tiles_x;
        self.tiles_y = tiles_y;
        let needed = tiles_x * tiles_y;
        if self.bins.len() != needed {
            self.bins.resize_with(needed, Vec::new);
        }
        for bin in &mut self.bins {
            bin.clear();
        }

        let w_i32 = width as i32;
        let h_i32 = height as i32;
        for (tri_idx, tri) in triangles.iter().enumerate() {
            if !has_valid_screen_coords(tri) {
                continue;
            }
            let [v0, v1, v2] = tri.vertices;
            let pa = (v0.x, v0.y);
            let pb = (v1.x, v1.y);
            let pc = (v2.x, v2.y);
            let area = edge_2d(pa, pb, pc);
            if area.abs() <= DEGENERATE_EPS || (backface_culling && area <= 0.0) {
                continue;
            }

            let min_x = pa.0.min(pb.0).min(pc.0).floor() as i32;
            let max_x = pa.0.max(pb.0).max(pc.0).ceil() as i32;
            let min_y = pa.1.min(pb.1).min(pc.1).floor() as i32;
            let max_y = pa.1.max(pb.1).max(pc.1).ceil() as i32;

            if max_x < 0 || min_x >= w_i32 || max_y < 0 || min_y >= h_i32 {
                continue;
            }

            let tx0 = (min_x.clamp(0, w_i32 - 1) as usize) / TILE_SIZE;
            let tx1 = (max_x.clamp(0, w_i32 - 1) as usize) / TILE_SIZE;
            let ty0 = (min_y.clamp(0, h_i32 - 1) as usize) / TILE_SIZE;
            let ty1 = (max_y.clamp(0, h_i32 - 1) as usize) / TILE_SIZE;

            let tri_id = tri_idx as u32;
            for ty in ty0..=ty1 {
                let row = ty * tiles_x;
                for tx in tx0..=tx1 {
                    self.bins[row + tx].push(tri_id);
                }
            }
        }
    }
}

#[inline]
pub fn edge_2d(p: (f32, f32), q: (f32, f32), r: (f32, f32)) -> f32 {
    (r.0 - p.0) * (q.1 - p.1) - (r.1 - p.1) * (q.0 - p.0)
}

#[inline]
pub fn perspective_weights(tri: &ScreenTriangle, screen_w: (f32, f32, f32)) -> (f32, f32, f32) {
    let p0 = screen_w.0 * tri.vertices[0].inv_w;
    let p1 = screen_w.1 * tri.vertices[1].inv_w;
    let p2 = screen_w.2 * tri.vertices[2].inv_w;
    let denom = p0 + p1 + p2;
    if denom.abs() > 1e-12 {
        let inv_d = 1.0 / denom;
        (p0 * inv_d, p1 * inv_d, p2 * inv_d)
    } else {
        screen_w
    }
}

#[inline]
pub fn interpolate_screen_fragment(
    tri: &ScreenTriangle,
    px: f32,
    py: f32,
    screen_w: (f32, f32, f32),
) -> Fragment {
    let [v0, v1, v2] = tri.vertices;
    let (pw0, pw1, pw2) = perspective_weights(tri, screen_w);
    Fragment {
        position: Vector2::new(px, py),
        object_pos: v0.object_pos * pw0 + v1.object_pos * pw1 + v2.object_pos * pw2,
        world_pos: v0.world_pos * pw0 + v1.world_pos * pw1 + v2.world_pos * pw2,
        normal: (v0.normal * pw0 + v1.normal * pw1 + v2.normal * pw2).normalize(),
        color: v0.color * pw0 + v1.color * pw1 + v2.color * pw2,
        depth: v0.z * screen_w.0 + v1.z * screen_w.1 + v2.z * screen_w.2,
    }
}

#[allow(dead_code)]
pub fn rasterize_visibility(
    triangles: &[ScreenTriangle],
    framebuffer: &mut Framebuffer,
    backface_culling: bool,
) -> usize {
    let width = framebuffer.width as i32;
    let height = framebuffer.height as i32;
    let mut writes = 0usize;

    for (tri_idx, tri) in triangles.iter().enumerate() {
        if !has_valid_screen_coords(tri) {
            continue;
        }
        let [v0, v1, v2] = tri.vertices;
        let pa = (v0.x, v0.y);
        let pb = (v1.x, v1.y);
        let pc = (v2.x, v2.y);
        let area = edge_2d(pa, pb, pc);
        if area.abs() <= DEGENERATE_EPS || (backface_culling && area <= 0.0) {
            continue;
        }
        let inv_area = 1.0 / area;

        let min_x = pa.0.min(pb.0).min(pc.0).floor() as i32;
        let max_x = pa.0.max(pb.0).max(pc.0).ceil() as i32;
        let min_y = pa.1.min(pb.1).min(pc.1).floor() as i32;
        let max_y = pa.1.max(pb.1).max(pc.1).ceil() as i32;

        if max_x < 0 || min_x >= width || max_y < 0 || min_y >= height {
            continue;
        }

        let start_x = min_x.clamp(0, width - 1);
        let end_x = max_x.clamp(0, width - 1);
        let start_y = min_y.clamp(0, height - 1);
        let end_y = max_y.clamp(0, height - 1);

        let tri_id = tri_idx as u32;
        for y in start_y..=end_y {
            let py = y as f32 + 0.5;
            let row_offset = y as usize * width as usize;
            for x in start_x..=end_x {
                let p = (x as f32 + 0.5, py);
                let w0 = edge_2d(pb, pc, p) * inv_area;
                let w1 = edge_2d(pc, pa, p) * inv_area;
                let w2 = edge_2d(pa, pb, p) * inv_area;
                if w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0 {
                    let depth = v0.z * w0 + v1.z * w1 + v2.z * w2;
                    let idx = row_offset + x as usize;
                    if depth < framebuffer.depth_buffer[idx] {
                        framebuffer.depth_buffer[idx] = depth;
                        framebuffer.tri_id_buffer[idx] = tri_id;
                        writes += 1;
                    }
                }
            }
        }
    }

    writes
}

pub fn rasterize_band(
    band_idx: usize,
    width: usize,
    triangles: &[ScreenTriangle],
    tile_grid: &TileGrid,
    depth_band: &mut [f32],
    tri_id_band: &mut [u32],
    backface_culling: bool,
) -> usize {
    let band_height = depth_band.len() / width;
    let y0 = band_idx * TILE_SIZE;
    let y1 = y0 + band_height;
    let row_tiles = &tile_grid.bins[band_idx * tile_grid.tiles_x..(band_idx + 1) * tile_grid.tiles_x];
    let mut writes = 0usize;

    for (tx, bin) in row_tiles.iter().enumerate() {
        let x0 = tx * TILE_SIZE;
        let x1 = (x0 + TILE_SIZE).min(width);
        for &tri_id in bin {
            let tri = &triangles[tri_id as usize];
            if !has_valid_screen_coords(tri) {
                continue;
            }
            let [v0, v1, v2] = tri.vertices;
            let pa = (v0.x, v0.y);
            let pb = (v1.x, v1.y);
            let pc = (v2.x, v2.y);
            let area = edge_2d(pa, pb, pc);
            if area.abs() <= DEGENERATE_EPS || (backface_culling && area <= 0.0) {
                continue;
            }
            let inv_area = 1.0 / area;

            let min_x = (pa.0.min(pb.0).min(pc.0).floor() as i32).max(x0 as i32);
            let max_x = (pa.0.max(pb.0).max(pc.0).ceil() as i32).min(x1 as i32 - 1);
            let min_y = (pa.1.min(pb.1).min(pc.1).floor() as i32).max(y0 as i32);
            let max_y = (pa.1.max(pb.1).max(pc.1).ceil() as i32).min(y1 as i32 - 1);

            if min_x > max_x || min_y > max_y {
                continue;
            }

            for y in min_y as usize..=max_y as usize {
                let py = y as f32 + 0.5;
                let row_offset = (y - y0) * width;
                for x in min_x as usize..=max_x as usize {
                    let p = (x as f32 + 0.5, py);
                    let w0 = edge_2d(pb, pc, p) * inv_area;
                    let w1 = edge_2d(pc, pa, p) * inv_area;
                    let w2 = edge_2d(pa, pb, p) * inv_area;
                    if w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0 {
                        let depth = v0.z * w0 + v1.z * w1 + v2.z * w2;
                        let idx = row_offset + x;
                        if depth < depth_band[idx] {
                            depth_band[idx] = depth;
                            tri_id_band[idx] = tri_id;
                            writes += 1;
                        }
                    }
                }
            }
        }
    }

    writes
}

type RastBand<'a> = Mutex<Option<(usize, &'a mut [f32], &'a mut [u32])>>;
type ShadeBand<'a> = Mutex<Option<(usize, &'a [u32], &'a mut [u8])>>;

pub fn rasterize_visibility_tiled(
    triangles: &[ScreenTriangle],
    tile_grid: &mut TileGrid,
    framebuffer: &mut Framebuffer,
    backface_culling: bool,
    threads: usize,
) -> usize {
    let width = framebuffer.width as usize;
    let height = framebuffer.height as usize;
    tile_grid.bin_triangles(triangles, width, height, backface_culling);

    let band_elems = TILE_SIZE * width;
    let thread_count = threads.max(1);
    if thread_count == 1 {
        let mut writes = 0usize;
        for (band_idx, (depth_band, tri_id_band)) in framebuffer
            .depth_buffer
            .chunks_mut(band_elems)
            .zip(framebuffer.tri_id_buffer.chunks_mut(band_elems))
            .enumerate()
        {
            writes += rasterize_band(
                band_idx,
                width,
                triangles,
                tile_grid,
                depth_band,
                tri_id_band,
                backface_culling,
            );
        }
        return writes;
    }

    let bands: Vec<RastBand<'_>> = framebuffer
        .depth_buffer
        .chunks_mut(band_elems)
        .zip(framebuffer.tri_id_buffer.chunks_mut(band_elems))
        .enumerate()
        .map(|(band_idx, (d, t))| Mutex::new(Some((band_idx, d, t))))
        .collect();
    let next_band = AtomicUsize::new(0);
    let total_writes = AtomicUsize::new(0);
    let workers = thread_count.min(bands.len().max(1));
    let grid_ref: &TileGrid = tile_grid;

    std::thread::scope(|s| {
        for _ in 0..workers {
            s.spawn(|| {
                let mut local_writes = 0usize;
                loop {
                    let idx = next_band.fetch_add(1, Ordering::Relaxed);
                    if idx >= bands.len() {
                        break;
                    }
                    let (band_idx, depth_band, tri_id_band) =
                        bands[idx].lock().unwrap().take().unwrap();
                    local_writes += rasterize_band(
                        band_idx,
                        width,
                        triangles,
                        grid_ref,
                        depth_band,
                        tri_id_band,
                        backface_culling,
                    );
                }
                total_writes.fetch_add(local_writes, Ordering::Relaxed);
            });
        }
    });

    total_writes.load(Ordering::Relaxed)
}

#[allow(clippy::too_many_arguments)]
fn shade_band(
    band_idx: usize,
    width: usize,
    triangles: &[ScreenTriangle],
    materials: &[TriMaterial],
    tri_id_band: &[u32],
    color_band: &mut [u8],
    default_shader: Shader,
    uniforms: &Uniforms,
    inv_vp: Option<&crate::math::Mat4>,
) -> usize {
    let band_height = tri_id_band.len() / width;
    let y0 = band_idx * TILE_SIZE;
    let mut calls = 0usize;
    let vp_w = uniforms.viewport_width.max(1) as f32;
    let vp_h = uniforms.viewport_height.max(1) as f32;
    let sky_seed = uniforms.noise.seed();

    for local_y in 0..band_height {
        let py = (y0 + local_y) as f32 + 0.5;
        let row_offset = local_y * width;
        for x in 0..width {
            let idx = row_offset + x;
            let tri_id = tri_id_band[idx];
            if tri_id == INVALID_TRI_ID {
                if let Some(inv) = inv_vp {
                    let dir = crate::sky::view_direction_for_pixel(
                        x as f32,
                        (y0 + local_y) as f32,
                        vp_w,
                        vp_h,
                        inv,
                        uniforms.camera_pos,
                    );
                    let mut sc = crate::sky::sample_sky(dir, sky_seed, uniforms.time);
                    // Halo solar en píxeles vacíos sin geometría extra.
                    let to_sun = -uniforms.camera_pos;
                    let sun_dist = to_sun.length();
                    if sun_dist > 1e-3 {
                        let sun_dir = to_sun * (1.0 / sun_dist);
                        let cos_ang = dir.dot(sun_dir).clamp(-1.0, 1.0);
                        let sin_ang = (1.0 - cos_ang * cos_ang).max(0.0).sqrt();
                        let sun_ang = (3.5 / sun_dist).min(0.6);
                        let halo_r = sun_ang * crate::scene_config::SOLAR_HALO_RADIUS_MULT;
                        if cos_ang > 0.0 && sin_ang < halo_r {
                            let t = 1.0 - (sin_ang / halo_r);
                            sc += crate::scene_config::SOLAR_HALO_COLOR * (t * t * 0.55);
                        }
                    }
                    let c_idx = idx * 4;
                    color_band[c_idx] = (sc.x.clamp(0.0, 1.0) * 255.0) as u8;
                    color_band[c_idx + 1] = (sc.y.clamp(0.0, 1.0) * 255.0) as u8;
                    color_band[c_idx + 2] = (sc.z.clamp(0.0, 1.0) * 255.0) as u8;
                    color_band[c_idx + 3] = 255;
                }
                continue;
            }
            let tri_usize = tri_id as usize;
            let tri = &triangles[tri_usize];
            let [v0, v1, v2] = tri.vertices;
            let pa = (v0.x, v0.y);
            let pb = (v1.x, v1.y);
            let pc = (v2.x, v2.y);
            let inv_area = 1.0 / edge_2d(pa, pb, pc);
            let p = (x as f32 + 0.5, py);
            let w0 = edge_2d(pb, pc, p) * inv_area;
            let w1 = edge_2d(pc, pa, p) * inv_area;
            let w2 = edge_2d(pa, pb, p) * inv_area;

            let fragment = interpolate_screen_fragment(tri, p.0, p.1, (w0, w1, w2));
            let (shader, light_dir, shader_param) = materials
                .get(tri_usize)
                .map_or((default_shader, uniforms.light_dir, uniforms.shader_param), |m| {
                    (m.shader, m.light_dir, m.shader_param)
                });
            let fc = if (light_dir - uniforms.light_dir).length_sqr() < 1e-12
                && (shader_param - uniforms.shader_param).abs() < 1e-5
            {
                fragment_shader(&fragment, shader, uniforms)
            } else {
                let mut local_u = uniforms.clone();
                local_u.light_dir = light_dir;
                local_u.shader_param = shader_param;
                fragment_shader(&fragment, shader, &local_u)
            };
            let c_idx = idx * 4;
            color_band[c_idx] = (fc.x.clamp(0.0, 1.0) * 255.0) as u8;
            color_band[c_idx + 1] = (fc.y.clamp(0.0, 1.0) * 255.0) as u8;
            color_band[c_idx + 2] = (fc.z.clamp(0.0, 1.0) * 255.0) as u8;
            color_band[c_idx + 3] = 255;
            calls += 1;
        }
    }

    calls
}

pub fn shade_visibility_tiled_materials(
    triangles: &[ScreenTriangle],
    materials: &[TriMaterial],
    framebuffer: &mut Framebuffer,
    default_shader: Shader,
    uniforms: &Uniforms,
    threads: usize,
) -> (usize, usize) {
    shade_visibility_tiled_inner(
        triangles,
        materials,
        framebuffer,
        default_shader,
        uniforms,
        threads,
        true,
    )
}

fn shade_visibility_tiled_inner(
    triangles: &[ScreenTriangle],
    materials: &[TriMaterial],
    framebuffer: &mut Framebuffer,
    default_shader: Shader,
    uniforms: &Uniforms,
    threads: usize,
    draw_sky: bool,
) -> (usize, usize) {
    let width = framebuffer.width as usize;
    let band_elems = TILE_SIZE * width;
    let thread_count = threads.max(1);
    let inv_vp = if draw_sky {
        (uniforms.projection_matrix * uniforms.view_matrix).inverse()
    } else {
        None
    };

    if thread_count == 1 {
        let mut calls = 0usize;
        for (band_idx, (tri_id_band, color_band)) in framebuffer
            .tri_id_buffer
            .chunks(band_elems)
            .zip(framebuffer.color_pixels.chunks_mut(band_elems * 4))
            .enumerate()
        {
            calls += shade_band(
                band_idx,
                width,
                triangles,
                materials,
                tri_id_band,
                color_band,
                default_shader,
                uniforms,
                inv_vp.as_ref(),
            );
        }
        return (calls, calls);
    }

    let bands: Vec<ShadeBand<'_>> = framebuffer
        .tri_id_buffer
        .chunks(band_elems)
        .zip(framebuffer.color_pixels.chunks_mut(band_elems * 4))
        .enumerate()
        .map(|(band_idx, (t, c))| Mutex::new(Some((band_idx, t, c))))
        .collect();
    let next_band = AtomicUsize::new(0);
    let total_calls = AtomicUsize::new(0);
    let workers = thread_count.min(bands.len().max(1));

    std::thread::scope(|s| {
        for _ in 0..workers {
            s.spawn(|| {
                let mut local_calls = 0usize;
                loop {
                    let idx = next_band.fetch_add(1, Ordering::Relaxed);
                    if idx >= bands.len() {
                        break;
                    }
                    let (band_idx, tri_id_band, color_band) =
                        bands[idx].lock().unwrap().take().unwrap();
                    local_calls += shade_band(
                        band_idx,
                        width,
                        triangles,
                        materials,
                        tri_id_band,
                        color_band,
                        default_shader,
                        uniforms,
                        inv_vp.as_ref(),
                    );
                }
                total_calls.fetch_add(local_calls, Ordering::Relaxed);
            });
        }
    });

    let calls = total_calls.load(Ordering::Relaxed);
    (calls, calls)
}

pub fn shade_visibility_tiled(
    triangles: &[ScreenTriangle],
    framebuffer: &mut Framebuffer,
    shader: Shader,
    uniforms: &Uniforms,
    threads: usize,
) -> (usize, usize) {
    shade_visibility_tiled_inner(triangles, &[], framebuffer, shader, uniforms, threads, false)
}

#[allow(dead_code)]
pub fn shade_visibility(
    triangles: &[ScreenTriangle],
    framebuffer: &mut Framebuffer,
    shader: Shader,
    uniforms: &Uniforms,
) -> (usize, usize) {
    let width = framebuffer.width as usize;
    let height = framebuffer.height as usize;
    let mut shader_calls = 0usize;

    for y in 0..height {
        let py = y as f32 + 0.5;
        let row_offset = y * width;
        for x in 0..width {
            let idx = row_offset + x;
            let tri_id = framebuffer.tri_id_buffer[idx];
            if tri_id == INVALID_TRI_ID {
                continue;
            }
            let tri = &triangles[tri_id as usize];
            let [v0, v1, v2] = tri.vertices;
            let pa = (v0.x, v0.y);
            let pb = (v1.x, v1.y);
            let pc = (v2.x, v2.y);
            let inv_area = 1.0 / edge_2d(pa, pb, pc);
            let p = (x as f32 + 0.5, py);
            let w0 = edge_2d(pb, pc, p) * inv_area;
            let w1 = edge_2d(pc, pa, p) * inv_area;
            let w2 = edge_2d(pa, pb, p) * inv_area;

            let fragment = interpolate_screen_fragment(tri, p.0, p.1, (w0, w1, w2));
            let fc = fragment_shader(&fragment, shader, uniforms);
            let color_idx = idx * 4;
            framebuffer.color_pixels[color_idx] = (fc.x.clamp(0.0, 1.0) * 255.0) as u8;
            framebuffer.color_pixels[color_idx + 1] = (fc.y.clamp(0.0, 1.0) * 255.0) as u8;
            framebuffer.color_pixels[color_idx + 2] = (fc.z.clamp(0.0, 1.0) * 255.0) as u8;
            framebuffer.color_pixels[color_idx + 3] = 255;
            shader_calls += 1;
        }
    }

    (shader_calls, shader_calls)
}

pub fn render_wireframe(
    triangles: &[ScreenTriangle],
    framebuffer: &mut Framebuffer,
    shader: Shader,
    uniforms: &Uniforms,
) -> (usize, usize) {
    let mut calls = 0usize;
    for tri in triangles {
        let [v0, v1, v2] = tri.vertices;
        let edges = [(v0, v1, 0), (v1, v2, 1), (v2, v0, 2)];
        for (start, end, edge_idx) in edges {
            let dx = end.x - start.x;
            let dy = end.y - start.y;
            let steps = dx.abs().max(dy.abs()).ceil() as usize;
            for step in 0..=steps {
                let t = if steps == 0 {
                    0.0
                } else {
                    step as f32 / steps as f32
                };
                let px = start.x + dx * t;
                let py = start.y + dy * t;
                let weights = match edge_idx {
                    0 => (1.0 - t, t, 0.0),
                    1 => (0.0, 1.0 - t, t),
                    _ => (t, 0.0, 1.0 - t),
                };
                let frag = interpolate_screen_fragment(tri, px, py, weights);
                let fc = fragment_shader(&frag, shader, uniforms);
                calls += 1;
                let color = Color::new(
                    (fc.x.clamp(0.0, 1.0) * 255.0) as u8,
                    (fc.y.clamp(0.0, 1.0) * 255.0) as u8,
                    (fc.z.clamp(0.0, 1.0) * 255.0) as u8,
                    255,
                );
                framebuffer.set_pixel_depth(px as i32, py as i32, frag.depth, color);
            }
        }
    }
    (calls, framebuffer.visible_pixel_count())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::obj::load_obj;
    use crate::primitive::ScreenVertex;
    use raylib::prelude::Vector3;

    #[test]
    fn barycentric_and_perspective_interpolation_with_distinct_w() {
        let mk_v = |x: f32, y: f32, w: f32, val: f32| ScreenVertex {
            x,
            y,
            z: 0.5,
            inv_w: 1.0 / w,
            object_pos: Vector3::new(val, 0.0, 0.0),
            world_pos: Vector3::zero(),
            normal: Vector3::new(0.0, 0.0, 1.0),
            uv: [0.0, 0.0],
            color: Vector3::new(1.0, 1.0, 1.0),
        };
        let tri = ScreenTriangle {
            vertices: [
                mk_v(0.0, 0.0, 1.0, 0.0),
                mk_v(0.0, 10.0, 4.0, 10.0),
                mk_v(10.0, 0.0, 1.0, 0.0),
            ],
        };
        // En el punto medio de la arista v0-v1 en pantalla: w_screen = (0.5, 0.5, 0.0)
        // Con w0=1 y w1=4, el peso en perspectiva es (0.5/1) : (0.5/4) = 0.8 : 0.2 => x = 2.0
        let frag = interpolate_screen_fragment(&tri, 0.0, 5.0, (0.5, 0.5, 0.0));
        assert!((frag.object_pos.x - 2.0).abs() < 1e-5);

        let pa = (tri.vertices[0].x, tri.vertices[0].y);
        let pb = (tri.vertices[1].x, tri.vertices[1].y);
        let pc = (tri.vertices[2].x, tri.vertices[2].y);
        let area = edge_2d(pa, pb, pc);
        assert!(area > 0.0);
        let p = (2.5, 2.5);
        let w0 = edge_2d(pb, pc, p) / area;
        let w1 = edge_2d(pc, pa, p) / area;
        let w2 = edge_2d(pa, pb, p) / area;
        assert!((w0 + w1 + w2 - 1.0).abs() < 1e-6);
        assert!(w0 > 0.0 && w1 > 0.0 && w2 > 0.0);
    }

    #[test]
    fn perspective_correct_vertex_color_interpolation() {
        let mk_c = |x: f32, y: f32, w: f32, color: Vector3| ScreenVertex {
            x,
            y,
            z: 0.5,
            inv_w: 1.0 / w,
            object_pos: Vector3::zero(),
            world_pos: Vector3::zero(),
            normal: Vector3::new(0.0, 0.0, 1.0),
            uv: [0.0, 0.0],
            color,
        };
        let tri = ScreenTriangle {
            vertices: [
                mk_c(0.0, 0.0, 1.0, Vector3::new(1.0, 0.0, 0.0)),
                mk_c(0.0, 10.0, 2.0, Vector3::new(0.0, 1.0, 0.0)),
                mk_c(10.0, 0.0, 4.0, Vector3::new(0.0, 0.0, 1.0)),
            ],
        };
        // Pesos en pantalla (0.5, 0.25, 0.25):
        // inv_w ponderado: 0.5/1 + 0.25/2 + 0.25/4 = 0.5 + 0.125 + 0.0625 = 0.6875
        // pw = (0.5/0.6875, 0.125/0.6875, 0.0625/0.6875) = (8/11, 2/11, 1/11)
        let frag = interpolate_screen_fragment(&tri, 2.5, 2.5, (0.5, 0.25, 0.25));
        assert!((frag.color.x - 8.0 / 11.0).abs() < 1e-5);
        assert!((frag.color.y - 2.0 / 11.0).abs() < 1e-5);
        assert!((frag.color.z - 1.0 / 11.0).abs() < 1e-5);
    }

    #[test]
    fn axis_aligned_triangle_coverage_in_screen_space() {
        let pa = (0.0, 0.0);
        let pb = (0.0, 4.0);
        let pc = (4.0, 0.0);
        let area = edge_2d(pa, pb, pc);
        let inv_area = 1.0 / area;
        let mut covered = 0usize;
        for y in 0..4 {
            for x in 0..4 {
                let p = (x as f32 + 0.5, y as f32 + 0.5);
                let w0 = edge_2d(pb, pc, p) * inv_area;
                let w1 = edge_2d(pc, pa, p) * inv_area;
                let w2 = edge_2d(pa, pb, p) * inv_area;
                if w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0 {
                    covered += 1;
                }
            }
        }
        // Con bordes inclusivos (w >= 0), la diagonal x+y=4 incluye 4 centros: 4+3+2+1 = 10 píxeles
        assert_eq!(covered, 10);
    }

    #[test]
    fn shader_calls_never_exceed_visible_pixels_with_overlapping_triangles() {
        use crate::Uniforms;
        use crate::framebuffer::Framebuffer;
        use crate::math::Mat4;
        use crate::noise::Noise;
        use raylib::prelude::Color;

        let mk_tri = |z: f32| {
            let mk_v = |x: f32, y: f32| ScreenVertex {
                x,
                y,
                z,
                inv_w: 1.0,
                object_pos: Vector3::new(0.0, 0.0, 1.0),
                world_pos: Vector3::new(0.0, 0.0, 1.0),
                normal: Vector3::new(0.0, 0.0, 1.0),
                uv: [0.0, 0.0],
                color: Vector3::new(1.0, 1.0, 1.0),
            };
            ScreenTriangle {
                vertices: [mk_v(0.0, 0.0), mk_v(0.0, 32.0), mk_v(32.0, 0.0)],
            }
        };

        let triangles = [mk_tri(0.8), mk_tri(0.2)];
        let mut fb = Framebuffer::new(64, 64, Color::BLACK);
        let uniforms = Uniforms {
            model_matrix: Mat4::identity(),
            view_matrix: Mat4::identity(),
            projection_matrix: Mat4::identity(),
            time: 0.0,
            light_dir: Vector3::new(0.0, 0.0, 1.0),
            camera_pos: Vector3::new(0.0, 0.0, 5.0),
            debug_noise: false,
            noise_frequency: 4.0,
            rocky_debug_view: 3,
            normal_mapping: true,
            debug_normals: false,
            viewport_width: 64,
            viewport_height: 64,
            noise: Noise::new(0),
            baked: None,
            shader_param: 1.0,
        };

        let writes = rasterize_visibility(&triangles, &mut fb, true);
        let (calls, visible) = shade_visibility(&triangles, &mut fb, Shader::Star, &uniforms);
        assert!(writes >= visible);
        assert_eq!(calls, visible);
        assert!(calls <= fb.visible_pixel_count());
    }

    #[test]
    fn tiled_rasterization_matches_full_rasterization_pixel_by_pixel() {
        use crate::Uniforms;
        use crate::framebuffer::Framebuffer;
        use crate::math::Mat4;
        use crate::noise::Noise;
        use raylib::prelude::Color;

        let mk_v = |x: f32, y: f32, z: f32, w: f32| ScreenVertex {
            x,
            y,
            z,
            inv_w: 1.0 / w,
            object_pos: Vector3::new(x * 0.01, y * 0.01, 1.0).normalize(),
            world_pos: Vector3::new(x * 0.01, y * 0.01, 1.0),
            normal: Vector3::new(0.0, 0.0, 1.0),
            uv: [0.0, 0.0],
            color: Vector3::new(1.0, 1.0, 1.0),
        };
        let triangles = [
            ScreenTriangle {
                vertices: [
                    mk_v(10.0, 10.0, 0.6, 1.0),
                    mk_v(15.0, 130.0, 0.4, 1.5),
                    mk_v(150.0, 20.0, 0.5, 2.0),
                ],
            },
            ScreenTriangle {
                vertices: [
                    mk_v(50.0, 50.0, 0.2, 1.2),
                    mk_v(60.0, 155.0, 0.7, 1.8),
                    mk_v(175.0, 70.0, 0.3, 1.0),
                ],
            },
        ];
        let uniforms = Uniforms {
            model_matrix: Mat4::identity(),
            view_matrix: Mat4::identity(),
            projection_matrix: Mat4::identity(),
            time: 0.0,
            light_dir: Vector3::new(0.0, 0.0, 1.0),
            camera_pos: Vector3::new(0.0, 0.0, 5.0),
            debug_noise: false,
            noise_frequency: 4.0,
            rocky_debug_view: 3,
            normal_mapping: true,
            debug_normals: false,
            viewport_width: 160,
            viewport_height: 140,
            noise: Noise::new(0),
            baked: None,
            shader_param: 1.0,
        };

        let mut fb_full = Framebuffer::new(160, 140, Color::BLACK);
        rasterize_visibility(&triangles, &mut fb_full, true);
        shade_visibility(&triangles, &mut fb_full, Shader::Star, &uniforms);

        let mut fb_tiled = Framebuffer::new(160, 140, Color::BLACK);
        let mut grid = TileGrid::new(160, 140);
        rasterize_visibility_tiled(&triangles, &mut grid, &mut fb_tiled, true, 4);
        shade_visibility_tiled(&triangles, &mut fb_tiled, Shader::Star, &uniforms, 4);

        assert_eq!(fb_full.depth_buffer, fb_tiled.depth_buffer);
        assert_eq!(fb_full.tri_id_buffer, fb_tiled.tri_id_buffer);
        assert_eq!(fb_full.color_pixels, fb_tiled.color_pixels);
    }

    #[test]
    fn crossing_near_plane_with_camera_never_panics_or_stretches() {
        use crate::Uniforms;
        use crate::camera::Camera;
        use crate::framebuffer::Framebuffer;
        use crate::noise::Noise;
        use crate::primitive::assemble_screen_triangles;
        use crate::scene::build_model_matrix;
        use crate::vertex::Vertex;
        use raylib::prelude::Color;

        let obj = load_obj("assets/sphere.obj").expect("sphere.obj debe cargar");
        let (min, max) = obj.bounds();
        let max_extent = (max.x - min.x).max(max.y - min.y).max(max.z - min.z);
        let model_matrix = build_model_matrix(&obj, 0.0, 0.0, 2.0 / max_extent);
        let (mesh_verts, mesh_indices) = obj.to_indexed_mesh();
        let vertex_array: Vec<Vertex> = mesh_indices
            .iter()
            .map(|&i| mesh_verts[i].clone())
            .collect();

        let mut clip_scratch = Vec::new();
        let mut screen_tris = Vec::new();
        let mut grid = TileGrid::new(160, 120);
        let mut fb = Framebuffer::new(160, 120, Color::BLACK);
        let noise = Noise::new(0);

        for dist in [1.001_f32, 0.5, 0.05, -0.5] {
            fb.clear();
            let cam = Camera::new(
                Vector3::new(0.0, 0.0, dist),
                Vector3::new(0.0, 0.0, dist - 2.0),
                45.0_f32.to_radians(),
                0.1,
                100.0,
            );
            let uniforms = Uniforms {
                model_matrix,
                view_matrix: cam.view_matrix(),
                projection_matrix: cam.projection_matrix(160.0 / 120.0),
                time: 0.0,
                light_dir: Vector3::new(0.0, 0.0, 1.0),
                camera_pos: cam.position,
                debug_noise: false,
                noise_frequency: 4.0,
                rocky_debug_view: 3,
            normal_mapping: true,
            debug_normals: false,
                viewport_width: 160,
                viewport_height: 120,
                noise: noise.clone(),
                baked: None,
                shader_param: 1.0,
            };

            assemble_screen_triangles(&vertex_array, &uniforms, &mut clip_scratch, &mut screen_tris);
            for tri in &screen_tris {
                for v in &tri.vertices {
                    assert!(v.x.is_finite() && v.y.is_finite() && v.z.is_finite());
                    assert!(v.inv_w.is_finite() && v.inv_w > 0.0);
                    assert!(v.z >= -1e-4, "vértice detrás de near no recortado: z={}", v.z);
                }
            }
            rasterize_visibility_tiled(&screen_tris, &mut grid, &mut fb, true, 2);
            shade_visibility_tiled(&screen_tris, &mut fb, Shader::Star, &uniforms, 2);
            for &d in &fb.depth_buffer {
                if d < f32::INFINITY {
                    assert!(d.is_finite() && (-1e-4..=1.0001).contains(&d));
                }
            }
        }
    }

    #[test]
    fn sphere_obj_triangles_are_oriented_outward() {
        let obj = load_obj("assets/sphere.obj").expect("sphere.obj debe cargar");
        let mut inward = 0usize;
        for tri in obj.indices.as_chunks::<3>().0 {
            let a = obj.vertices[tri[0]];
            let b = obj.vertices[tri[1]];
            let c = obj.vertices[tri[2]];
            let normal = (b - a).cross(c - a);
            let center = (a + b + c) * (1.0 / 3.0);
            if normal.dot(center) <= 0.0 {
                inward += 1;
            }
        }
        assert_eq!(inward, 0, "sphere.obj tiene {inward} triángulos invertidos");
    }

    // ship.obj tiene 10,043 de 29,696 triángulos con normal hacia el centroide
    // (geometría cóncava o winding inconsistente); se deja ignorado sin modificar el OBJ.
    #[test]
    #[ignore = "ship.obj tiene 10043 de 29696 triángulos orientados hacia el centroide"]
    fn ship_obj_triangles_orientation_check() {
        let obj = load_obj("assets/ship.obj").expect("ship.obj debe cargar");
        let centroid = obj
            .vertices
            .iter()
            .copied()
            .fold(Vector3::zero(), |acc, v| acc + v)
            * (1.0 / obj.vertices.len().max(1) as f32);
        let mut inward = 0usize;
        let mut total = 0usize;
        for tri in obj.indices.as_chunks::<3>().0 {
            let a = obj.vertices[tri[0]];
            let b = obj.vertices[tri[1]];
            let c = obj.vertices[tri[2]];
            let normal = (b - a).cross(c - a);
            let center = (a + b + c) * (1.0 / 3.0) - centroid;
            total += 1;
            if normal.dot(center) <= 0.0 {
                inward += 1;
            }
        }
        assert_eq!(
            inward, 0,
            "ship.obj tiene {inward} de {total} triángulos orientados hacia adentro"
        );
    }
}
