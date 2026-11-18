mod camera;
mod fragment;
mod framebuffer;
mod math;
mod noise;
mod obj;
mod perf;
mod primitive;
mod rasterizer;
mod scene;
mod shaders;
mod surface_map;
mod vertex;

use camera::Camera;
use framebuffer::Framebuffer;
use math::Mat4;
use noise::Noise;
use obj::load_obj;
use perf::{FrameSample, PerfStats};
use primitive::{ScreenTriangle, assemble_screen_triangles};
use rasterizer::{
    TILE_SIZE, TileGrid, TriangleMode, WINDOW_HEIGHT, WINDOW_WIDTH, rasterize_visibility_tiled,
    render_wireframe, shade_visibility,
};
use raylib::prelude::*;
use scene::build_model_matrix;
use shaders::{Shader, shader_name};
use surface_map::{BakedMaps, DEFAULT_BAKE_HEIGHT, DEFAULT_BAKE_WIDTH};
use std::fs;
use std::path::PathBuf;

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
    pub baked: Option<BakedMaps>,
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
    transformed_vertices: Vec<vertex::Vertex>,
    screen_triangles: Vec<ScreenTriangle>,
    tile_grid: TileGrid,
}

impl RenderBuffers {
    fn new(vertex_capacity: usize) -> Self {
        Self {
            transformed_vertices: Vec::with_capacity(vertex_capacity),
            screen_triangles: Vec::with_capacity(vertex_capacity / 3),
            tile_grid: TileGrid::new(WINDOW_WIDTH as usize, WINDOW_HEIGHT as usize),
        }
    }
}

fn render(
    framebuffer: &mut Framebuffer,
    uniforms: &Uniforms,
    vertex_array: &[vertex::Vertex],
    buffers: &mut RenderBuffers,
    mode: TriangleMode,
    shader: Shader,
    backface_culling: bool,
) -> FrameSample {
    let t_vtx = std::time::Instant::now();
    assemble_screen_triangles(
        vertex_array,
        uniforms,
        &mut buffers.transformed_vertices,
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
            );
            let _ = TILE_SIZE;
            let raster_ms = t_rast.elapsed().as_secs_f64() * 1000.0;

            let t_shade = std::time::Instant::now();
            let (calls, visible) =
                shade_visibility(&buffers.screen_triangles, framebuffer, shader, uniforms);
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
    }
}

