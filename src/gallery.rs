#![allow(dead_code)]
//! Visor de trofeos (galería): cuerpo aislado centrado en el origen con cámara
//! orbital propia, luz fija, rotación automática, zoom y control de LOD por presupuesto.

use std::sync::Arc;

use raylib::prelude::Vector3;

use crate::camera::Camera;
use crate::framebuffer::Framebuffer;
use crate::icosphere::{DeformedIcosphereCache, MAX_ICOSPHERE_LEVEL};
use crate::math::Mat4;
use crate::mesh_builder::LodMeshCache;
use crate::noise::Noise;
use crate::primitive::append_screen_triangles;
use crate::rasterizer::{
    TriMaterial, TriangleMode, rasterize_visibility_tiled, shade_visibility_tiled_materials,
};
use crate::scene::Body;
use crate::shaders::{self, Shader};
use crate::showcase::{
    SHOWCASE_ENTRIES, Showcase, body_for_showcase, mesh_builder_for_showcase, ready_indices,
};
use crate::surface_map::BakedMaps;
use crate::{RenderBuffers, Uniforms};

pub const GALLERY_FRAME_BUDGET_MS: f64 = 25.0;
pub const GALLERY_MIN_LOD: usize = 3;

pub struct GallerySelector {
    pub ready: Vec<usize>,
    pub bodies: Vec<Body>,
    pub cursor: usize,
    pub shader_params: Vec<f32>,
}

impl GallerySelector {
    pub fn new() -> Self {
        let ready = ready_indices();
        let bodies: Vec<Body> = ready
            .iter()
            .filter_map(|&idx| body_for_showcase(&SHOWCASE_ENTRIES[idx]))
            .collect();
        let shader_params = bodies
            .iter()
            .map(|b| shaders::live_param(b.shader).default)
            .collect();
        Self {
            ready,
            bodies,
            cursor: 0,
            shader_params,
        }
    }

    pub fn current_showcase_idx(&self) -> usize {
        self.ready[self.cursor]
    }

    pub fn current_showcase(&self) -> &'static Showcase {
        &SHOWCASE_ENTRIES[self.current_showcase_idx()]
    }

    pub fn current_body_idx(&self) -> usize {
        self.cursor
    }

    pub fn current_body(&self) -> &Body {
        &self.bodies[self.cursor]
    }

    pub fn select_next(&mut self) {
        if !self.ready.is_empty() {
            self.cursor = (self.cursor + 1) % self.ready.len();
        }
    }

    pub fn select_prev(&mut self) {
        if !self.ready.is_empty() {
            self.cursor = if self.cursor == 0 {
                self.ready.len() - 1
            } else {
                self.cursor - 1
            };
        }
    }

    pub fn adjust_param(&mut self, body: &Body, direction: f32) {
        let spec = shaders::live_param(body.shader);
        let idx = self.current_body_idx();
        self.shader_params[idx] =
            (self.shader_params[idx] + spec.step * direction).clamp(spec.min, spec.max);
    }

    pub fn current_param(&self) -> f32 {
        self.shader_params[self.current_body_idx()]
    }
}

pub struct GalleryViewer {
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
    pub target: Vector3,
    pub spin: f32,
    pub lod_level: usize,
    over_budget_frames: u8,
    cache: Option<(usize, DeformedIcosphereCache)>,
    proc_cache: Option<(usize, LodMeshCache)>,
}

impl GalleryViewer {
    pub fn new(body_radius: f32) -> Self {
        let mut viewer = Self {
            yaw: 0.0,
            pitch: 0.25,
            distance: 4.0,
            target: Vector3::zero(),
            spin: 0.0,
            lod_level: MAX_ICOSPHERE_LEVEL,
            over_budget_frames: 0,
            cache: None,
            proc_cache: None,
        };
        viewer.reset_view(body_radius);
        viewer
    }

    pub fn reset_view(&mut self, body_radius: f32) {
        self.yaw = 0.0;
        self.pitch = 0.25;
        self.distance = (body_radius * 3.2).clamp(2.2, 18.0);
        self.target = Vector3::zero();
        self.spin = 0.0;
    }

