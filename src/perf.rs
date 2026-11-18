//! Medición de rendimiento por etapa del frame y superposición F3.
//!
//! Registra los tiempos en milisegundos de cada etapa (vértices, rasterización,
//! sombreado, escritura al framebuffer con z-test y presentación) junto con
//! contadores de fragmentos, llamadas al shader y píxeles visibles.
//! Mantiene un promedio móvil de los últimos 30 cuadros para estabilizar la
//! lectura en vivo sin afectar el flujo de renderizado.

use std::collections::VecDeque;

const WINDOW_FRAMES: usize = 30;

#[derive(Clone, Copy, Debug, Default)]
pub struct FrameSample {
    pub vertex_ms: f64,
    pub raster_ms: f64,
    pub shading_ms: f64,
    pub fb_ms: f64,
    pub present_ms: f64,
    pub total_ms: f64,
    pub fragments: usize,
    pub shader_calls: usize,
    pub visible_pixels: usize,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug)]
pub struct PerfStats {
    pub enabled: bool,
    samples: VecDeque<FrameSample>,
}

impl PerfStats {
    pub fn new() -> Self {
        Self {
            enabled: false,
            samples: VecDeque::with_capacity(WINDOW_FRAMES),
        }
    }

    pub fn push(&mut self, sample: FrameSample) {
        if self.samples.len() == WINDOW_FRAMES {
            self.samples.pop_front();
        }
        self.samples.push_back(sample);
    }

    pub fn average(&self) -> FrameSample {
        let count = self.samples.len();
        if count == 0 {
            return FrameSample::default();
        }
        let inv = 1.0 / count as f64;
        let mut sum = FrameSample::default();
        for s in &self.samples {
            sum.vertex_ms += s.vertex_ms;
            sum.raster_ms += s.raster_ms;
            sum.shading_ms += s.shading_ms;
            sum.fb_ms += s.fb_ms;
            sum.present_ms += s.present_ms;
            sum.total_ms += s.total_ms;
            sum.fragments += s.fragments;
            sum.shader_calls += s.shader_calls;
            sum.visible_pixels += s.visible_pixels;
        }
        FrameSample {
            vertex_ms: sum.vertex_ms * inv,
            raster_ms: sum.raster_ms * inv,
            shading_ms: sum.shading_ms * inv,
            fb_ms: sum.fb_ms * inv,
            present_ms: sum.present_ms * inv,
            total_ms: sum.total_ms * inv,
            fragments: sum.fragments / count,
            shader_calls: sum.shader_calls / count,
            visible_pixels: sum.visible_pixels / count,
            width: 0,
            height: 0,
        }
    }

    pub fn overlay_lines(&self) -> Vec<String> {
        let avg = self.average();
        let fps = if avg.total_ms > 0.0 {
            1000.0 / avg.total_ms
        } else {
            0.0
        };
        let last = self.samples.back().copied().unwrap_or_default();
        vec![
            format!(
                "FPS: {fps:.1} ({:.2} ms) | Res: {}x{}",
                avg.total_ms, last.width, last.height
            ),
            format!(
                "Vtx: {:.2} | Rast: {:.2} | Shade: {:.2} ms",
                avg.vertex_ms, avg.raster_ms, avg.shading_ms
            ),
            format!(
                "FB+Z: {:.2} | Pres: {:.2} ms",
                avg.fb_ms, avg.present_ms
            ),
            format!(
                "Frags: {} | Shader: {} | Visibles: {}",
                avg.fragments, avg.shader_calls, avg.visible_pixels
            ),
        ]
    }
}
