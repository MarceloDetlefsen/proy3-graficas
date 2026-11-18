mod camera;
mod clipping;
mod fragment;
mod framebuffer;
mod math;
mod mesh_lod;
mod nav;
mod noise;
mod obj;
mod perf;
mod physics;
mod primitive;
mod rasterizer;
mod scene;
mod shaders;
mod ship;
mod sim;
mod surface_map;
mod vertex;

use camera::{Camera, FollowCamera};
use clipping::ClipVertex;
use framebuffer::Framebuffer;
use math::Mat4;
use mesh_lod::{load_ship_lods, mesh_bounds};
use noise::Noise;
use obj::load_obj;
use perf::{FrameSample, PerfStats};
use primitive::{ScreenTriangle, append_screen_triangles, assemble_screen_triangles};
use rasterizer::{
    TILE_SIZE, TileGrid, TriMaterial, TriangleMode, WINDOW_HEIGHT, WINDOW_WIDTH,
    rasterize_visibility_tiled, render_wireframe, shade_visibility_tiled,
    shade_visibility_tiled_materials,
};
use raylib::prelude::*;
use scene::{body_model_matrix_at, body_position_at, build_model_matrix, solar_system};
use shaders::{Shader, shader_name};
use ship::Ship;
use sim::SimClock;
use surface_map::{BakedMaps, DEFAULT_BAKE_HEIGHT, DEFAULT_BAKE_WIDTH};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Clone)]
pub struct Uniforms {
    pub model_matrix: Mat4,
    pub view_matrix: Mat4,
    pub projection_matrix: Mat4,
    pub time: f32,
    pub light_dir: Vector3,
    pub camera_pos: Vector3,
    pub debug_noise: bool,
    pub noise_frequency: f32,
    pub rocky_debug_view: i32,
    pub viewport_width: i32,
    pub viewport_height: i32,
    pub noise: Noise,
    pub baked: Option<Arc<BakedMaps>>,
}

struct CaptureOptions {
    shader: Shader,
    view: i32,
    light_dir: Vector3,
    time: f32,
    output: PathBuf,
}

fn parse_vector3(value: &str, argument: &str) -> Vector3 {
    let components: Vec<f32> = value
        .split(',')
        .map(|component| {
            component
                .parse::<f32>()
                .unwrap_or_else(|_| panic!("Valor inválido para {argument}: {value}"))
        })
        .collect();
    if components.len() != 3 {
        panic!("{argument} debe tener formato x,y,z: {value}");
    }
    Vector3::new(components[0], components[1], components[2]).normalize()
}

fn parse_shader(value: &str) -> Shader {
    match value {
        "1" => Shader::Star,
        "2" => Shader::Rocky,
        "3" => Shader::GasGiant,
        _ => panic!("Shader inválido: {value}; usá 1, 2 o 3"),
    }
}

fn parse_view(value: &str) -> i32 {
    match value {
        "height" | "altura" => 0,
        "biome" | "bioma" => 1,
        "cracks" | "grietas" => 2,
        "color" => 3,
        _ => panic!("Vista inválida: {value}; usá height, biome, cracks o color"),
    }
}

fn parse_capture_options() -> Option<CaptureOptions> {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    if arguments.is_empty() {
        return None;
    }

    let mut shader = None;
    let mut view = None;
    let mut light_dir = None;
    let mut time = None;
    let mut output = None;
    let mut index = 0;
    while index < arguments.len() {
        let argument = &arguments[index];
        let value = arguments.get(index + 1).unwrap_or_else(|| {
            panic!("Falta valor para {argument}; argumentos esperados: --shader, --view, --light, --time, --out")
        });
        match argument.as_str() {
            "--shader" => shader = Some(parse_shader(value)),
            "--view" => view = Some(parse_view(value)),
            "--light" => light_dir = Some(parse_vector3(value, "--light")),
            "--time" => {
                time = Some(
                    value
                        .parse::<f32>()
                        .unwrap_or_else(|_| panic!("Tiempo inválido: {value}")),
                )
            }
            "--out" => output = Some(PathBuf::from(value)),
            _ => panic!("Argumento desconocido: {argument}"),
        }
        index += 2;
    }

    Some(CaptureOptions {
        shader: shader.expect("Falta --shader"),
        view: view.expect("Falta --view"),
        light_dir: light_dir.expect("Falta --light"),
        time: time.expect("Falta --time"),
        output: output.expect("Falta --out"),
    })
}