    pub fn reset_view_for(&mut self, body_radius: f32, showcase_id: &str) {
        if showcase_id == "meteoro_ff7" {
            // Encuadre de referencia: cabeza abajo a la izquierda, cola hacia arriba a la derecha.
            self.yaw = 0.0;
            self.pitch = 0.04;
            self.distance = 6.6;
            self.target = Vector3::new(1.55, 0.85, 0.0);
            self.spin = 0.0;
        } else if showcase_id == "spurs_crest" {
            self.yaw = 0.0;
            self.pitch = 0.05;
            self.distance = 7.4;
            self.target = Vector3::new(0.0, 1.35, 0.0);
            self.spin = 0.0;
        } else if showcase_id == "phoenix_suns" {
            self.yaw = 0.0;
            self.pitch = 0.04;
            self.distance = 8.4;
            self.target = Vector3::new(-0.55, -0.35, 0.0);
            self.spin = 0.0;
        } else if showcase_id == "raqueta_tenis" {
            self.yaw = 0.0;
            self.pitch = 0.0;
            self.distance = 2.9;
            self.target = Vector3::zero();
            self.spin = 0.0;
        } else {
            self.reset_view(body_radius);
        }
    }

    pub fn apply_orbit_delta(&mut self, dyaw: f32, dpitch: f32) {
        self.yaw += dyaw;
        self.pitch = (self.pitch + dpitch).clamp(-1.45, 1.45);
    }

    pub fn apply_zoom(&mut self, wheel_delta: f32, body_radius: f32) {
        let min_d = (body_radius * 1.55).max(1.4);
        let max_d = (body_radius * 8.0).max(8.0);
        self.distance = (self.distance - wheel_delta * 0.45).clamp(min_d, max_d);
    }

    pub fn advance(&mut self, dt: f32, body: &Body, shader_param: f32) {
        if matches!(body.shader, Shader::Meteor) {
            return;
        }
        let base_rate = if body.rotation_period.abs() > 1e-3 {
            std::f32::consts::TAU / body.rotation_period.max(18.0)
        } else {
            0.25
        };
        let mult = if matches!(body.shader, Shader::Chunky | Shader::PhoenixSuns | Shader::Spurs) {
            shader_param
        } else {
            1.0
        };
        self.spin += dt * base_rate * mult;
    }

    pub fn record_frame_time(&mut self, frame_ms: f64) {
        if frame_ms > GALLERY_FRAME_BUDGET_MS {
            self.over_budget_frames = self.over_budget_frames.saturating_add(1);
            if self.over_budget_frames >= 3 && self.lod_level > GALLERY_MIN_LOD {
                self.lod_level -= 1;
                self.over_budget_frames = 0;
            }
        } else {
            self.over_budget_frames = 0;
        }
    }

    pub fn camera(&self) -> Camera {
        let cam_pos = self.target
            + Vector3::new(
                self.distance * self.pitch.cos() * self.yaw.sin(),
                self.distance * self.pitch.sin(),
                self.distance * self.pitch.cos() * self.yaw.cos(),
            );
        Camera::new(cam_pos, self.target, 45.0_f32.to_radians(), 0.1, 120.0)
    }

