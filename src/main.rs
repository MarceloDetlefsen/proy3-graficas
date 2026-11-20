mod app;
mod bodies;
mod camera;
mod clipping;
mod culling;
mod docs_capture;
mod fragment;
mod framebuffer;
mod gallery;
mod hud;
mod icosphere;
mod lod;
mod logo_data;
mod racket_data;
mod samus_bombonera_data;
mod math;
mod mesh_builder;
mod mesh_lod;
mod nav;
mod noise;
mod obj;
mod perf;
mod physics;
mod primitive;
mod rasterizer;
mod rng;
mod scene;
mod scene_config;
mod shaders;
mod ship;
mod showcase;
mod sim;
mod sky;
mod surface_map;
mod surface_props;
mod surface_scatter;
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
use scene::{
    body_model_matrix_at, body_model_matrix_unit, body_position_at, build_model_matrix,
    system_bodies,
};
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
    pub normal_mapping: bool,
    pub debug_normals: bool,
    pub viewport_width: i32,
    pub viewport_height: i32,
    pub noise: Noise,
    pub baked: Option<Arc<BakedMaps>>,
    pub shader_param: f32,
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
        tris_before_cull: vertex_array.len() / 3,
        tris_after_cull: buffers.screen_triangles.len(),
        width: framebuffer.width,
        height: framebuffer.height,
        threads,
    }
}

struct MenuBackdrop {
    bodies: Vec<scene::Body>,
    body_caches: Vec<icosphere::DeformedIcosphereCache>,
    body_lods: Vec<usize>,
    baked: Arc<BakedMaps>,
    time: f32,
    yaw: f32,
}