fn next_capture_path() -> PathBuf {
    let captures_dir = PathBuf::from("captures");
    fs::create_dir_all(&captures_dir).expect("No se pudo crear el directorio captures");

    let mut index = 1;
    loop {
        let path = captures_dir.join(format!("shot_{index:03}.png"));
        if !path.exists() {
            return path;
        }
        index += 1;
    }
}

struct RenderBuffers {
    clip_vertices: Vec<ClipVertex>,
    screen_triangles: Vec<ScreenTriangle>,
    tri_materials: Vec<TriMaterial>,
    tile_grid: TileGrid,
}

impl RenderBuffers {
    fn new(vertex_capacity: usize) -> Self {
        Self {
            clip_vertices: Vec::with_capacity(vertex_capacity),
            screen_triangles: Vec::with_capacity(vertex_capacity / 3),
            tri_materials: Vec::with_capacity(vertex_capacity / 3),
            tile_grid: TileGrid::new(WINDOW_WIDTH as usize, WINDOW_HEIGHT as usize),
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn render(
    framebuffer: &mut Framebuffer,
    uniforms: &Uniforms,
    vertex_array: &[vertex::Vertex],
    buffers: &mut RenderBuffers,
    mode: TriangleMode,
    shader: Shader,
    backface_culling: bool,
    threads: usize,
) -> FrameSample {
    let t_vtx = std::time::Instant::now();
    assemble_screen_triangles(
        vertex_array,
        uniforms,
        &mut buffers.clip_vertices,
        &mut buffers.screen_triangles,
    );
    let vertex_ms = t_vtx.elapsed().as_secs_f64() * 1000.0;

    let (raster_ms, shading_ms, fragments, shader_calls, visible_pixels) = match mode {
        TriangleMode::FilledBarycentric => {
            let t_rast = std::time::Instant::now();
            let writes = rasterize_visibility_tiled(
                &buffers.screen_triangles,
                &mut buffers.tile_grid,
                framebuffer,
                backface_culling,
                threads,
            );
            let _ = TILE_SIZE;
            let raster_ms = t_rast.elapsed().as_secs_f64() * 1000.0;

            let t_shade = std::time::Instant::now();
            let (calls, visible) = shade_visibility_tiled(
                &buffers.screen_triangles,
                framebuffer,
                shader,
                uniforms,
                threads,
            );
            let shading_ms = t_shade.elapsed().as_secs_f64() * 1000.0;
            (raster_ms, shading_ms, writes, calls, visible)
        }
        TriangleMode::Wireframe => {
            let t_wire = std::time::Instant::now();
            let (calls, visible) =
                render_wireframe(&buffers.screen_triangles, framebuffer, shader, uniforms);
            let wire_ms = t_wire.elapsed().as_secs_f64() * 1000.0;
            (0.0, wire_ms, calls, calls, visible)
        }
    };

    FrameSample {
        vertex_ms,
        raster_ms,
        shading_ms,
        fb_ms: 0.0,
        present_ms: 0.0,
        total_ms: 0.0,
        fragments,
        shader_calls,
        visible_pixels,
        width: framebuffer.width,
        height: framebuffer.height,
        threads,
    }
}

fn main() {
    let obj_path = "assets/sphere.obj";
    let obj = load_obj(obj_path).expect("No se pudo cargar assets/sphere.obj");
    let (mesh_vertices, mesh_indices) = obj.to_indexed_mesh();
    println!("Modelo cargado: '{obj_path}'");
    println!("  Posiciones fuente (v): {}", obj.vertices.len());
    println!("  UVs fuente (vt): {}", obj.texcoords.len());
    println!("  Normales fuente (vn): {}", obj.source_normals.len());
    println!("  Vértices indexados: {}", mesh_vertices.len());
    println!("  Triángulos: {}", mesh_indices.len() / 3);

    let vertex_array: Vec<vertex::Vertex> = mesh_indices
        .iter()
        .map(|&index| mesh_vertices[index].clone())
        .collect();

    let (min, max) = obj.bounds();
    let max_extent = (max.x - min.x)
        .max(max.y - min.y)
        .max(max.z - min.z)
        .max(1e-4);
    let base_scale = 2.0 / max_extent;
    let internal_width = WINDOW_WIDTH;
    let internal_height = WINDOW_HEIGHT;

    let mut framebuffer = Framebuffer::new(
        internal_width as u32,
        internal_height as u32,
        Color::new(15, 18, 28, 255),
    );

    let noise = Noise::new(0);
    let bake_start = std::time::Instant::now();
    let mut baked = Arc::new(BakedMaps::bake(
        &noise,
        DEFAULT_BAKE_WIDTH,
        DEFAULT_BAKE_HEIGHT,
    ));
    println!(
        "Mapas estáticos horneados ({}x{}) en {:.2} ms",
        DEFAULT_BAKE_WIDTH,
        DEFAULT_BAKE_HEIGHT,
        bake_start.elapsed().as_secs_f64() * 1000.0
    );

    if let Some(options) = parse_capture_options() {
        let aspect = internal_width as f32 / internal_height as f32;
        let model_matrix = build_model_matrix(&obj, 0.0, 0.0, base_scale);
        let cam = Camera::new(
            Vector3::new(0.0, 0.0, 3.0),
            Vector3::zero(),
            45.0_f32.to_radians(),
            0.1,
            100.0,
        );
        let uniforms = Uniforms {
            model_matrix,
            view_matrix: cam.view_matrix(),
            projection_matrix: cam.projection_matrix(aspect),
            time: options.time,
            light_dir: options.light_dir,
            camera_pos: cam.position,
            debug_noise: false,
            noise_frequency: 4.0,
            rocky_debug_view: options.view,
            viewport_width: internal_width,
            viewport_height: internal_height,
            noise: noise.clone(),
            baked: Some(Arc::clone(&baked)),
        };
        let mut render_buffers = RenderBuffers::new(vertex_array.len());
        framebuffer.clear();
        let default_threads = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1);
        render(
            &mut framebuffer,
            &uniforms,
            &vertex_array,
            &mut render_buffers,
            TriangleMode::FilledBarycentric,
            options.shader,
            true,
            default_threads,
        );
        if let Some(parent) = options.output.parent() {
            fs::create_dir_all(parent).expect("No se pudo crear el directorio de captura");
        }
        framebuffer.render_to_file(options.output.to_str().expect("Ruta de captura inválida"));
        println!(
            "Captura automática guardada en {}",
            options.output.display()
        );
        return;
    }

    let (mut window, thread) = raylib::init()
        .size(WINDOW_WIDTH, WINDOW_HEIGHT)
        .title("Proyecto 3 - Base del sistema solar")
        .build();
    window.set_target_fps(60);

    let mut mode = TriangleMode::FilledBarycentric;
    let mut cam_yaw = 0.0_f32;
    let mut cam_pitch = 0.45_f32;
    let mut cam_distance = 34.0_f32;
    let mut sim_clock = SimClock::new();
    let time_scales = [0.25_f32, 0.5, 1.0, 2.0, 5.0];
    let mut time_scale_idx = 2_usize;
    let ship_lods = load_ship_lods();
    let (ship_v0, ship_i0) = &ship_lods.levels[0];
    let (ship_min, ship_max) = mesh_bounds(ship_v0);
    let ship_center = (ship_min + ship_max) * 0.5;
    let ship_extent = (ship_max.x - ship_min.x)
        .max(ship_max.y - ship_min.y)
        .max(ship_max.z - ship_min.z)
        .max(1e-4);
    let ship_unit_scale = 2.0 / ship_extent;
    let ship_vertex_array: Vec<vertex::Vertex> =
        ship_i0.iter().map(|&i| ship_v0[i].clone()).collect();
    let mut ship = Ship::new(Vector3::new(0.0, 1.5, 16.0));
    let mut physics_stepper = physics::PhysicsStepper::new();
    let mut gravity_enabled = true;
    let mut gravity_mag = 0.0_f32;
    let mut show_help = false;
    let mut follow_cam = FollowCamera::new(&ship, 5.5, 1.6);
    let mut follow_mode = true;
    window.disable_cursor();
    let bodies = solar_system();
    let _ = build_model_matrix;
    let _ = shader_name;
    let mut shader = Shader::Rocky;
    let mut debug_noise = false;
    let mut noise_frequency = 4.0_f32;
    let mut rocky_debug_view = 3_i32;
    let mut light_angle = 0.0_f32;
    let mut backface_culling = true;
    let res_scales = [2_u32, 3, 1];
    let mut res_scale_idx = 0_usize; // 1/2 por defecto (400x300)
    let mut internal_width = WINDOW_WIDTH / res_scales[res_scale_idx] as i32;
    let mut internal_height = WINDOW_HEIGHT / res_scales[res_scale_idx] as i32;
    framebuffer.resize(internal_width as u32, internal_height as u32);
    let max_threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);
    let mut thread_options: Vec<usize> = [1, 2, 4, 8, 16]
        .into_iter()
        .filter(|&n| n < max_threads)
        .collect();
    thread_options.push(max_threads);
    let mut thread_idx = thread_options.len() - 1;
    let mut threads = thread_options[thread_idx];
    let orbit_speed = 1.8_f32;
    let light_speed = 1.8_f32;
    let mut perf_stats = PerfStats::new();
    let mut render_buffers = RenderBuffers::new(vertex_array.len());

    while !window.window_should_close() {
        let frame_start = std::time::Instant::now();
        let dt = window.get_frame_time();
        let _sim_dt = sim_clock.advance(dt);
        if window.is_key_pressed(KeyboardKey::KEY_F) {
            follow_mode = !follow_mode;
            if follow_mode {
                window.disable_cursor();
                follow_cam.snap(&ship);
            } else {
                window.enable_cursor();
            }
        }
        if window.is_key_pressed(KeyboardKey::KEY_BACKSPACE) {
            follow_cam.recenter();
        }
        if window.is_key_pressed(KeyboardKey::KEY_TAB) {
            sim_clock.toggle_pause();
        }
        if window.is_key_pressed(KeyboardKey::KEY_H) {
            show_help = !show_help;
        }
        if window.is_key_pressed(KeyboardKey::KEY_G) {
            gravity_enabled = !gravity_enabled;
        }
        if window.is_key_pressed(KeyboardKey::KEY_MINUS) {
            time_scale_idx = time_scale_idx.saturating_sub(1);
            sim_clock.set_scale(time_scales[time_scale_idx]);
        }
        if window.is_key_pressed(KeyboardKey::KEY_EQUAL) {
            time_scale_idx = (time_scale_idx + 1).min(time_scales.len() - 1);
            sim_clock.set_scale(time_scales[time_scale_idx]);
        }
        if window.is_key_pressed(KeyboardKey::KEY_F3) {
            perf_stats.enabled = !perf_stats.enabled;
        }
        if window.is_key_pressed(KeyboardKey::KEY_C) {
            backface_culling = !backface_culling;
        }
        if window.is_key_pressed(KeyboardKey::KEY_T) {
            thread_idx = (thread_idx + 1) % thread_options.len();
            threads = thread_options[thread_idx];
        }
        if window.is_key_pressed(KeyboardKey::KEY_K) {
            let t_bake = std::time::Instant::now();
            baked = Arc::new(BakedMaps::bake(
                &noise,
                DEFAULT_BAKE_WIDTH,
                DEFAULT_BAKE_HEIGHT,
            ));
            println!(
                "Mapas re-horneados ({}x{}) en {:.2} ms",
                DEFAULT_BAKE_WIDTH,
                DEFAULT_BAKE_HEIGHT,
                t_bake.elapsed().as_secs_f64() * 1000.0
            );
        }
        if window.is_key_pressed(KeyboardKey::KEY_R) {
            res_scale_idx = (res_scale_idx + 1) % res_scales.len();
            let div = res_scales[res_scale_idx] as i32;
            internal_width = WINDOW_WIDTH / div;
            internal_height = WINDOW_HEIGHT / div;
            framebuffer.resize(internal_width as u32, internal_height as u32);
        }
        if window.is_key_pressed(KeyboardKey::KEY_F1) {
            mode = TriangleMode::Wireframe;
        }
        if window.is_key_pressed(KeyboardKey::KEY_F2) {
            mode = TriangleMode::FilledBarycentric;
        }
        if window.is_key_pressed(KeyboardKey::KEY_ONE) {
            shader = Shader::Star;
        }
        if window.is_key_pressed(KeyboardKey::KEY_TWO) {
            shader = Shader::Rocky;
        }
        if window.is_key_pressed(KeyboardKey::KEY_THREE) {
            shader = Shader::GasGiant;
        }
        if window.is_key_pressed(KeyboardKey::KEY_N) {
            debug_noise = !debug_noise;
        }
        if window.is_key_pressed(KeyboardKey::KEY_V) {
            rocky_debug_view = (rocky_debug_view + 1) % 4;
        }
        if window.is_key_down(KeyboardKey::KEY_J) {
            light_angle -= light_speed * dt;
        }
        if window.is_key_down(KeyboardKey::KEY_L) {
            light_angle += light_speed * dt;
        }
        if window.is_key_pressed(KeyboardKey::KEY_LEFT_BRACKET) {
            noise_frequency = (noise_frequency - 0.25).max(0.25);
        }
        if window.is_key_pressed(KeyboardKey::KEY_RIGHT_BRACKET) {
            noise_frequency = (noise_frequency + 0.25).min(20.0);
        }
        let capture_requested = window.is_key_pressed(KeyboardKey::KEY_P);
        if !follow_mode {
            if window.is_key_down(KeyboardKey::KEY_LEFT) {
                cam_yaw -= orbit_speed * dt;
            }
            if window.is_key_down(KeyboardKey::KEY_RIGHT) {
                cam_yaw += orbit_speed * dt;
            }
            if window.is_key_down(KeyboardKey::KEY_UP) {
                cam_pitch = (cam_pitch + orbit_speed * dt).min(1.5);
            }
            if window.is_key_down(KeyboardKey::KEY_DOWN) {
                cam_pitch = (cam_pitch - orbit_speed * dt).max(-1.5);
            }
        }

        // Controles de vuelo con paso de tiempo fijo.
        let mut throttle = 0.0_f32;
        if window.is_key_down(KeyboardKey::KEY_W) {
            throttle += 1.0;
        }
        if window.is_key_down(KeyboardKey::KEY_S) {
            throttle -= 1.0;
        }
        let boost = window.is_key_down(KeyboardKey::KEY_LEFT_SHIFT);
        let braking = window.is_key_down(KeyboardKey::KEY_SPACE);

        let mut yaw_in = 0.0_f32;
        let mut pitch_in = 0.0_f32;
        let mut roll_in = 0.0_f32;
        if follow_mode {
            if window.is_key_down(KeyboardKey::KEY_LEFT) {
                yaw_in += 1.0;
            }
            if window.is_key_down(KeyboardKey::KEY_RIGHT) {
                yaw_in -= 1.0;
            }
            if window.is_key_down(KeyboardKey::KEY_UP) {
                pitch_in += 1.0;
            }
            if window.is_key_down(KeyboardKey::KEY_DOWN) {
                pitch_in -= 1.0;
            }
            let mouse_delta = window.get_mouse_delta();
            yaw_in = (yaw_in - mouse_delta.x * 0.04).clamp(-1.0, 1.0);
            pitch_in = (pitch_in - mouse_delta.y * 0.04).clamp(-1.0, 1.0);
        }
        if window.is_key_down(KeyboardKey::KEY_Q) {
            roll_in -= 1.0;
        }
        if window.is_key_down(KeyboardKey::KEY_E) {
            roll_in += 1.0;
        }

        let mut collision_body: Option<usize> = None;
        physics_stepper.step(dt, |sub_dt| {
            ship.apply_thrust(throttle, boost, sub_dt);
            ship.apply_damping(braking, sub_dt);
            ship.apply_angular_input(pitch_in, yaw_in, roll_in, sub_dt);
            gravity_mag = physics::integrate_ship_with_gravity(
                &mut ship,
                &bodies,
                sim_clock.time,
                sub_dt,
                gravity_enabled,
            );
            if let Some(idx) = physics::resolve_body_collisions(&mut ship, &bodies, sim_clock.time)
            {
                collision_body = Some(idx);
            }
        });

        cam_distance = (cam_distance - window.get_mouse_wheel_move() * 1.5).clamp(2.5, 120.0);

        let aspect = internal_width as f32 / internal_height as f32;
        let cam = if follow_mode {
            follow_cam.update(&ship, dt);
            follow_cam.to_camera(45.0_f32.to_radians(), 0.1, 300.0)
        } else {
            let cam_pos = Vector3::new(
                cam_distance * cam_pitch.cos() * cam_yaw.sin(),
                cam_distance * cam_pitch.sin(),
                cam_distance * cam_pitch.cos() * cam_yaw.cos(),
            );
            Camera::new(cam_pos, Vector3::zero(), 45.0_f32.to_radians(), 0.1, 300.0)
        };
        let base_light = Vector3::new(-0.65, 0.80, 1.0).normalize();
        let light_dir = Vector3::new(
            base_light.x * light_angle.cos() - base_light.z * light_angle.sin(),
            base_light.y,
            base_light.x * light_angle.sin() + base_light.z * light_angle.cos(),
        )
        .normalize();
        let mut uniforms = Uniforms {
            model_matrix: Mat4::identity(),
            view_matrix: cam.view_matrix(),
            projection_matrix: cam.projection_matrix(aspect),
            time: sim_clock.time,
            light_dir,
            camera_pos: cam.position,
            debug_noise,
            noise_frequency,
            rocky_debug_view,
            viewport_width: internal_width,
            viewport_height: internal_height,
            noise: noise.clone(),
            baked: Some(Arc::clone(&baked)),
        };

        let t_clear = std::time::Instant::now();
        framebuffer.clear();
        let clear_ms = t_clear.elapsed().as_secs_f64() * 1000.0;

        let t_vtx = std::time::Instant::now();
        render_buffers.screen_triangles.clear();
        render_buffers.tri_materials.clear();
        let sun_pos = body_position_at(&bodies[0], sim_clock.time);
        for body in &bodies {
            let body_pos = body_position_at(body, sim_clock.time);
            let to_sun = sun_pos - body_pos;
            let body_light = if to_sun.length_sqr() > 1e-6 {
                to_sun.normalize()
            } else {
                light_dir
            };
            uniforms.model_matrix = body_model_matrix_at(&obj, body, sim_clock.time);
            let before = render_buffers.screen_triangles.len();
            append_screen_triangles(
                &vertex_array,
                &uniforms,
                &mut render_buffers.clip_vertices,
                &mut render_buffers.screen_triangles,
            );
            let added = render_buffers.screen_triangles.len() - before;
            render_buffers.tri_materials.resize(
                before + added,
                TriMaterial {
                    shader: body.shader,
                    light_dir: body_light,
                },
            );
        }

        // Dibujar nave (LOD0)
        let ship_to_sun = sun_pos - ship.position;
        let ship_light = if ship_to_sun.length_sqr() > 1e-6 {
            ship_to_sun.normalize()
        } else {
            light_dir
        };
        uniforms.model_matrix = ship.model_matrix(ship_center, ship_unit_scale);
        let before_ship = render_buffers.screen_triangles.len();
        append_screen_triangles(
            &ship_vertex_array,
            &uniforms,
            &mut render_buffers.clip_vertices,
            &mut render_buffers.screen_triangles,
        );
        let added_ship = render_buffers.screen_triangles.len() - before_ship;
        render_buffers.tri_materials.resize(
            before_ship + added_ship,
            TriMaterial {
                shader: Shader::Ship,
                light_dir: ship_light,
            },
        );
        let vertex_ms = t_vtx.elapsed().as_secs_f64() * 1000.0;

        let (raster_ms, shading_ms, fragments, shader_calls, visible_pixels) = match mode {
            TriangleMode::FilledBarycentric => {
                let t_rast = std::time::Instant::now();
                let writes = rasterize_visibility_tiled(
                    &render_buffers.screen_triangles,
                    &mut render_buffers.tile_grid,
                    &mut framebuffer,
                    backface_culling,
                    threads,
                );
                let raster_ms = t_rast.elapsed().as_secs_f64() * 1000.0;

                let t_shade = std::time::Instant::now();
                let (calls, visible) = shade_visibility_tiled_materials(
                    &render_buffers.screen_triangles,
                    &render_buffers.tri_materials,
                    &mut framebuffer,
                    shader,
                    &uniforms,
                    threads,
                );
                let shading_ms = t_shade.elapsed().as_secs_f64() * 1000.0;
                (raster_ms, shading_ms, writes, calls, visible)
            }
            TriangleMode::Wireframe => {
                let t_wire = std::time::Instant::now();
                let (calls, visible) = render_wireframe(
                    &render_buffers.screen_triangles,
                    &mut framebuffer,
                    shader,
                    &uniforms,
                );
                let wire_ms = t_wire.elapsed().as_secs_f64() * 1000.0;
                (0.0, wire_ms, calls, calls, visible)
            }
        };

        let mut sample = FrameSample {
            vertex_ms,
            raster_ms,
            shading_ms,
            fb_ms: clear_ms,
            present_ms: 0.0,
            total_ms: 0.0,
            fragments,
            shader_calls,
            visible_pixels,
            width: framebuffer.width,
            height: framebuffer.height,
            threads,
        };
        if capture_requested {
            let capture_path = next_capture_path();
            framebuffer.render_to_file(capture_path.to_str().expect("Ruta de captura inválida"));
            println!("Captura guardada en {}", capture_path.display());
        }
        let label = format!(
            "Sistema Solar ({} cuerpos) | t={:.1}s ({:.2}x{}) | Cam(F): {} | Ayuda(H)",
            bodies.len(),
            sim_clock.time,
            sim_clock.scale,
            if sim_clock.paused { " PAUSA" } else { "" },
            if follow_mode { "Seguir" } else { "Libre" },
        );
        let mut overlay = Vec::new();
        overlay.push(format!(
            "Vel: {:.2} u/s | Gravedad(G): {} ({:.2} u/s^2)",
            ship.velocity.length(),
            if gravity_enabled { "ON" } else { "OFF" },
            gravity_mag,
        ));
        if let Some(idx) = collision_body {
            overlay.push(format!("COLISION: {} (superficie)", bodies[idx].name));
        }
        if perf_stats.enabled {
            overlay.extend(perf_stats.overlay_lines());
        }
        if show_help {
            overlay.extend([
                "--- Controles de vuelo (H para cerrar) ---".to_string(),
                "W / S       : Empuje adelante / atras".to_string(),
                "Shift       : Impulso (boost)".to_string(),
                "Espacio     : Frenado progresivo".to_string(),
                "Mouse/Flech.: Cabeceo y guinada".to_string(),
                "Q / E       : Alabeo (roll)".to_string(),
                "G           : Activar / desactivar gravedad".to_string(),
                "F / Backsp. : Camara seguir/libre / Recentrar".to_string(),
                "Tab / - =   : Pausa / Escala de tiempo".to_string(),
            ]);
        }
        let t_pres = std::time::Instant::now();
        framebuffer.draw_in_window(&mut window, &thread, &label, &overlay);
        sample.present_ms = t_pres.elapsed().as_secs_f64() * 1000.0;
        sample.total_ms = frame_start.elapsed().as_secs_f64() * 1000.0;
        perf_stats.push(sample);
    }
}