    pub fn active_mesh<'a>(
        &'a mut self,
        body_index: usize,
        showcase_id: &str,
        body: &Body,
        noise: &Noise,
    ) -> &'a [crate::vertex::Vertex] {
        if let Some(builder) = mesh_builder_for_showcase(showcase_id) {
            let needs_new = match &self.proc_cache {
                Some((idx, _)) => *idx != body_index,
                None => true,
            };
            if needs_new {
                self.proc_cache = Some((body_index, LodMeshCache::new(builder)));
            }
            let proc_lod = self.lod_level.saturating_sub(3).min(2);
            let (_, cache) = self.proc_cache.as_mut().expect("proc cache inicializada");
            return cache.get_or_build(proc_lod);
        }
        let needs_new = match &self.cache {
            Some((idx, _)) => *idx != body_index,
            None => true,
        };
        if needs_new {
            self.cache = Some((
                body_index,
                DeformedIcosphereCache::new_for_shader(
                    body.shader,
                    body.surface.deformation_amplitude,
                    body.surface.noise_frequency,
                ),
            ));
        }
        let (_, cache) = self.cache.as_mut().expect("cache inicializada");
        cache.get_or_build(self.lod_level, noise)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn render_body(
        &mut self,
        body_index: usize,
        showcase_id: &str,
        body: &Body,
        custom_light: Option<Vector3>,
        shader_param: f32,
        time: f32,
        noise: &Noise,
        baked: &Arc<BakedMaps>,
        framebuffer: &mut Framebuffer,
        render_buffers: &mut RenderBuffers,
        mode: TriangleMode,
        threads: usize,
    ) -> usize {
        let cam = self.camera();
        let aspect = framebuffer.width as f32 / framebuffer.height.max(1) as f32;
        let light_dir = custom_light
            .unwrap_or_else(|| Vector3::new(-0.65, 0.80, 1.0))
            .normalize();
        let model_matrix = if matches!(body.shader, Shader::PhoenixSuns) {
            let osc = (time * std::f32::consts::TAU / 12.0).sin() * std::f32::consts::FRAC_PI_6; // ±30°
            Mat4::rotation_y(osc)
                * Mat4::scale(Vector3::new(body.radius, body.radius, body.radius))
        } else if matches!(body.shader, Shader::Spurs) {
            let osc = (self.spin * 1.4).sin() * 0.6109; // ±35°
            Mat4::rotation_y(osc)
                * Mat4::scale(Vector3::new(body.radius, body.radius, body.radius))
        } else if body.name == "Raqueta de tenis" {
            let rot = self.spin * (std::f32::consts::TAU / 12.0); // periodo ≈ 16 s (spin=0.75 rad/s)
            Mat4::rotation_y(rot)
                * Mat4::scale(Vector3::new(body.radius, body.radius, body.radius))
        } else {
            Mat4::rotation_z(body.axial_tilt)
                * Mat4::rotation_y(self.spin)
                * Mat4::scale(Vector3::new(body.radius, body.radius, body.radius))
        };

        let uniforms = Uniforms {
            model_matrix,
            view_matrix: cam.view_matrix(),
            projection_matrix: cam.projection_matrix(aspect),
            time,
            light_dir,
            camera_pos: cam.position,
            debug_noise: false,
            noise_frequency: body.surface.noise_frequency,
            rocky_debug_view: 3,
            normal_mapping: true,
            debug_normals: false,
            viewport_width: framebuffer.width as i32,
            viewport_height: framebuffer.height as i32,
            noise: noise.clone(),
            baked: Some(Arc::clone(baked)),
            shader_param,
        };

        let mesh = self.active_mesh(body_index, showcase_id, body, noise);
        let tri_count = mesh.len() / 3;

        framebuffer.clear();
        render_buffers.screen_triangles.clear();
        render_buffers.tri_materials.clear();
        append_screen_triangles(
            mesh,
            &uniforms,
            &mut render_buffers.clip_vertices,
            &mut render_buffers.screen_triangles,
        );
        render_buffers.tri_materials.resize(
            render_buffers.screen_triangles.len(),
            TriMaterial {
                shader: body.shader,
                light_dir,
                shader_param,
            },
        );

        match mode {
            TriangleMode::FilledBarycentric => {
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
            TriangleMode::Wireframe => {
                crate::rasterizer::render_wireframe(
                    &render_buffers.screen_triangles,
                    framebuffer,
                    body.shader,
                    &uniforms,
                );
            }
        }
        tri_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::showcase::ShowcaseStatus;

    #[test]
    fn gallery_viewer_orbit_zoom_and_reset() {
        let mut viewer = GalleryViewer::new(1.25);
        let initial_dist = viewer.distance;
        viewer.apply_orbit_delta(0.5, 2.0);
        assert!((viewer.yaw - 0.5).abs() < 1e-5);
        assert!(viewer.pitch <= 1.45);

        viewer.apply_zoom(20.0, 1.25);
        assert!(viewer.distance >= 1.25 * 1.55 - 1e-4);

        viewer.reset_view(1.25);
        assert_eq!(viewer.yaw, 0.0);
        assert!((viewer.distance - initial_dist).abs() < 1e-5);
    }

    #[test]
    fn gallery_selector_only_opens_ready_entries_and_clamps_live_param() {
        let mut sel = GallerySelector::new();
        assert!(sel.ready.len() >= 10);
        for _ in 0..16 {
            assert_eq!(sel.current_showcase().estado, ShowcaseStatus::Ready);
            sel.select_next();
        }
        sel.select_prev();
        assert_eq!(sel.current_showcase().estado, ShowcaseStatus::Ready);

        let body = *sel.current_body();
        let spec = shaders::live_param(body.shader);
        for _ in 0..20 {
            sel.adjust_param(&body, 1.0);
        }
        assert!(sel.current_param() <= spec.max + 1e-5);
        for _ in 0..40 {
            sel.adjust_param(&body, -1.0);
        }
        assert!(sel.current_param() >= spec.min - 1e-5);
    }

    #[test]
    fn gallery_viewer_steps_down_lod_when_over_budget() {
        let mut viewer = GalleryViewer::new(1.0);
        assert_eq!(viewer.lod_level, MAX_ICOSPHERE_LEVEL);
        for _ in 0..3 {
            viewer.record_frame_time(GALLERY_FRAME_BUDGET_MS + 5.0);
        }
        assert_eq!(viewer.lod_level, MAX_ICOSPHERE_LEVEL - 1);
    }
}