impl MenuBackdrop {
    fn new(baked: Arc<BakedMaps>) -> Self {
        let bodies = system_bodies(app::SystemId::Realista);
        let body_caches = bodies
            .iter()
            .map(|b| {
                icosphere::DeformedIcosphereCache::new_for_shader(
                    b.shader,
                    b.surface.deformation_amplitude,
                    b.surface.noise_frequency,
                )
            })
            .collect();
        let body_lods = vec![2_usize; bodies.len()];
        Self {
            bodies,
            body_caches,
            body_lods,
            baked,
            time: 0.0,
            yaw: 0.4,
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn render_frame(
        &mut self,
        dt: f32,
        framebuffer: &mut Framebuffer,
        render_buffers: &mut RenderBuffers,
        noise: &Noise,
        threads: usize,
        internal_width: i32,
        internal_height: i32,
    ) {
        self.time += dt;
        self.yaw += 0.14 * dt;

        let cam_distance = 44.0_f32;
        let cam_pitch = 0.36_f32;
        let cam_pos = Vector3::new(
            cam_distance * cam_pitch.cos() * self.yaw.sin(),
            cam_distance * cam_pitch.sin(),
            cam_distance * cam_pitch.cos() * self.yaw.cos(),
        );
        let cam = Camera::new(cam_pos, Vector3::zero(), 45.0_f32.to_radians(), 0.1, 300.0);
        let aspect = internal_width as f32 / internal_height as f32;
        let light_dir = Vector3::new(-0.65, 0.80, 1.0).normalize();
        let mut uniforms = Uniforms {
            model_matrix: Mat4::identity(),
            view_matrix: cam.view_matrix(),
            projection_matrix: cam.projection_matrix(aspect),
            time: self.time,
            light_dir,
            camera_pos: cam.position,
            debug_noise: false,
            noise_frequency: 4.0,
            rocky_debug_view: 3,
            normal_mapping: true,
            debug_normals: false,
            viewport_width: internal_width,
            viewport_height: internal_height,
            noise: noise.clone(),
            baked: Some(Arc::clone(&self.baked)),
            shader_param: 1.0,
        };

        framebuffer.clear();
        render_buffers.screen_triangles.clear();
        render_buffers.tri_materials.clear();
        let vp = uniforms.projection_matrix * uniforms.view_matrix;
        let frustum = culling::Frustum::from_vp(&vp);
        let sun_pos = self
            .bodies
            .first()
            .map(|b| body_position_at(b, self.time))
            .unwrap_or_else(Vector3::zero);

        for (body_idx, body) in self.bodies.iter().enumerate() {
            let body_pos = body_position_at(body, self.time);
            if !frustum.intersects_sphere(body_pos, scene::effective_radius(body)) {
                continue;
            }
            let radius_px = lod::projected_radius_px(
                &cam,
                body_pos,
                scene::effective_radius(body),
                internal_height as f32,
            );
            let level = lod::select_detail_level_hysteresis(
                self.body_lods[body_idx],
                radius_px,
                &lod::ICOSPHERE_THRESHOLDS_PX,
                lod::DEFAULT_HYSTERESIS,
            )
            .min(3);
            self.body_lods[body_idx] = level;
            let body_mesh = self.body_caches[body_idx].get_or_build(level, noise);
            let body_light = scene::body_light_dir(body_pos, sun_pos, light_dir);
            uniforms.model_matrix = body_model_matrix_unit(body, self.time);
            let before = render_buffers.screen_triangles.len();
            append_screen_triangles(
                body_mesh,
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
                    shader_param: 1.0,
                },
            );
        }

        rasterize_visibility_tiled(
            &render_buffers.screen_triangles,
            &mut render_buffers.tile_grid,
            framebuffer,
            true,
            threads,
        );
        shade_visibility_tiled_materials(
            &render_buffers.screen_triangles,
            &render_buffers.tri_materials,
            framebuffer,
            Shader::Rocky,
            &uniforms,
            threads,
        );
    }
}

struct FlightSession {
    system_id: app::SystemId,
    bodies: Vec<scene::Body>,
    body_shader_params: Vec<f32>,
    body_caches: Vec<icosphere::DeformedIcosphereCache>,
    proc_caches: Vec<Option<mesh_builder::LodMeshCache>>,
    body_lods: Vec<usize>,
    planet_candidates: Vec<Vec<surface_scatter::SurfaceCandidate>>,
    planet_instances: Vec<Vec<surface_scatter::SurfaceInstance>>,
    baked: Arc<BakedMaps>,
    sim_clock: SimClock,
    time_scale_idx: usize,
    ship: Ship,
    ship_lod: usize,
    follow_cam: FollowCamera,
    follow_mode: bool,
    physics_stepper: physics::PhysicsStepper,
    nav_state: nav::NavState,
    gravity_enabled: bool,
    gravity_mag: f32,
    selected_body: usize,
    forced_lod: Option<usize>,
    surface_seed: u64,
    noise_frequency: f32,
}

impl FlightSession {
    fn new(system_id: app::SystemId, noise: &Noise) -> Self {
        let bodies = system_bodies(system_id);
        let body_shader_params = bodies
            .iter()
            .map(|b| shaders::live_param(b.shader).default)
            .collect();
        let body_caches = bodies
            .iter()
            .map(|b| {
                icosphere::DeformedIcosphereCache::new_for_shader(
                    b.shader,
                    b.surface.deformation_amplitude,
                    b.surface.noise_frequency,
                )
            })
            .collect();
        let proc_caches = bodies
            .iter()
            .map(|b| {
                if system_id == app::SystemId::Favoritos {
                    showcase::SHOWCASE_ENTRIES
                        .iter()
                        .find(|e| e.sistema == app::SystemId::Favoritos && e.nombre == b.name)
                        .and_then(|e| showcase::mesh_builder_for_showcase(e.id))
                        .map(mesh_builder::LodMeshCache::new)
                } else {
                    None
                }
            })
            .collect();
        let body_lods = vec![3_usize; bodies.len()];
        let surface_seed: u64 = 42;
        let planet_candidates: Vec<_> = bodies
            .iter()
            .map(|body| surface_scatter::scatter_for_body(body, surface_seed))
            .collect();
        let planet_instances: Vec<_> = bodies
            .iter()
            .zip(planet_candidates.iter())
            .map(|(b, c)| surface_scatter::build_body_instances(b, c, noise))
            .collect();
        let baked = Arc::new(BakedMaps::bake(
            noise,
            DEFAULT_BAKE_WIDTH,
            DEFAULT_BAKE_HEIGHT,
        ));
        let ship_start = if system_id == app::SystemId::Favoritos {
            Vector3::new(0.0, 2.2, 22.0)
        } else {
            Vector3::new(0.0, 1.5, 16.0)
        };
        let ship = Ship::new(ship_start);
        let follow_cam = FollowCamera::new(&ship, 5.5, 1.6);
        Self {
            system_id,
            bodies,
            body_shader_params,
            body_caches,
            proc_caches,
            body_lods,
            planet_candidates,
            planet_instances,
            baked,
            sim_clock: SimClock::new(),
            time_scale_idx: 2,
            ship,
            ship_lod: 0,
            follow_cam,
            follow_mode: true,
            physics_stepper: physics::PhysicsStepper::new(),
            nav_state: nav::NavState::new(),
            gravity_enabled: true,
            gravity_mag: 0.0,
            selected_body: 0,
            forced_lod: None,
            surface_seed,
            noise_frequency: 4.0,
        }
    }
}

fn main() {
    if std::env::args().any(|a| a == "--docs") {
        docs_capture::generate_docs_captures();
        return;
    }

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
    let baked = Arc::new(BakedMaps::bake(
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
            normal_mapping: true,
            debug_normals: false,
            viewport_width: internal_width,
            viewport_height: internal_height,
            noise: noise.clone(),
            baked: Some(Arc::clone(&baked)),
            shader_param: 1.0,
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
    window.set_exit_key(None);

    let mut mode = TriangleMode::FilledBarycentric;
    let mut cam_yaw = 0.0_f32;
    let mut cam_pitch = 0.45_f32;
    let mut cam_distance = 34.0_f32;
    let time_scales = [0.25_f32, 0.5, 1.0, 2.0, 5.0];
    let ship_lods = load_ship_lods();
    let (ship_v0, _ship_i0) = &ship_lods.levels[0];
    let (ship_min, ship_max) = mesh_bounds(ship_v0);
    let ship_center = (ship_min + ship_max) * 0.5;
    let ship_extent = (ship_max.x - ship_min.x)
        .max(ship_max.y - ship_min.y)
        .max(ship_max.z - ship_min.z)
        .max(1e-4);
    let ship_unit_scale = 2.0 / ship_extent;
    let ship_vertex_arrays: [Vec<vertex::Vertex>; 3] = std::array::from_fn(|lvl| {
        let (verts, idxs) = &ship_lods.levels[lvl];
        idxs.iter().map(|&i| verts[i].clone()).collect()
    });
    let mut overlay_buf = hud::OverlayBuffer::new(WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32);
    let mut show_help = false;
    let mut show_hud = true;
    let mut app_ctrl = app::AppController::new();
    let mut should_exit = false;
    window.enable_cursor();
    let audio = raylib::core::audio::RaylibAudio::init_audio_device().ok();
    let bg_music_path = ["Souvlaki Space Station.mp3", "assets/Souvlaki Space Station.mp3"]
        .into_iter()
        .find(|p| std::path::Path::new(p).exists());
    let mut bg_music = audio.as_ref().and_then(|aud| {
        let path = bg_music_path?;
        let mut music = aud.new_music(path).ok()?;
        music.set_looping(true);
        music.play_stream();
        Some(music)
    });
    let icosphere_lods = icosphere::build_icosphere_lods();
    let prop_meshes = surface_props::load_surface_props();
    let mut flight_session: Option<FlightSession> = None;
    let mut gallery_viewer: Option<gallery::GalleryViewer> = None;
    let mut gallery_selector: Option<gallery::GallerySelector> = None;
    let mut gallery_time = 0.0_f32;
    let mut gallery_fps = 60.0_f64;
    let mut menu_backdrop = MenuBackdrop::new(Arc::clone(&baked));
    let _ = (build_model_matrix, body_model_matrix_at);
    let _ = shader_name;
    let mut shader = Shader::Rocky;
    let mut debug_noise = false;
    let mut rocky_debug_view = 3_i32;
    let mut normal_mapping = true;
    let mut debug_normals = false;
    let mut light_angle = 0.0_f32;
    let mut backface_culling = true;
    let mut frustum_culling = true;
    let mut instance_culling = true;
    const MAX_PROP_DISTANCE: f32 = 32.0;
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

    while !window.window_should_close() && !should_exit {
        if let Some(ref mut music) = bg_music {
            music.update_stream();
        }
        let frame_start = std::time::Instant::now();
        let dt = window.get_frame_time();
        if window.is_key_pressed(KeyboardKey::KEY_ESCAPE) {
            match app_ctrl.on_escape() {
                app::TransitionOutcome::ExitApp => {
                    should_exit = true;
                    continue;
                }
                app::TransitionOutcome::ChangeState(new_state) => {
                    match new_state {
                        app::AppState::Flight(sys_id) => {
                            let session = flight_session
                                .insert(FlightSession::new(sys_id, &noise));
                            perf_stats = PerfStats::new();
                            if session.follow_mode {
                                window.disable_cursor();
                            } else {
                                window.enable_cursor();
                            }
                        }
                        _ => {
                            flight_session = None;
                            gallery_viewer = None;
                            gallery_selector = None;
                            window.enable_cursor();
                        }
                    }
                }
                _ => {}
            }
        }
        if matches!(app_ctrl.state, app::AppState::Menu) {
            let fav_ready = showcase::is_favorites_system_ready(showcase::SHOWCASE_ENTRIES);
            if window.is_key_pressed(KeyboardKey::KEY_UP) || window.is_key_pressed(KeyboardKey::KEY_W) {
                app_ctrl.move_menu_up();
            }
            if window.is_key_pressed(KeyboardKey::KEY_DOWN) || window.is_key_pressed(KeyboardKey::KEY_S) {
                app_ctrl.move_menu_down();
            }
            if app_ctrl.show_menu_controls
                && (window.is_key_pressed(KeyboardKey::KEY_BACKSPACE)
                    || window.is_key_pressed(KeyboardKey::KEY_H))
            {
                app_ctrl.show_menu_controls = false;
            }
            if window.is_key_pressed(KeyboardKey::KEY_ENTER) {
                match app_ctrl.activate_menu(fav_ready) {
                    app::TransitionOutcome::ExitApp => {
                        should_exit = true;
                        continue;
                    }
                    app::TransitionOutcome::ChangeState(app::AppState::Flight(sys_id)) => {
                        let session = flight_session.insert(FlightSession::new(sys_id, &noise));
                        perf_stats = PerfStats::new();
                        if session.follow_mode {
                            window.disable_cursor();
                        } else {
                            window.enable_cursor();
                        }
                    }
                    app::TransitionOutcome::ChangeState(_) => {
                        flight_session = None;
                        window.enable_cursor();
                    }
                    _ => {}
                }
            }
            menu_backdrop.render_frame(
                dt,
                &mut framebuffer,
                &mut render_buffers,
                &noise,
                threads,
                internal_width,
                internal_height,
            );
            overlay_buf.resize(
                window.get_screen_width().max(1) as u32,
                window.get_screen_height().max(1) as u32,
            );
            hud::render_menu_overlay(
                &mut overlay_buf,
                app_ctrl.selected_menu,
                fav_ready,
                app_ctrl.show_menu_controls,
            );
            framebuffer.draw_with_overlay(&mut window, &thread, &overlay_buf);
            continue;
        }
        if matches!(app_ctrl.state, app::AppState::Gallery) {
            let selector = gallery_selector.get_or_insert_with(gallery::GallerySelector::new);
            let mut body_changed = false;
            if window.is_key_pressed(KeyboardKey::KEY_A) {
                selector.select_prev();
                body_changed = true;
            }
            if window.is_key_pressed(KeyboardKey::KEY_D) {
                selector.select_next();
                body_changed = true;
            }
            let body_idx = selector.current_body_idx();
            let body = *selector.current_body();
            let showcase_entry = selector.current_showcase();
            let viewer = gallery_viewer.get_or_insert_with(|| gallery::GalleryViewer::new(body.radius));
            if body_changed {
                viewer.reset_view_for(body.radius, showcase_entry.id);
            }
            if window.is_key_pressed(KeyboardKey::KEY_FOUR) {
                selector.adjust_param(&body, -1.0);
            }
            if window.is_key_pressed(KeyboardKey::KEY_FIVE) {
                selector.adjust_param(&body, 1.0);
            }
            let param = selector.current_param();
            gallery_time += dt;
            viewer.advance(dt, &body, param);

            if window.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT) {
                let md = window.get_mouse_delta();
                viewer.apply_orbit_delta(-md.x * 0.008, md.y * 0.008);
            }
            if window.is_key_down(KeyboardKey::KEY_LEFT) {
                viewer.apply_orbit_delta(-1.8 * dt, 0.0);
            }
            if window.is_key_down(KeyboardKey::KEY_RIGHT) {
                viewer.apply_orbit_delta(1.8 * dt, 0.0);
            }
            if window.is_key_down(KeyboardKey::KEY_UP) {
                viewer.apply_orbit_delta(0.0, 1.8 * dt);
            }
            if window.is_key_down(KeyboardKey::KEY_DOWN) {
                viewer.apply_orbit_delta(0.0, -1.8 * dt);
            }
            let wheel = window.get_mouse_wheel_move();
            if wheel.abs() > 1e-4 {
                viewer.apply_zoom(wheel, body.radius);
            }
            if window.is_key_pressed(KeyboardKey::KEY_R) {
                viewer.reset_view_for(body.radius, showcase_entry.id);
            }

            let t_gal = std::time::Instant::now();
            let tris = viewer.render_body(
                body_idx,
                showcase_entry.id,
                &body,
                showcase_entry.light_dir,
                param,
                gallery_time,
                &noise,
                &baked,
                &mut framebuffer,
                &mut render_buffers,
                mode,
                threads,
            );
            let spec = shaders::live_param(body.shader);
            let snap = hud::GalleryHudSnapshot {
                entry: showcase_entry,
                active_showcase_idx: selector.current_showcase_idx(),
                radius: body.radius,
                tris,
                lod_level: viewer.lod_level,
                shader_name: shader_name(body.shader),
                live_param_name: spec.name,
                live_param_value: param,
                fps: gallery_fps,
            };
            overlay_buf.resize(
                window.get_screen_width().max(1) as u32,
                window.get_screen_height().max(1) as u32,
            );
            hud::render_gallery_overlay(&mut overlay_buf, &snap);
            framebuffer.draw_with_overlay(&mut window, &thread, &overlay_buf);
            let frame_ms = t_gal.elapsed().as_secs_f64() * 1000.0;
            if frame_ms > 0.0 {
                gallery_fps = gallery_fps * 0.85 + (1000.0 / frame_ms) * 0.15;
            }
            viewer.record_frame_time(frame_ms);
            continue;
        }
        let sys_id = match app_ctrl.state {
            app::AppState::Flight(id) => id,
            _ => app::SystemId::Realista,
        };
        let session = flight_session.get_or_insert_with(|| FlightSession::new(sys_id, &noise));
        if session.system_id != sys_id {
            *session = FlightSession::new(sys_id, &noise);
        }
        let _sim_dt = session.sim_clock.advance(dt);
        if window.is_key_pressed(KeyboardKey::KEY_F) {
            session.follow_mode = !session.follow_mode;
            if session.follow_mode {
                window.disable_cursor();
                session.follow_cam.snap(&session.ship);
            } else {
                window.enable_cursor();
            }
        }
        if window.is_key_pressed(KeyboardKey::KEY_BACKSPACE) {
            session.follow_cam.recenter();
        }
        if window.is_key_pressed(KeyboardKey::KEY_ENTER) {
            session.sim_clock.toggle_pause();
        }
        if window.is_key_pressed(KeyboardKey::KEY_TAB) && !session.bodies.is_empty() {
            session.selected_body = (session.selected_body + 1) % session.bodies.len();
        }
        if window.is_key_pressed(KeyboardKey::KEY_NINE) && !session.bodies.is_empty() {
            let idx = session.selected_body;
            let b = &mut session.bodies[idx];
            b.radius = (b.radius - 0.1).max(0.3);
            b.surface.max_deformation = b.radius * b.surface.deformation_amplitude;
            session.planet_instances[idx] =
                surface_scatter::build_body_instances(b, &session.planet_candidates[idx], &noise);
        }
        if window.is_key_pressed(KeyboardKey::KEY_ZERO) && !session.bodies.is_empty() {
            let idx = session.selected_body;
            let b = &mut session.bodies[idx];
            b.radius = (b.radius + 0.1).min(6.0);
            b.surface.max_deformation = b.radius * b.surface.deformation_amplitude;
            session.planet_instances[idx] =
                surface_scatter::build_body_instances(b, &session.planet_candidates[idx], &noise);
        }
        if window.is_key_pressed(KeyboardKey::KEY_FOUR) && !session.bodies.is_empty() {
            let idx = session.selected_body;
            let spec = shaders::live_param(session.bodies[idx].shader);
            session.body_shader_params[idx] =
                (session.body_shader_params[idx] - spec.step).max(spec.min);
        }
        if window.is_key_pressed(KeyboardKey::KEY_FIVE) && !session.bodies.is_empty() {
            let idx = session.selected_body;
            let spec = shaders::live_param(session.bodies[idx].shader);
            session.body_shader_params[idx] =
                (session.body_shader_params[idx] + spec.step).min(spec.max);
        }
        if window.is_key_pressed(KeyboardKey::KEY_H) {
            show_help = !show_help;
        }
        if window.is_key_pressed(KeyboardKey::KEY_G) {
            session.gravity_enabled = !session.gravity_enabled;
        }
        if window.is_key_pressed(KeyboardKey::KEY_L) {
            session.nav_state.cycle_override();
        }
        if window.is_key_pressed(KeyboardKey::KEY_MINUS) {
            session.time_scale_idx = session.time_scale_idx.saturating_sub(1);
            session.sim_clock.set_scale(time_scales[session.time_scale_idx]);
        }
        if window.is_key_pressed(KeyboardKey::KEY_EQUAL) {
            session.time_scale_idx = (session.time_scale_idx + 1).min(time_scales.len() - 1);
            session.sim_clock.set_scale(time_scales[session.time_scale_idx]);
        }
        if window.is_key_pressed(KeyboardKey::KEY_F3) {
            perf_stats.enabled = !perf_stats.enabled;
        }
        if window.is_key_pressed(KeyboardKey::KEY_F4) {
            show_hud = !show_hud;
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
            session.baked = Arc::new(BakedMaps::bake(
                &noise,
                DEFAULT_BAKE_WIDTH,
                DEFAULT_BAKE_HEIGHT,
            ));
            println!(
                "Texturas de superficie recalculadas ({}x{}) en {:.2} ms.",
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
        if window.is_key_pressed(KeyboardKey::KEY_M) {
            mode = match mode {
                TriangleMode::FilledBarycentric => TriangleMode::Wireframe,
                TriangleMode::Wireframe => TriangleMode::FilledBarycentric,
            };
        }
        if window.is_key_pressed(KeyboardKey::KEY_X) {
            frustum_culling = !frustum_culling;
        }
        if window.is_key_pressed(KeyboardKey::KEY_Z) {
            session.forced_lod = match session.forced_lod {
                None => Some(0),
                Some(l) if l + 1 < icosphere_lods.len() => Some(l + 1),
                Some(_) => None,
            };
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
        if window.is_key_pressed(KeyboardKey::KEY_B) {
            normal_mapping = !normal_mapping;
        }
        if window.is_key_pressed(KeyboardKey::KEY_Y) {
            debug_normals = !debug_normals;
        }
        if window.is_key_pressed(KeyboardKey::KEY_I) {
            session.surface_seed = session.surface_seed.wrapping_sub(1);
            session.planet_candidates = session
                .bodies
                .iter()
                .map(|body| surface_scatter::scatter_for_body(body, session.surface_seed))
                .collect();
            session.planet_instances = session
                .bodies
                .iter()
                .zip(session.planet_candidates.iter())
                .map(|(b, c)| surface_scatter::build_body_instances(b, c, &noise))
                .collect();
        }
        if window.is_key_pressed(KeyboardKey::KEY_O) {
            session.surface_seed = session.surface_seed.wrapping_add(1);
            session.planet_candidates = session
                .bodies
                .iter()
                .map(|body| surface_scatter::scatter_for_body(body, session.surface_seed))
                .collect();
            session.planet_instances = session
                .bodies
                .iter()
                .zip(session.planet_candidates.iter())
                .map(|(b, c)| surface_scatter::build_body_instances(b, c, &noise))
                .collect();
        }
        if window.is_key_pressed(KeyboardKey::KEY_SLASH) {
            instance_culling = !instance_culling;
        }
        if window.is_key_down(KeyboardKey::KEY_J) {
            light_angle -= light_speed * dt;
        }
        if window.is_key_down(KeyboardKey::KEY_U) {
            light_angle += light_speed * dt;
        }
        let mut rebuild_surface = false;
        if window.is_key_pressed(KeyboardKey::KEY_LEFT_BRACKET) {
            session.noise_frequency = (session.noise_frequency - 0.5).max(0.5);
            rebuild_surface = true;
        }
        if window.is_key_pressed(KeyboardKey::KEY_RIGHT_BRACKET) {
            session.noise_frequency = (session.noise_frequency + 0.5).min(20.0);
            rebuild_surface = true;
        }
        let mut amp_delta = 0.0_f32;
        if window.is_key_pressed(KeyboardKey::KEY_COMMA) {
            amp_delta = -0.03;
        }
        if window.is_key_pressed(KeyboardKey::KEY_PERIOD) {
            amp_delta = 0.03;
        }
        if amp_delta != 0.0 || rebuild_surface {
            for (i, body) in session.bodies.iter_mut().enumerate() {
                if matches!(body.shader, Shader::Rocky) {
                    body.surface.deformation_amplitude =
                        (body.surface.deformation_amplitude + amp_delta).clamp(0.0, 0.40);
                    body.surface.max_deformation =
                        body.radius * body.surface.deformation_amplitude;
                    body.surface.noise_frequency = session.noise_frequency;
                    session.body_caches[i].invalidate(
                        body.surface.deformation_amplitude,
                        body.surface.noise_frequency,
                    );
                }
            }
            if rebuild_surface {
                session.baked = Arc::new(BakedMaps::bake_with_frequency(
                    &noise,
                    DEFAULT_BAKE_WIDTH,
                    DEFAULT_BAKE_HEIGHT,
                    session.noise_frequency,
                ));
            }
            session.planet_instances = session
                .bodies
                .iter()
                .zip(session.planet_candidates.iter())
                .map(|(b, c)| surface_scatter::build_body_instances(b, c, &noise))
                .collect();
        }
        let capture_requested = window.is_key_pressed(KeyboardKey::KEY_P);
        if !session.follow_mode {
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
        if session.follow_mode {
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
        let controls = nav::FlightControls {
            throttle,
            boost,
            braking,
        };
        session.physics_stepper.step(dt, |sub_dt| {
            session.nav_state.update_mode(
                session.ship.position,
                &session.bodies,
                session.sim_clock.time,
            );
            nav::apply_controls_in_frame(
                &mut session.ship,
                session.nav_state.mode,
                &session.bodies,
                session.sim_clock.time,
                controls,
                sub_dt,
            );
            let (_, turn_scale) = nav::local_flight_scales(
                session.ship.position,
                session.nav_state.mode,
                &session.bodies,
                session.sim_clock.time,
            );
            session.ship.apply_angular_input(
                pitch_in * turn_scale,
                yaw_in * turn_scale,
                roll_in * turn_scale,
                sub_dt,
            );
            session.gravity_mag = physics::integrate_ship_with_gravity(
                &mut session.ship,
                &session.bodies,
                session.sim_clock.time,
                sub_dt,
                session.gravity_enabled,
            );
            nav::align_up_to_surface(
                &mut session.ship,
                session.nav_state.mode,
                &session.bodies,
                session.sim_clock.time,
                sub_dt,
            );
            if let Some(idx) = physics::resolve_body_collisions(
                &mut session.ship,
                &session.bodies,
                session.sim_clock.time,
            ) {
                collision_body = Some(idx);
            }
        });

        cam_distance = (cam_distance - window.get_mouse_wheel_move() * 1.5).clamp(2.5, 220.0);

        let aspect = internal_width as f32 / internal_height as f32;
        let cam = if session.follow_mode {
            session.follow_cam.update(&session.ship, dt);
            session.follow_cam.to_camera(45.0_f32.to_radians(), 0.1, 300.0)
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
            time: session.sim_clock.time,
            light_dir,
            camera_pos: cam.position,
            debug_noise,
            noise_frequency: session.noise_frequency,
            rocky_debug_view,
            normal_mapping,
            debug_normals,
            viewport_width: internal_width,
            viewport_height: internal_height,
            noise: noise.clone(),
            baked: Some(Arc::clone(&session.baked)),
            shader_param: 1.0,
        };

        let t_clear = std::time::Instant::now();
        framebuffer.clear();
        let clear_ms = t_clear.elapsed().as_secs_f64() * 1000.0;

        let t_vtx = std::time::Instant::now();
        render_buffers.screen_triangles.clear();
        render_buffers.tri_materials.clear();
        let vp = uniforms.projection_matrix * uniforms.view_matrix;
        let frustum = culling::Frustum::from_vp(&vp);
        let sun_pos = session
            .bodies
            .first()
            .map(|b| body_position_at(b, session.sim_clock.time))
            .unwrap_or_else(Vector3::zero);
        let mut tris_before_cull = 0usize;
        let mut tris_after_cull = 0usize;
        let mut instances_total = 0usize;
        let mut instances_drawn = 0usize;
        for (body_idx, body) in session.bodies.iter().enumerate() {
            let body_pos = body_position_at(body, session.sim_clock.time);
            let radius_px = lod::projected_radius_px(
                &cam,
                body_pos,
                scene::effective_radius(body),
                internal_height as f32,
            );
            let level = match session.forced_lod {
                Some(f) => f.min(icosphere_lods.len() - 1),
                None => lod::select_detail_level_hysteresis(
                    session.body_lods[body_idx],
                    radius_px,
                    &lod::ICOSPHERE_THRESHOLDS_PX,
                    lod::DEFAULT_HYSTERESIS,
                ),
            };
            session.body_lods[body_idx] = level;
            let body_mesh = if let Some(proc_cache) = &mut session.proc_caches[body_idx] {
                proc_cache.get_or_build(level.min(2))
            } else {
                session.body_caches[body_idx].get_or_build(level, &noise)
            };
            let body_tris = body_mesh.len() / 3;
            tris_before_cull += body_tris;
            if frustum_culling && !frustum.intersects_sphere(body_pos, scene::culling_radius(body))
            {
                continue;
            }
            tris_after_cull += body_tris;
            let body_light = scene::body_light_dir(body_pos, sun_pos, light_dir);
            uniforms.model_matrix = body_model_matrix_unit(body, session.sim_clock.time);
            let before = render_buffers.screen_triangles.len();
            append_screen_triangles(
                body_mesh,
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
                    shader_param: session.body_shader_params[body_idx],
                },
            );

            // Dibujar instancias sobre la superficie compuestas con la matriz del planeta.
            let planet_rt = scene::body_rt_matrix_at(body, session.sim_clock.time);
            for inst in &session.planet_instances[body_idx] {
                instances_total += 1;
                let world_mat = planet_rt * inst.local_matrix;
                let inst_world = Vector3::new(
                    world_mat.cols[3][0],
                    world_mat.cols[3][1],
                    world_mat.cols[3][2],
                );
                let dist_sq = (inst_world - cam.position).length_sqr();
                let prop_lod = lod::select_instance_lod(dist_sq);
                let prop_mesh = prop_meshes.mesh_for(inst.kind, prop_lod);
                let prop_tris = prop_mesh.len() / 3;
                tris_before_cull += prop_tris;
                if instance_culling
                    && (dist_sq > MAX_PROP_DISTANCE * MAX_PROP_DISTANCE
                        || !frustum.intersects_sphere(inst_world, inst.bounding_radius))
                {
                    continue;
                }
                instances_drawn += 1;
                tris_after_cull += prop_tris;
                uniforms.model_matrix = world_mat;
                let b_inst = render_buffers.screen_triangles.len();
                append_screen_triangles(
                    prop_mesh,
                    &uniforms,
                    &mut render_buffers.clip_vertices,
                    &mut render_buffers.screen_triangles,
                );
                let added_inst = render_buffers.screen_triangles.len() - b_inst;
                render_buffers.tri_materials.resize(
                    b_inst + added_inst,
                    TriMaterial {
                        shader: Shader::Prop(inst.kind),
                        light_dir: body_light,
                        shader_param: 1.0,
                    },
                );
            }
        }

        // Dibujar nave con LOD según distancia a la cámara
        let ship_radius_px = lod::projected_radius_px(
            &cam,
            session.ship.position,
            session.ship.radius,
            internal_height as f32,
        );
        session.ship_lod = match session.forced_lod {
            Some(f) => f.min(2),
            None => {
                let ship_detail = lod::select_detail_level_hysteresis(
                    2 - session.ship_lod.min(2),
                    ship_radius_px,
                    &lod::SHIP_THRESHOLDS_PX,
                    lod::DEFAULT_HYSTERESIS,
                );
                2 - ship_detail
            }
        };
        let ship_mesh = &ship_vertex_arrays[session.ship_lod];
        let ship_tris = ship_mesh.len() / 3;
        tris_before_cull += ship_tris;
        if !frustum_culling
            || frustum.intersects_sphere(session.ship.position, session.ship.radius * 1.5)
        {
            tris_after_cull += ship_tris;
            let ship_light = scene::body_light_dir(session.ship.position, sun_pos, light_dir);
            uniforms.model_matrix = session.ship.model_matrix(ship_center, ship_unit_scale);
            let before_ship = render_buffers.screen_triangles.len();
            append_screen_triangles(
                ship_mesh,
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
                    shader_param: 1.0,
                },
            );
        }
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
                let (calls, visible) = rasterizer::shade_visibility_tiled_materials_halo(
                    &render_buffers.screen_triangles,
                    &render_buffers.tri_materials,
                    &mut framebuffer,
                    shader,
                    &uniforms,
                    threads,
                    scene_config::solar_halo_for_system(session.system_id),
                );
                let vp = uniforms.projection_matrix * uniforms.view_matrix;
                rasterizer::draw_orbit_lines(&session.bodies, session.selected_body, &vp, &mut framebuffer);
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
            tris_before_cull,
            tris_after_cull,
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
            "{} ({} objetos) · Tiempo: {:.1}s ({:.2}x{}) · Cámara [F]: {} · Ayuda [H]",
            session.system_id.display_name(),
            session.bodies.len(),
            session.sim_clock.time,
            session.sim_clock.scale,
            if session.sim_clock.paused { " PAUSA" } else { "" },
            if session.follow_mode { "Seguir nave" } else { "Libre" },
        );
        let mode_str = match session.nav_state.mode {
            nav::NavMode::Global => "Espacio libre".to_string(),
            nav::NavMode::Local(idx) => format!("Cerca de {}", session.bodies[idx].name),
        };
        let override_str = match session.nav_state.override_mode {
            nav::NavOverride::Auto => "Automático",
            nav::NavOverride::ForceLocal => "Fijo cerca",
            nav::NavOverride::ForceGlobal => "Fijo libre",
        };
        let alt_str = match session.nav_state.nearest {
            Some(n) => format!(
                "{:.2} ({})",
                n.altitude.max(0.0),
                session.bodies[n.body_index].name
            ),
            None => "--".to_string(),
        };
        let lod_mode_str = match session.forced_lod {
            None => "Automático".to_string(),
            Some(l) => format!("Fijo {l} de 5"),
        };
        let body_lod_summary: Vec<String> = session
            .bodies
            .iter()
            .enumerate()
            .map(|(i, b)| {
                let lvl = session.body_lods[i];
                let tris = icosphere_lods[lvl].len() / 3;
                let short: String = b.name.chars().take(3).collect();
                format!("{short}:{lvl}/5({tris})")
            })
            .collect();
        let selected_summary = if let Some(sel_body) = session.bodies.get(session.selected_body) {
            let sel_spec = shaders::live_param(sel_body.shader);
            format!(
                "Elegido [Tab]: {} · Tamaño [9/0]: {:.2} · {} [4/5]: {:.2}",
                sel_body.name,
                sel_body.radius,
                sel_spec.name,
                session.body_shader_params[session.selected_body]
            )
        } else {
            "Sin objetos".to_string()
        };
        let avg = perf_stats.average();
        let fps = if avg.total_ms > 0.0 {
            1000.0 / avg.total_ms
        } else {
            0.0
        };
        let win_w = window.get_screen_width().max(1) as u32;
        let win_h = window.get_screen_height().max(1) as u32;
        overlay_buf.resize(win_w, win_h);
        let mode_full = format!("{} ({})", mode_str, override_str);
        let lod_full = format!(
            "Nivel de detalle [Z]: {} · {}",
            lod_mode_str,
            body_lod_summary.join(" ")
        );
        let ship_lod_str = format!(
            "Nave: nivel {} de 3 ({} triángulos)",
            session.ship_lod + 1,
            ship_vertex_arrays[session.ship_lod].len() / 3
        );
        let ship_fwd = session.ship.forward;
        let minimap_max_r = session
            .bodies
            .iter()
            .map(|b| b.orbit_radius)
            .fold(10.0_f32, f32::max);
        let minimap_bodies: Vec<hud::MinimapBody> = session
            .bodies
            .iter()
            .enumerate()
            .map(|(i, b)| {
                let p = body_position_at(b, session.sim_clock.time);
                let col = match b.shader {
                    Shader::Star | Shader::PhoenixSuns => Color::new(255, 205, 80, 255),
                    Shader::Rocky => Color::new(85, 210, 165, 255),
                    Shader::GasGiant => Color::new(150, 210, 110, 255),
                    Shader::Ardens => Color::new(255, 110, 65, 255),
                    Shader::Abismo => Color::new(70, 175, 235, 255),
                    Shader::Basquia => Color::new(240, 200, 60, 255),
                    Shader::Vestigio => Color::new(200, 165, 120, 255),
                    Shader::Mural => Color::new(210, 110, 95, 255),
                    Shader::Fundido => Color::new(245, 145, 90, 255),
                    Shader::Meteor => Color::new(90, 225, 200, 255),
                    Shader::Triforce => Color::new(255, 220, 75, 255),
                    Shader::Spurs => Color::new(120, 155, 235, 255),
                    _ => Color::new(205, 215, 235, 255),
                };
                hud::MinimapBody {
                    x: p.x,
                    z: p.z,
                    orbit_radius: b.orbit_radius,
                    color: col,
                    is_star: b.orbit_radius <= 0.0,
                    is_selected: i == session.selected_body,
                }
            })
            .collect();
        let hud_snap = hud::HudSnapshot {
            title: &label,
            fps,
            total_ms: avg.total_ms,
            vertex_ms: avg.vertex_ms,
            raster_ms: avg.raster_ms,
            shading_ms: avg.shading_ms,
            present_ms: avg.present_ms,
            internal_width: framebuffer.width,
            internal_height: framebuffer.height,
            threads,
            tris_before_cull,
            tris_after_cull,
            frustum_culling,
            lod_summary: &lod_full,
            ship_lod_summary: &ship_lod_str,
            mode_summary: &mode_full,
            gravity_enabled: session.gravity_enabled,
            gravity_mag: session.gravity_mag,
            speed: session.ship.velocity.length(),
            altitude_summary: &alt_str,
            time_scale: session.sim_clock.scale,
            paused: session.sim_clock.paused,
            surface_seed: session.surface_seed,
            instances_drawn,
            instances_total,
            instance_culling,
            selected_summary: &selected_summary,
            collision_body: collision_body.map(|idx| session.bodies[idx].name),
            show_perf: perf_stats.enabled,
            show_help,
            show_hud,
            minimap: Some(hud::MinimapSnapshot {
                bodies: &minimap_bodies,
                ship_x: session.ship.position.x,
                ship_z: session.ship.position.z,
                ship_dir_x: ship_fwd.x,
                ship_dir_z: ship_fwd.z,
                max_radius: minimap_max_r,
            }),
        };
        hud::render_hud(&mut overlay_buf, &hud_snap);
        let t_pres = std::time::Instant::now();
        framebuffer.draw_with_overlay(&mut window, &thread, &overlay_buf);
        sample.present_ms = t_pres.elapsed().as_secs_f64() * 1000.0;
        sample.total_ms = frame_start.elapsed().as_secs_f64() * 1000.0;
        perf_stats.push(sample);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn favorites_flight_menu_transition_and_ten_session_cycles_drop_cleanly() {
        let mut ctrl = app::AppController::new();
        let fav_ready = showcase::is_favorites_system_ready(showcase::SHOWCASE_ENTRIES);
        assert!(fav_ready);
        ctrl.move_menu_down();
        assert_eq!(
            ctrl.activate_menu(fav_ready),
            app::TransitionOutcome::ChangeState(app::AppState::Flight(app::SystemId::Favoritos))
        );
        assert_eq!(
            ctrl.on_escape(),
            app::TransitionOutcome::ChangeState(app::AppState::Menu)
        );

        let noise = Noise::new(7);
        for _ in 0..10 {
            let mut fav_session = FlightSession::new(app::SystemId::Favoritos, &noise);
            assert_eq!(fav_session.bodies.len(), showcase::favorites_ready_entries().len());
            assert!(fav_session.proc_caches.iter().all(|c| c.is_some()));
            if let Some(Some(cache)) = fav_session.proc_caches.first_mut() {
                assert!(!cache.get_or_build(2).is_empty());
            }
            drop(fav_session);

            let real_session = FlightSession::new(app::SystemId::Realista, &noise);
            assert_eq!(real_session.bodies.len(), 9);
            drop(real_session);
        }
    }
}
