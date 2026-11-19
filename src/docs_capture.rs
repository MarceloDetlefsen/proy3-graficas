//! Modo `--docs`: genera las capturas oficiales en `docs/captures/`
//! con cámaras, semillas y tiempo fijos de forma reproducible y sin ventana.

use std::fs;
use std::path::Path;
use std::sync::Arc;

use raylib::prelude::*;

use crate::camera::Camera;
use crate::culling::Frustum;
use crate::framebuffer::Framebuffer;
use crate::hud::{self, HudSnapshot, OverlayBuffer};
use crate::icosphere::DeformedIcosphereCache;
use crate::lod::{ICOSPHERE_THRESHOLDS_PX, projected_radius_px, select_detail_level};
use crate::math::Mat4;
use crate::mesh_lod::load_ship_lods;
use crate::noise::Noise;
use crate::primitive::append_screen_triangles;
use crate::rasterizer::{
    TileGrid, TriMaterial, TriangleMode, WINDOW_HEIGHT, WINDOW_WIDTH,
    rasterize_visibility_tiled, render_wireframe, shade_visibility_tiled_materials,
};
use crate::{RenderBuffers, Uniforms};
use crate::scene::{Body, body_position_at, body_rt_matrix_at, solar_system};
use crate::shaders::Shader;
use crate::surface_map::BakedMaps;
use crate::surface_props::load_surface_props;
use crate::surface_scatter::{build_body_instances, scatter_for_body};

fn compose_and_save(fb: &Framebuffer, overlay: Option<&OverlayBuffer>, path: &Path) {
    let mut out = Framebuffer::new(WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32, Color::BLACK);
    let sx = fb.width as f32 / out.width as f32;
    let sy = fb.height as f32 / out.height as f32;
    for y in 0..out.height {
        let src_y = ((y as f32 * sy) as u32).min(fb.height - 1) as i32;
        for x in 0..out.width {
            let src_x = ((x as f32 * sx) as u32).min(fb.width - 1) as i32;
            let mut c = fb.pixel_color(src_x, src_y);
            if let Some(ov) = overlay {
                let idx = (y as usize * ov.width as usize + x as usize) * 4;
                let a = ov.pixels[idx + 3] as u16;
                if a > 0 {
                    let inv = 255 - a;
                    c.r = ((ov.pixels[idx] as u16 * a + c.r as u16 * inv) / 255) as u8;
                    c.g = ((ov.pixels[idx + 1] as u16 * a + c.g as u16 * inv) / 255) as u8;
                    c.b = ((ov.pixels[idx + 2] as u16 * a + c.b as u16 * inv) / 255) as u8;
                }
            }
            out.set_pixel(x as i32, y as i32, c);
        }
    }
    out.render_to_file(path.to_str().expect("Ruta de captura inválida"));
}