fn main() {
    let obj_path = "assets/sphere.obj";
    let obj = load_obj(obj_path).expect("No se pudo cargar assets/sphere.obj");
    println!("Modelo cargado: '{obj_path}'");
    println!("  Vértices:   {}", obj.vertices.len());
    println!("  Normales fuente (vn): {}", obj.source_normals.len());
    println!("  Triángulos: {}", obj.indices.len() / 3);
    let normals_are_unit = obj
        .vertex_normals
        .iter()
        .all(|normal| (normal.length() - 1.0).abs() <= 1e-4);
    println!("  Normales suaves unitarias (~1): {normals_are_unit}");

    let vertex_array: Vec<vertex::Vertex> = obj
        .indices
        .iter()
        .map(|&index| vertex::Vertex::new(obj.vertices[index], obj.vertex_normals[index]))
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
    let mut baked = BakedMaps::bake(&noise, DEFAULT_BAKE_WIDTH, DEFAULT_BAKE_HEIGHT);
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
            baked: Some(baked.clone()),
        };
        let mut render_buffers = RenderBuffers::new(vertex_array.len());
        framebuffer.clear();
        render(
            &mut framebuffer,
            &uniforms,
            &vertex_array,
            &mut render_buffers,
            TriangleMode::FilledBarycentric,
            options.shader,
            true,
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
    let mut cam_pitch = 0.0_f32;
    let mut cam_distance = 3.0_f32;
    let mut time = 0.0_f32;
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
    let orbit_speed = 1.8_f32;
    let light_speed = 1.8_f32;
    let mut perf_stats = PerfStats::new();
    let mut render_buffers = RenderBuffers::new(vertex_array.len());

    while !window.window_should_close() {
        let frame_start = std::time::Instant::now();
        let dt = window.get_frame_time();
        time += dt;
        if window.is_key_pressed(KeyboardKey::KEY_F3) {
            perf_stats.enabled = !perf_stats.enabled;
        }
        if window.is_key_pressed(KeyboardKey::KEY_C) {
            backface_culling = !backface_culling;
        }
        if window.is_key_pressed(KeyboardKey::KEY_K) {
            let t_bake = std::time::Instant::now();
            baked = BakedMaps::bake(&noise, DEFAULT_BAKE_WIDTH, DEFAULT_BAKE_HEIGHT);
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
        if window.is_key_pressed(KeyboardKey::KEY_W) {
            mode = TriangleMode::Wireframe;
        }
        if window.is_key_pressed(KeyboardKey::KEY_B) {
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
        if window.is_key_pressed(KeyboardKey::KEY_D) {
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
        cam_distance = (cam_distance - window.get_mouse_wheel_move() * 0.4).clamp(1.2, 20.0);

        let aspect = internal_width as f32 / internal_height as f32;
        let model_matrix = build_model_matrix(&obj, 0.0, 0.0, base_scale);
        let cam_pos = Vector3::new(
            cam_distance * cam_pitch.cos() * cam_yaw.sin(),
            cam_distance * cam_pitch.sin(),
            cam_distance * cam_pitch.cos() * cam_yaw.cos(),
        );
        let cam = Camera::new(cam_pos, Vector3::zero(), 45.0_f32.to_radians(), 0.1, 100.0);
        // En la convención del renderer, +z es el frente visible hacia la cámara.
        let base_light = Vector3::new(-0.65, 0.80, 1.0).normalize();
        let light_dir = Vector3::new(
            base_light.x * light_angle.cos() - base_light.z * light_angle.sin(),
            base_light.y,
            base_light.x * light_angle.sin() + base_light.z * light_angle.cos(),
        )
        .normalize();
        let uniforms = Uniforms {
            model_matrix,
            view_matrix: cam.view_matrix(),
            projection_matrix: cam.projection_matrix(aspect),
            time,
            light_dir,
            camera_pos: cam.position,
            debug_noise,
            noise_frequency,
            rocky_debug_view,
            viewport_width: internal_width,
            viewport_height: internal_height,
            noise: noise.clone(),
            baked: Some(baked.clone()),
        };

        let t_clear = std::time::Instant::now();
        framebuffer.clear();
        let clear_ms = t_clear.elapsed().as_secs_f64() * 1000.0;
        let mut sample = render(
            &mut framebuffer,
            &uniforms,
            &vertex_array,
            &mut render_buffers,
            mode,
            shader,
            backface_culling,
        );
        sample.fb_ms += clear_ms;
        if capture_requested {
            let capture_path = next_capture_path();
            framebuffer.render_to_file(capture_path.to_str().expect("Ruta de captura inválida"));
            println!("Captura guardada en {}", capture_path.display());
        }
        let debug_label = if debug_noise { " | DEBUG FBM" } else { "" };
        let label = format!(
            "{}{} | Rocky: {} | Cull(C): {} | Noise: {:.2}",
            shader_name(shader),
            debug_label,
            ["altura", "bioma", "grietas", "color"][rocky_debug_view as usize],
            if backface_culling { "ON" } else { "OFF" },
            noise_frequency,
        );
        let overlay = if perf_stats.enabled {
            perf_stats.overlay_lines()
        } else {
            Vec::new()
        };
        let t_pres = std::time::Instant::now();
        framebuffer.draw_in_window(&mut window, &thread, &label, &overlay);
        sample.present_ms = t_pres.elapsed().as_secs_f64() * 1000.0;
        sample.total_ms = frame_start.elapsed().as_secs_f64() * 1000.0;
        perf_stats.push(sample);
    }
}