#[allow(clippy::too_many_arguments)]
fn render_scene_frame(
    bodies: &[Body],
    caches: &mut [DeformedIcosphereCache],
    noise: &Noise,
    baked: &Arc<BakedMaps>,
    cam: &Camera,
    time: f32,
    threads: usize,
    mode: TriangleMode,
    frustum_culling: bool,
    forced_lod: Option<usize>,
    normal_mapping: bool,
    surface_seed: u64,
    with_props: bool,
    fb: &mut Framebuffer,
    bufs: &mut RenderBuffers,
) -> (usize, usize, usize) {
    let aspect = fb.width as f32 / fb.height as f32;
    let mut uniforms = Uniforms {
        model_matrix: Mat4::identity(),
        view_matrix: cam.view_matrix(),
        projection_matrix: cam.projection_matrix(aspect),
        time,
        light_dir: Vector3::new(-0.65, 0.80, 1.0).normalize(),
        camera_pos: cam.position,
        debug_noise: false,
        noise_frequency: 4.0,
        rocky_debug_view: 3,
        normal_mapping,
        debug_normals: false,
        viewport_width: fb.width as i32,
        viewport_height: fb.height as i32,
        noise: noise.clone(),
        baked: Some(Arc::clone(baked)),
        shader_param: 1.0,
    };
    fb.clear();
    bufs.screen_triangles.clear();
    bufs.tri_materials.clear();

    let vp = uniforms.projection_matrix * uniforms.view_matrix;
    let frustum = Frustum::from_vp(&vp);
    let sun_pos = body_position_at(&bodies[0], time);
    let prop_meshes = load_surface_props();

    let mut tris_before = 0usize;
    let mut tris_after = 0usize;
    let mut top_lod = 0usize;

    for (i, body) in bodies.iter().enumerate() {
        let pos = body_position_at(body, time);
        let eff_r = body.radius + body.surface.max_deformation;
        let dist = (pos - cam.position).length();
        let proj_px = projected_radius_px(cam, pos, eff_r, fb.height as f32);
        let lvl = forced_lod.unwrap_or_else(|| select_detail_level(proj_px, &ICOSPHERE_THRESHOLDS_PX).min(4));
        top_lod = top_lod.max(lvl);
        let mesh = caches[i].get_or_build(lvl, noise);
        let tris = mesh.len() / 3;
        tris_before += tris;

        if frustum_culling && !frustum.intersects_sphere(pos, eff_r) {
            continue;
        }
        tris_after += tris;
        uniforms.model_matrix =
            body_rt_matrix_at(body, time) * Mat4::scale(Vector3::new(body.radius, body.radius, body.radius));
        let body_light = if matches!(body.shader, Shader::Star) {
            Vector3::new(0.0, 1.0, 0.0)
        } else {
            (sun_pos - pos).normalize()
        };
        let b = bufs.screen_triangles.len();
        append_screen_triangles(mesh, &uniforms, &mut bufs.clip_vertices, &mut bufs.screen_triangles);
        let added = bufs.screen_triangles.len() - b;
        bufs.tri_materials.resize(
            b + added,
            TriMaterial {
                shader: body.shader,
                light_dir: body_light,
                shader_param: 1.0,
            },
        );

        if with_props && !matches!(body.shader, Shader::Star | Shader::GasGiant) && dist < 24.0 {
            let candidates = scatter_for_body(body, surface_seed);
            let instances = build_body_instances(body, &candidates, noise);
            let rt = body_rt_matrix_at(body, time);
            for inst in &instances {
                let wm = rt * inst.local_matrix;
                let prop_mesh = prop_meshes.mesh_for(inst.kind, 0);
                let p_tris = prop_mesh.len() / 3;
                tris_before += p_tris;
                let iw = Vector3::new(wm.cols[3][0], wm.cols[3][1], wm.cols[3][2]);
                if frustum_culling && !frustum.intersects_sphere(iw, inst.bounding_radius) {
                    continue;
                }
                tris_after += p_tris;
                uniforms.model_matrix = wm;
                let bi = bufs.screen_triangles.len();
                append_screen_triangles(prop_mesh, &uniforms, &mut bufs.clip_vertices, &mut bufs.screen_triangles);
                let ai = bufs.screen_triangles.len() - bi;
                bufs.tri_materials.resize(
                    bi + ai,
                    TriMaterial {
                        shader: Shader::Prop(inst.kind),
                        light_dir: body_light,
                        shader_param: 1.0,
                    },
                );
            }
        }
    }

    match mode {
        TriangleMode::FilledBarycentric => {
            let mut tile_grid = TileGrid::new(fb.width as usize, fb.height as usize);
            rasterize_visibility_tiled(&bufs.screen_triangles, &mut tile_grid, fb, true, threads);
            shade_visibility_tiled_materials(
                &bufs.screen_triangles,
                &bufs.tri_materials,
                fb,
                Shader::Rocky,
                &uniforms,
                threads,
            );
        }
        TriangleMode::Wireframe => {
            render_wireframe(&bufs.screen_triangles, fb, Shader::Rocky, &uniforms);
        }
    }

    (tris_before, tris_after, top_lod)
}

pub fn generate_docs_captures() {
    let out_dir = Path::new("docs/captures");
    fs::create_dir_all(out_dir).expect("No se pudo crear docs/captures");

    let noise = Noise::new(0);
    let baked = Arc::new(BakedMaps::bake(&noise, 512, 256));
    let mut bodies = solar_system();
    let mut caches: Vec<DeformedIcosphereCache> = bodies
        .iter()
        .map(|b| DeformedIcosphereCache::new_for_shader(b.shader, b.surface.deformation_amplitude, b.surface.noise_frequency))
        .collect();
    let threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4);
    let mut fb = Framebuffer::new(400, 300, Color::new(15, 18, 28, 255));
    let mut bufs = RenderBuffers::new(100_000);
    let mut ov = OverlayBuffer::new(WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32);
    let time = 1.5_f32;

    // 1. Sistema completo.
    let cam_sys = Camera::new(Vector3::new(0.0, 26.0, 68.0), Vector3::zero(), 45.0_f32.to_radians(), 0.1, 400.0);
    render_scene_frame(&bodies, &mut caches, &noise, &baked, &cam_sys, time, threads, TriangleMode::FilledBarycentric, true, None, true, 42, false, &mut fb, &mut bufs);
    compose_and_save(&fb, None, &out_dir.join("sistema_completo.png"));

    // 2. Nave de cerca.
    let ship_lods = load_ship_lods();
    let (sv, si) = &ship_lods.levels[0];
    let ship_verts: Vec<_> = si.iter().map(|&idx| sv[idx].clone()).collect();
    let cam_ship = Camera::new(Vector3::new(0.95, 0.45, 1.35), Vector3::zero(), 45.0_f32.to_radians(), 0.1, 100.0);
    let ship_u = Uniforms {
        model_matrix: Mat4::scale(Vector3::new(0.025, 0.025, 0.025)),
        view_matrix: cam_ship.view_matrix(),
        projection_matrix: cam_ship.projection_matrix(400.0 / 300.0),
        time,
        light_dir: Vector3::new(-0.65, 0.80, 1.0).normalize(),
        camera_pos: cam_ship.position,
        debug_noise: false,
        noise_frequency: 4.0,
        rocky_debug_view: 3,
        normal_mapping: true,
        debug_normals: false,
        viewport_width: 400,
        viewport_height: 300,
        noise: noise.clone(),
        baked: Some(Arc::clone(&baked)),
        shader_param: 1.0,
    };
    fb.clear();
    bufs.screen_triangles.clear();
    append_screen_triangles(&ship_verts, &ship_u, &mut bufs.clip_vertices, &mut bufs.screen_triangles);
    let mut ship_grid = TileGrid::new(fb.width as usize, fb.height as usize);
    rasterize_visibility_tiled(&bufs.screen_triangles, &mut ship_grid, &mut fb, true, threads);
    let ship_mats = vec![TriMaterial { shader: Shader::Ship, light_dir: ship_u.light_dir, shader_param: 1.0 }; bufs.screen_triangles.len()];
    shade_visibility_tiled_materials(&bufs.screen_triangles, &ship_mats, &mut fb, Shader::Ship, &ship_u, threads);
    compose_and_save(&fb, None, &out_dir.join("nave.png"));

    // 3. Cada cuerpo de cerca.
    let slugs = ["sol", "zebes", "tau_ceti", "ardens", "abismo", "basquia", "vestigio", "mural", "fundido"];
    for (idx, body) in bodies.iter().enumerate() {
        let pos = body_position_at(body, time);
        let to_sun = if idx == 0 { Vector3::new(0.0, 0.0, 1.0) } else { (body_position_at(&bodies[0], time) - pos).normalize() };
        let side = to_sun.cross(Vector3::new(0.0, 1.0, 0.0)).normalize();
        let cam_dir = (to_sun * 0.78 + side * 0.55 + Vector3::new(0.0, 0.25, 0.0)).normalize();
        let cam_b = Camera::new(pos + cam_dir * (body.radius * 2.8), pos, 45.0_f32.to_radians(), 0.1, 400.0);
        render_scene_frame(&bodies, &mut caches, &noise, &baked, &cam_b, time, threads, TriangleMode::FilledBarycentric, true, None, true, 42, false, &mut fb, &mut bufs);
        compose_and_save(&fb, None, &out_dir.join(format!("{}.png", slugs[idx])));
    }

    // 4. LOD wireframe cerca y lejos sobre Zebes (idx 1).
    let zebes_pos = body_position_at(&bodies[1], time);
    let cam_lod_near = Camera::new(zebes_pos + Vector3::new(0.0, 0.5, 3.2), zebes_pos, 45.0_f32.to_radians(), 0.1, 400.0);
    render_scene_frame(&bodies, &mut caches, &noise, &baked, &cam_lod_near, time, threads, TriangleMode::Wireframe, true, Some(4), true, 42, false, &mut fb, &mut bufs);
    compose_and_save(&fb, None, &out_dir.join("lod_cerca.png"));

    let cam_lod_far = Camera::new(zebes_pos + Vector3::new(0.0, 2.0, 26.0), zebes_pos, 45.0_f32.to_radians(), 0.1, 400.0);
    render_scene_frame(&bodies, &mut caches, &noise, &baked, &cam_lod_far, time, threads, TriangleMode::Wireframe, true, Some(1), true, 42, false, &mut fb, &mut bufs);
    compose_and_save(&fb, None, &out_dir.join("lod_lejos.png"));

    // 5. Culling ON / OFF con contadores HUD visibles.
    let cam_cull = Camera::new(Vector3::new(0.0, 12.0, 34.0), Vector3::new(15.0, 0.0, 0.0), 45.0_f32.to_radians(), 0.1, 400.0);
    for (cull, fname) in [(true, "culling_on.png"), (false, "culling_off.png")] {
        let (tb, ta, lod) = render_scene_frame(&bodies, &mut caches, &noise, &baked, &cam_cull, time, threads, TriangleMode::FilledBarycentric, cull, None, true, 42, false, &mut fb, &mut bufs);
        let lod_str = format!("Max L{}", lod);
        let snap = HudSnapshot {
            title: "Sistema Solar | Frustum Culling",
            fps: 60.0, total_ms: 8.5, vertex_ms: 1.2, raster_ms: 1.1, shading_ms: 5.8, present_ms: 0.4,
            internal_width: 400, internal_height: 300, threads,
            tris_before_cull: tb, tris_after_cull: ta, frustum_culling: cull,
            lod_summary: &lod_str, ship_lod_summary: "Nave:L0", mode_summary: "Global [Auto]",
            gravity_enabled: true, gravity_mag: 0.45, speed: 4.2, altitude_summary: "12.4 u (Zebes)",
            time_scale: 1.0, paused: false, surface_seed: 42,
            instances_drawn: 0, instances_total: 0, instance_culling: true,
            selected_summary: "Sel(Tab): Zebes | R:1.25", collision_body: None,
            show_perf: true, show_help: false,
        };
        hud::render_hud(&mut ov, &snap);
        compose_and_save(&fb, Some(&ov), &out_dir.join(fname));
    }

    // 6. Normales ON / OFF en Zebes (lado iluminado por el Sol).
    let to_sun_zebes = (body_position_at(&bodies[0], time) - zebes_pos).normalize();
    let side_zebes = to_sun_zebes.cross(Vector3::new(0.0, 1.0, 0.0)).normalize();
    let cam_zebes = Camera::new(
        zebes_pos + (to_sun_zebes * 0.80 + side_zebes * 0.48 + Vector3::new(0.0, 0.22, 0.0)).normalize() * 3.1,
        zebes_pos,
        45.0_f32.to_radians(),
        0.1,
        400.0,
    );
    render_scene_frame(&bodies, &mut caches, &noise, &baked, &cam_zebes, time, threads, TriangleMode::FilledBarycentric, true, Some(4), true, 42, false, &mut fb, &mut bufs);
    compose_and_save(&fb, None, &out_dir.join("normales_on.png"));
    render_scene_frame(&bodies, &mut caches, &noise, &baked, &cam_zebes, time, threads, TriangleMode::FilledBarycentric, true, Some(4), false, 42, false, &mut fb, &mut bufs);
    compose_and_save(&fb, None, &out_dir.join("normales_off.png"));

    // 7. Deformación con dos amplitudes (0.04 y 0.24) en Zebes.
    let orig_amp = bodies[1].surface.deformation_amplitude;
    for (amp, fname) in [(0.04_f32, "deform_baja.png"), (0.24_f32, "deform_alta.png")] {
        bodies[1].surface.deformation_amplitude = amp;
        bodies[1].surface.max_deformation = bodies[1].radius * amp;
        caches[1].invalidate(amp, bodies[1].surface.noise_frequency);
        render_scene_frame(&bodies, &mut caches, &noise, &baked, &cam_zebes, time, threads, TriangleMode::FilledBarycentric, true, Some(4), true, 42, false, &mut fb, &mut bufs);
        compose_and_save(&fb, None, &out_dir.join(fname));
    }
    bodies[1].surface.deformation_amplitude = orig_amp;
    bodies[1].surface.max_deformation = bodies[1].radius * orig_amp;
    caches[1].invalidate(orig_amp, bodies[1].surface.noise_frequency);

    // 8. Dos semillas de distribución (42 y 99) en Zebes.
    for (seed, fname) in [(42_u64, "semilla_42.png"), (99_u64, "semilla_99.png")] {
        render_scene_frame(&bodies, &mut caches, &noise, &baked, &cam_zebes, time, threads, TriangleMode::FilledBarycentric, true, Some(4), true, seed, true, &mut fb, &mut bufs);
        compose_and_save(&fb, None, &out_dir.join(fname));
    }

    // 9. Panel de ayuda (H) y Modo Local.
    let (tb, ta, _) = render_scene_frame(&bodies, &mut caches, &noise, &baked, &cam_zebes, time, threads, TriangleMode::FilledBarycentric, true, Some(4), true, 42, true, &mut fb, &mut bufs);
    let help_snap = HudSnapshot {
        title: "Sistema Solar | Ayuda (H)",
        fps: 68.4, total_ms: 14.6, vertex_ms: 2.1, raster_ms: 1.4, shading_ms: 10.7, present_ms: 0.4,
        internal_width: 400, internal_height: 300, threads,
        tris_before_cull: tb, tris_after_cull: ta, frustum_culling: true,
        lod_summary: "Zeb:L4", ship_lod_summary: "Nave:L0", mode_summary: "Local(Zebes) [Auto]",
        gravity_enabled: true, gravity_mag: 3.12, speed: 2.4, altitude_summary: "1.55 u (Zebes)",
        time_scale: 1.0, paused: false, surface_seed: 42,
        instances_drawn: 180, instances_total: 253, instance_culling: true,
        selected_summary: "Sel(Tab): Zebes | R(9/0):1.25 | Contraste relieve(4/5):1.00",
        collision_body: None, show_perf: true, show_help: true,
    };
    hud::render_hud(&mut ov, &help_snap);
    compose_and_save(&fb, Some(&ov), &out_dir.join("ayuda_hud.png"));

    let local_snap = HudSnapshot { show_help: false, title: "Sistema Solar | Modo Local(Zebes)", ..help_snap };
    hud::render_hud(&mut ov, &local_snap);
    compose_and_save(&fb, Some(&ov), &out_dir.join("modo_local.png"));

    // 10. Menú principal con fondo orbital animado.
    let cam_menu = Camera::new(
        Vector3::new(17.0, 15.5, 40.5),
        Vector3::zero(),
        45.0_f32.to_radians(),
        0.1,
        400.0,
    );
    render_scene_frame(
        &bodies,
        &mut caches,
        &noise,
        &baked,
        &cam_menu,
        time,
        threads,
        TriangleMode::FilledBarycentric,
        true,
        None,
        true,
        42,
        false,
        &mut fb,
        &mut bufs,
    );
    let fav_ready = crate::showcase::is_favorites_system_ready(crate::showcase::SHOWCASE_ENTRIES);
    hud::render_menu_overlay(&mut ov, 0, fav_ready, false);
    compose_and_save(&fb, Some(&ov), &out_dir.join("menu_principal.png"));

    // 11. Galería de trofeos (Zebes y Ardens).
    let mut gal_viewer = crate::gallery::GalleryViewer::new(bodies[1].radius);
    for &(idx, fname) in &[(1_usize, "galeria_zebes.png"), (3_usize, "galeria_ardens.png")] {
        let body = &bodies[idx];
        gal_viewer.reset_view(body.radius);
        gal_viewer.lod_level = 4;
        let tris = gal_viewer.render_body(
            idx,
            body,
            1.0,
            time,
            &noise,
            &baked,
            &mut fb,
            &mut bufs,
            TriangleMode::FilledBarycentric,
            threads,
        );
        let spec = crate::shaders::live_param(body.shader);
        let gsnap = hud::GalleryHudSnapshot {
            entry: &crate::showcase::SHOWCASE_ENTRIES[idx],
            active_showcase_idx: idx,
            radius: body.radius,
            tris,
            lod_level: gal_viewer.lod_level,
            shader_name: crate::shaders::shader_name(body.shader),
            live_param_name: spec.name,
            live_param_value: spec.default,
            fps: 60.0,
        };
        hud::render_gallery_overlay(&mut ov, &gsnap);
        compose_and_save(&fb, Some(&ov), &out_dir.join(fname));
    }

    println!("Capturas oficiales generadas en docs/captures/ (26 archivos)");
}
