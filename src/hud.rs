#![allow(dead_code)]
//! Fuente de mapa de bits propia (5×7) y búfer RGBA de superposición (HUD)
//! a la resolución de la ventana.

use raylib::prelude::*;

pub const GLYPH_WIDTH: usize = 5;
pub const GLYPH_HEIGHT: usize = 7;

/// Devuelve las 7 filas (5 bits LSB por fila, MSB a la izquierda: bit 4..bit 0)
/// para ASCII 32–126 más los caracteres españoles `á é í ó ú ñ ¿ ¡` (y mayúsculas).
pub fn glyph_5x7(ch: char) -> Option<[u8; 7]> {
    let rows = match ch {
        ' ' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
        '!' => [0x04, 0x04, 0x04, 0x04, 0x04, 0x00, 0x04],
        '"' => [0x0A, 0x0A, 0x0A, 0x00, 0x00, 0x00, 0x00],
        '#' => [0x0A, 0x0A, 0x1F, 0x0A, 0x1F, 0x0A, 0x0A],
        '$' => [0x04, 0x0F, 0x14, 0x0E, 0x05, 0x1E, 0x04],
        '%' => [0x18, 0x19, 0x02, 0x04, 0x08, 0x13, 0x03],
        '&' => [0x0C, 0x12, 0x14, 0x08, 0x15, 0x12, 0x0D],
        '\'' => [0x04, 0x04, 0x08, 0x00, 0x00, 0x00, 0x00],
        '(' => [0x02, 0x04, 0x08, 0x08, 0x08, 0x04, 0x02],
        ')' => [0x08, 0x04, 0x02, 0x02, 0x02, 0x04, 0x08],
        '*' => [0x00, 0x04, 0x15, 0x0E, 0x15, 0x04, 0x00],
        '+' => [0x00, 0x04, 0x04, 0x1F, 0x04, 0x04, 0x00],
        ',' => [0x00, 0x00, 0x00, 0x00, 0x04, 0x04, 0x08],
        '-' => [0x00, 0x00, 0x00, 0x0E, 0x00, 0x00, 0x00],
        '.' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x0C, 0x0C],
        '/' => [0x01, 0x02, 0x02, 0x04, 0x08, 0x08, 0x10],
        '0' => [0x0E, 0x11, 0x13, 0x15, 0x19, 0x11, 0x0E],
        '1' => [0x04, 0x0C, 0x04, 0x04, 0x04, 0x04, 0x0E],
        '2' => [0x0E, 0x11, 0x01, 0x06, 0x08, 0x10, 0x1F],
        '3' => [0x1F, 0x02, 0x04, 0x06, 0x01, 0x11, 0x0E],
        '4' => [0x02, 0x06, 0x0A, 0x12, 0x1F, 0x02, 0x02],
        '5' => [0x1F, 0x10, 0x1E, 0x01, 0x01, 0x11, 0x0E],
        '6' => [0x06, 0x08, 0x10, 0x1E, 0x11, 0x11, 0x0E],
        '7' => [0x1F, 0x01, 0x02, 0x04, 0x08, 0x08, 0x08],
        '8' => [0x0E, 0x11, 0x11, 0x0E, 0x11, 0x11, 0x0E],
        '9' => [0x0E, 0x11, 0x11, 0x0F, 0x01, 0x02, 0x0C],
        ':' => [0x00, 0x0C, 0x0C, 0x00, 0x0C, 0x0C, 0x00],
        ';' => [0x00, 0x0C, 0x0C, 0x00, 0x0C, 0x04, 0x08],
        '<' => [0x02, 0x04, 0x08, 0x10, 0x08, 0x04, 0x02],
        '=' => [0x00, 0x00, 0x1F, 0x00, 0x1F, 0x00, 0x00],
        '>' => [0x08, 0x04, 0x02, 0x01, 0x02, 0x04, 0x08],
        '?' => [0x0E, 0x11, 0x01, 0x06, 0x04, 0x00, 0x04],
        '@' => [0x0E, 0x11, 0x17, 0x15, 0x17, 0x10, 0x0E],
        'A' => [0x0E, 0x11, 0x11, 0x1F, 0x11, 0x11, 0x11],
        'B' => [0x1E, 0x11, 0x11, 0x1E, 0x11, 0x11, 0x1E],
        'C' => [0x0E, 0x11, 0x10, 0x10, 0x10, 0x11, 0x0E],
        'D' => [0x1E, 0x11, 0x11, 0x11, 0x11, 0x11, 0x1E],
        'E' => [0x1F, 0x10, 0x10, 0x1E, 0x10, 0x10, 0x1F],
        'F' => [0x1F, 0x10, 0x10, 0x1E, 0x10, 0x10, 0x10],
        'G' => [0x0E, 0x11, 0x10, 0x17, 0x11, 0x11, 0x0F],
        'H' => [0x11, 0x11, 0x11, 0x1F, 0x11, 0x11, 0x11],
        'I' => [0x0E, 0x04, 0x04, 0x04, 0x04, 0x04, 0x0E],
        'J' => [0x07, 0x02, 0x02, 0x02, 0x02, 0x12, 0x0C],
        'K' => [0x11, 0x12, 0x14, 0x18, 0x14, 0x12, 0x11],
        'L' => [0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x1F],
        'M' => [0x11, 0x1B, 0x15, 0x15, 0x11, 0x11, 0x11],
        'N' => [0x11, 0x11, 0x19, 0x15, 0x13, 0x11, 0x11],
        'O' => [0x0E, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0E],
        'P' => [0x1E, 0x11, 0x11, 0x1E, 0x10, 0x10, 0x10],
        'Q' => [0x0E, 0x11, 0x11, 0x11, 0x15, 0x12, 0x0D],
        'R' => [0x1E, 0x11, 0x11, 0x1E, 0x14, 0x12, 0x11],
        'S' => [0x0F, 0x10, 0x10, 0x0E, 0x01, 0x01, 0x1E],
        'T' => [0x1F, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04],
        'U' => [0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0E],
        'V' => [0x11, 0x11, 0x11, 0x11, 0x11, 0x0A, 0x04],
        'W' => [0x11, 0x11, 0x11, 0x15, 0x15, 0x1B, 0x11],
        'X' => [0x11, 0x11, 0x0A, 0x04, 0x0A, 0x11, 0x11],
        'Y' => [0x11, 0x11, 0x0A, 0x04, 0x04, 0x04, 0x04],
        'Z' => [0x1F, 0x01, 0x02, 0x04, 0x08, 0x10, 0x1F],
        '[' => [0x0E, 0x08, 0x08, 0x08, 0x08, 0x08, 0x0E],
        '\\' => [0x10, 0x08, 0x08, 0x04, 0x02, 0x02, 0x01],
        ']' => [0x0E, 0x02, 0x02, 0x02, 0x02, 0x02, 0x0E],
        '^' => [0x04, 0x0A, 0x11, 0x00, 0x00, 0x00, 0x00],
        '_' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x1F],
        '`' => [0x08, 0x04, 0x02, 0x00, 0x00, 0x00, 0x00],
        'a' => [0x00, 0x00, 0x0E, 0x01, 0x0F, 0x11, 0x0F],
        'b' => [0x10, 0x10, 0x1E, 0x11, 0x11, 0x11, 0x1E],
        'c' => [0x00, 0x00, 0x0E, 0x10, 0x10, 0x11, 0x0E],
        'd' => [0x01, 0x01, 0x0F, 0x11, 0x11, 0x11, 0x0F],
        'e' => [0x00, 0x00, 0x0E, 0x11, 0x1F, 0x10, 0x0E],
        'f' => [0x06, 0x08, 0x1E, 0x08, 0x08, 0x08, 0x08],
        'g' => [0x00, 0x0E, 0x11, 0x11, 0x0F, 0x01, 0x1E],
        'h' => [0x10, 0x10, 0x1E, 0x11, 0x11, 0x11, 0x11],
        'i' => [0x04, 0x00, 0x0C, 0x04, 0x04, 0x04, 0x0E],
        'j' => [0x02, 0x00, 0x06, 0x02, 0x02, 0x12, 0x0C],
        'k' => [0x10, 0x10, 0x12, 0x14, 0x18, 0x14, 0x12],
        'l' => [0x08, 0x08, 0x08, 0x08, 0x08, 0x08, 0x06],
        'm' => [0x00, 0x00, 0x1A, 0x15, 0x15, 0x11, 0x11],
        'n' => [0x00, 0x00, 0x1E, 0x11, 0x11, 0x11, 0x11],
        'o' => [0x00, 0x00, 0x0E, 0x11, 0x11, 0x11, 0x0E],
        'p' => [0x00, 0x00, 0x1E, 0x11, 0x1E, 0x10, 0x10],
        'q' => [0x00, 0x0E, 0x12, 0x12, 0x0E, 0x02, 0x03],
        'r' => [0x00, 0x00, 0x16, 0x19, 0x10, 0x10, 0x10],
        's' => [0x00, 0x00, 0x0F, 0x10, 0x0E, 0x01, 0x1E],
        't' => [0x08, 0x08, 0x1E, 0x08, 0x08, 0x09, 0x06],
        'u' => [0x00, 0x00, 0x11, 0x11, 0x11, 0x13, 0x0D],
        'v' => [0x00, 0x00, 0x11, 0x11, 0x11, 0x0A, 0x04],
        'w' => [0x00, 0x00, 0x11, 0x11, 0x15, 0x15, 0x0A],
        'x' => [0x00, 0x00, 0x11, 0x0A, 0x04, 0x0A, 0x11],
        'y' => [0x00, 0x11, 0x11, 0x0F, 0x01, 0x11, 0x0E],
        'z' => [0x00, 0x00, 0x1F, 0x02, 0x04, 0x08, 0x1F],
        '{' => [0x06, 0x08, 0x08, 0x10, 0x08, 0x08, 0x06],
        '|' => [0x04, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04],
        '}' => [0x0C, 0x02, 0x02, 0x01, 0x02, 0x02, 0x0C],
        '~' => [0x00, 0x00, 0x08, 0x15, 0x02, 0x00, 0x00],
        // Caracteres españoles extendidos
        'á' => [0x02, 0x04, 0x0E, 0x01, 0x0F, 0x11, 0x0F],
        'é' => [0x02, 0x04, 0x0E, 0x11, 0x1F, 0x10, 0x0E],
        'í' => [0x02, 0x04, 0x0C, 0x04, 0x04, 0x04, 0x0E],
        'ó' => [0x02, 0x04, 0x00, 0x0E, 0x11, 0x11, 0x0E],
        'ú' => [0x02, 0x04, 0x11, 0x11, 0x11, 0x13, 0x0D],
        'ü' => [0x0A, 0x00, 0x11, 0x11, 0x11, 0x13, 0x0D],
        'ñ' => [0x0D, 0x12, 0x1E, 0x11, 0x11, 0x11, 0x11],
        'Á' => [0x02, 0x04, 0x0E, 0x11, 0x1F, 0x11, 0x11],
        'É' => [0x02, 0x04, 0x1F, 0x10, 0x1E, 0x10, 0x1F],
        'Í' => [0x02, 0x04, 0x0E, 0x04, 0x04, 0x04, 0x0E],
        'Ó' => [0x02, 0x04, 0x0E, 0x11, 0x11, 0x11, 0x0E],
        'Ú' => [0x02, 0x04, 0x11, 0x11, 0x11, 0x11, 0x0E],
        'Ü' => [0x0A, 0x00, 0x11, 0x11, 0x11, 0x11, 0x0E],
        'Ñ' => [0x0D, 0x12, 0x11, 0x19, 0x15, 0x13, 0x11],
        '¿' => [0x04, 0x00, 0x04, 0x0C, 0x10, 0x11, 0x0E],
        '¡' => [0x04, 0x00, 0x04, 0x04, 0x04, 0x04, 0x04],
        '·' => [0x00, 0x00, 0x06, 0x06, 0x00, 0x00, 0x00],
        '—' => [0x00, 0x00, 0x00, 0x1F, 0x00, 0x00, 0x00],
        '…' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x15],
        _ => return None,
    };
    Some(rows)
}

/// Búfer de superposición RGBA a la resolución de la ventana.
pub struct OverlayBuffer {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

impl OverlayBuffer {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            pixels: vec![0; (width as usize) * (height as usize) * 4],
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if self.width == width && self.height == height {
            return;
        }
        self.width = width;
        self.height = height;
        self.pixels.resize((width as usize) * (height as usize) * 4, 0);
        self.clear();
    }

    pub fn clear(&mut self) {
        self.pixels.fill(0);
    }

    #[inline]
    pub fn blend_pixel(&mut self, x: i32, y: i32, color: Color) {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 || color.a == 0 {
            return;
        }
        let idx = (y as usize * self.width as usize + x as usize) * 4;
        if color.a == 255 {
            self.pixels[idx] = color.r;
            self.pixels[idx + 1] = color.g;
            self.pixels[idx + 2] = color.b;
            self.pixels[idx + 3] = 255;
            return;
        }
        let sa = color.a as u16;
        let inv = 255 - sa;
        let da = self.pixels[idx + 3] as u16;
        self.pixels[idx] = ((color.r as u16 * sa + self.pixels[idx] as u16 * inv) / 255) as u8;
        self.pixels[idx + 1] =
            ((color.g as u16 * sa + self.pixels[idx + 1] as u16 * inv) / 255) as u8;
        self.pixels[idx + 2] =
            ((color.b as u16 * sa + self.pixels[idx + 2] as u16 * inv) / 255) as u8;
        self.pixels[idx + 3] = (sa + (da * inv) / 255).min(255) as u8;
    }

    pub fn fill_rect(&mut self, x: i32, y: i32, w: i32, h: i32, color: Color) {
        if w <= 0 || h <= 0 {
            return;
        }
        let x0 = x.max(0);
        let y0 = y.max(0);
        let x1 = (x + w).min(self.width as i32);
        let y1 = (y + h).min(self.height as i32);
        for py in y0..y1 {
            for px in x0..x1 {
                self.blend_pixel(px, py, color);
            }
        }
    }

    pub fn stroke_rect(&mut self, x: i32, y: i32, w: i32, h: i32, color: Color) {
        if w <= 0 || h <= 0 {
            return;
        }
        self.fill_rect(x, y, w, 1, color);
        self.fill_rect(x, y + h - 1, w, 1, color);
        self.fill_rect(x, y, 1, h, color);
        self.fill_rect(x + w - 1, y, 1, h, color);
    }

    pub fn draw_char(&mut self, ch: char, x: i32, y: i32, scale: i32, color: Color) {
        let s = scale.max(1);
        let glyph = glyph_5x7(ch).unwrap_or_else(|| glyph_5x7('?').unwrap());
        for (row, &bits) in glyph.iter().enumerate() {
            for col in 0..GLYPH_WIDTH {
                if (bits & (1 << (GLYPH_WIDTH - 1 - col))) != 0 {
                    self.fill_rect(x + col as i32 * s, y + row as i32 * s, s, s, color);
                }
            }
        }
    }

    pub fn measure_text(text: &str, scale: i32) -> (i32, i32) {
        let s = scale.max(1);
        let chars = text.chars().count() as i32;
        if chars == 0 {
            return (0, GLYPH_HEIGHT as i32 * s);
        }
        let advance = (GLYPH_WIDTH as i32 + 1) * s;
        (chars * advance - s, GLYPH_HEIGHT as i32 * s)
    }

    pub fn draw_text(&mut self, text: &str, x: i32, y: i32, scale: i32, color: Color) {
        let s = scale.max(1);
        let advance = (GLYPH_WIDTH as i32 + 1) * s;
        let mut cx = x;
        for ch in text.chars() {
            if cx >= self.width as i32 {
                break;
            }
            if cx + GLYPH_WIDTH as i32 * s > 0 {
                self.draw_char(ch, cx, y, s, color);
            }
            cx += advance;
        }
    }

    pub fn text_fits_window(&self, text: &str, x: i32, y: i32, scale: i32) -> bool {
        let (tw, th) = Self::measure_text(text, scale);
        x >= 0 && y >= 0 && x + tw <= self.width as i32 && y + th <= self.height as i32
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ControlDoc {
    pub key: &'static str,
    pub action: &'static str,
}

/// Única tabla de verdad para el panel de ayuda (H) y la documentación.
pub const CONTROLS: &[ControlDoc] = &[
    ControlDoc { key: "W / S", action: "Acelerar hacia adelante o atrás" },
    ControlDoc { key: "Shift", action: "Turbo para ir más rápido" },
    ControlDoc { key: "Espacio", action: "Frenar poco a poco" },
    ControlDoc { key: "Ratón / Flechas", action: "Girar la vista (en cámara libre)" },
    ControlDoc { key: "Q / E", action: "Inclinar la nave a izquierda o derecha" },
    ControlDoc { key: "F / Retroceso", action: "Cambiar cámara (seguir / libre) · Recentrar" },
    ControlDoc { key: "L / G", action: "Control de vuelo (Auto/Cerca/Libre) · Gravedad" },
    ControlDoc { key: "Enter / - =", action: "Pausar el tiempo · Velocidad del tiempo" },
    ControlDoc { key: "Tab", action: "Elegir el siguiente objeto del sistema" },
    ControlDoc { key: "9 / 0", action: "Cambiar tamaño del objeto elegido" },
    ControlDoc { key: "4 / 5", action: "Ajustar el efecto visual del objeto elegido" },
    ControlDoc { key: ", . / [ ]", action: "Altura y detalle del relieve del planeta" },
    ControlDoc { key: "I / O", action: "Semilla (cambia el reparto aleatorio de objetos)" },
    ControlDoc { key: "Z / X / M", action: "Nivel de detalle · Descartar fuera de vista · Ver la malla (alambre)" },
    ControlDoc { key: "B / Y / /", action: "Relieve fino (normal map) · Ver relieve · Ocultar objetos lejanos" },
    ControlDoc { key: "F3 / R / T", action: "FPS (cuadros por segundo: mide la fluidez) · Calidad de imagen · Núcleos" },
    ControlDoc { key: "F4", action: "Ocultar o mostrar la interfaz de medición" },
    ControlDoc { key: "C / K / P", action: "Ocultar caras traseras · Recalcular texturas de superficie · Guardar foto" },
    ControlDoc { key: "A / D / R", action: "En galería: cambiar objeto · Reiniciar vista" },
    ControlDoc { key: "Esc", action: "Volver al menú principal (en el menú: salir)" },
    ControlDoc { key: "H", action: "Mostrar u ocultar este panel de ayuda" },
];

#[derive(Clone, Copy, Debug)]
pub struct MinimapBody {
    pub x: f32,
    pub z: f32,
    pub orbit_radius: f32,
    pub color: Color,
    pub is_star: bool,
    pub is_selected: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct MinimapSnapshot<'a> {
    pub bodies: &'a [MinimapBody],
    pub ship_x: f32,
    pub ship_z: f32,
    pub ship_dir_x: f32,
    pub ship_dir_z: f32,
    pub max_radius: f32,
}

pub struct HudSnapshot<'a> {
    pub title: &'a str,
    pub fps: f64,
    pub total_ms: f64,
    pub vertex_ms: f64,
    pub raster_ms: f64,
    pub shading_ms: f64,
    pub present_ms: f64,
    pub internal_width: u32,
    pub internal_height: u32,
    pub threads: usize,
    pub tris_before_cull: usize,
    pub tris_after_cull: usize,
    pub frustum_culling: bool,
    pub lod_summary: &'a str,
    pub ship_lod_summary: &'a str,
    pub mode_summary: &'a str,
    pub gravity_enabled: bool,
    pub gravity_mag: f32,
    pub speed: f32,
    pub altitude_summary: &'a str,
    pub time_scale: f32,
    pub paused: bool,
    pub surface_seed: u64,
    pub instances_drawn: usize,
    pub instances_total: usize,
    pub instance_culling: bool,
    pub selected_summary: &'a str,
    pub collision_body: Option<&'a str>,
    pub show_perf: bool,
    pub show_help: bool,
    pub show_hud: bool,
    pub minimap: Option<MinimapSnapshot<'a>>,
}

fn draw_overlay_line(overlay: &mut OverlayBuffer, x0: i32, y0: i32, x1: i32, y1: i32, color: Color) {
    let dx = (x1 - x0) as f32;
    let dy = (y1 - y0) as f32;
    let steps = dx.abs().max(dy.abs()).ceil() as i32;
    if steps <= 0 {
        overlay.blend_pixel(x0, y0, color);
        return;
    }
    for s in 0..=steps {
        let t = s as f32 / steps as f32;
        overlay.blend_pixel((x0 as f32 + dx * t).round() as i32, (y0 as f32 + dy * t).round() as i32, color);
    }
}

fn render_minimap(overlay: &mut OverlayBuffer, map: &MinimapSnapshot<'_>, top_y: i32) {
    let size = 148_i32.min(overlay.width as i32 / 3).max(96);
    let mx = overlay.width as i32 - size - 10;
    let my = top_y;
    overlay.fill_rect(mx, my, size, size, Color::new(6, 10, 18, 190));
    overlay.stroke_rect(mx, my, size, size, Color::new(80, 130, 185, 225));

    let cx = mx + size / 2;
    let cy = my + size / 2;
    let scale = (size as f32 * 0.43) / map.max_radius.max(1.0);
    let inner_min_x = mx + 3;
    let inner_max_x = mx + size - 4;
    let inner_min_y = my + 3;
    let inner_max_y = my + size - 4;

    // Anillos de órbita tenues
    let orbit_color = Color::new(85, 115, 150, 95);
    for b in map.bodies {
        if b.orbit_radius > 0.0 {
            let r_px = b.orbit_radius * scale;
            let segs = 48;
            for i in 0..segs {
                let a = std::f32::consts::TAU * (i as f32 / segs as f32);
                let px = cx + (r_px * a.cos()).round() as i32;
                let py = cy + (r_px * a.sin()).round() as i32;
                if px >= inner_min_x && px <= inner_max_x && py >= inner_min_y && py <= inner_max_y {
                    overlay.blend_pixel(px, py, orbit_color);
                }
            }
        }
    }

    // Planetas y estrella
    for b in map.bodies {
        let px = (cx + (b.x * scale).round() as i32).clamp(inner_min_x, inner_max_x);
        let py = (cy + (b.z * scale).round() as i32).clamp(inner_min_y, inner_max_y);
        let r = if b.is_star { 3 } else { 2 };
        overlay.fill_rect(px - r, py - r, r * 2 + 1, r * 2 + 1, b.color);
        if b.is_selected {
            overlay.stroke_rect(px - r - 2, py - r - 2, r * 2 + 5, r * 2 + 5, Color::new(130, 225, 255, 240));
        }
    }

    // Nave (flecha direccional brillante)
    let sx = (cx + (map.ship_x * scale).round() as i32).clamp(inner_min_x + 2, inner_max_x - 2);
    let sy = (cy + (map.ship_z * scale).round() as i32).clamp(inner_min_y + 2, inner_max_y - 2);
    let len = (map.ship_dir_x * map.ship_dir_x + map.ship_dir_z * map.ship_dir_z).sqrt();
    let (fx, fz) = if len > 1e-4 {
        (map.ship_dir_x / len, map.ship_dir_z / len)
    } else {
        (0.0, -1.0)
    };
    let rx = -fz;
    let rz = fx;
    let tip_x = sx + (fx * 6.0).round() as i32;
    let tip_y = sy + (fz * 6.0).round() as i32;
    let left_x = sx + (-fx * 4.0 + rx * 3.5).round() as i32;
    let left_y = sy + (-fz * 4.0 + rz * 3.5).round() as i32;
    let right_x = sx + (-fx * 4.0 - rx * 3.5).round() as i32;
    let right_y = sy + (-fz * 4.0 - rz * 3.5).round() as i32;
    let ship_col = Color::new(255, 255, 255, 255);
    draw_overlay_line(overlay, tip_x, tip_y, left_x, left_y, ship_col);
    draw_overlay_line(overlay, tip_x, tip_y, right_x, right_y, ship_col);
    draw_overlay_line(overlay, left_x, left_y, right_x, right_y, Color::new(130, 235, 255, 245));
    overlay.fill_rect(sx - 1, sy - 1, 3, 3, Color::new(130, 235, 255, 255));
}

pub fn render_hud(overlay: &mut OverlayBuffer, hud: &HudSnapshot<'_>) {
    overlay.clear();

    if let Some(ref map) = hud.minimap {
        let map_y = if hud.show_hud { 38 } else { 10 };
        render_minimap(overlay, map, map_y);
    }

    if !hud.show_hud {
        return;
    }

    // Barra superior de título.
    let bar_w = (overlay.width as i32 - 20).max(100);
    overlay.fill_rect(10, 8, bar_w, 24, Color::new(8, 12, 20, 195));
    overlay.stroke_rect(10, 8, bar_w, 24, Color::new(70, 110, 150, 220));
    let title_scale = if OverlayBuffer::measure_text(hud.title, 2).0 <= bar_w - 16 {
        2
    } else {
        1
    };
    let title_text = fit_text_with_ellipsis(hud.title, bar_w - 16, title_scale);
    let title_y = 8 + (24 - GLYPH_HEIGHT as i32 * title_scale) / 2;
    overlay.draw_text(&title_text, 18, title_y, title_scale, Color::WHITE);

    // Panel principal de telemetría y contadores (esquina superior izquierda).
    let raw_lines: Vec<(String, Color)> = vec![
        (
            format!(
                "FPS: {:5.1} ({:4.1} ms) · Imagen: {}x{} · Núcleos: {} · Tiempo: {:.2}x{}",
                hud.fps,
                hud.total_ms,
                hud.internal_width,
                hud.internal_height,
                hud.threads,
                hud.time_scale,
                if hud.paused { " (PAUSA)" } else { "" }
            ),
            Color::new(140, 245, 175, 255),
        ),
        (
            format!(
                "Vuelo: {} · Altura: {} · Velocidad: {:.2}",
                hud.mode_summary, hud.altitude_summary, hud.speed
            ),
            Color::new(210, 230, 255, 255),
        ),
        (
            format!(
                "Gravedad [G]: {} ({:.2}) · Semilla [I/O]: {} · Objetos: {}/{} ({})",
                if hud.gravity_enabled { "SÍ" } else { "NO" },
                hud.gravity_mag,
                hud.surface_seed,
                hud.instances_drawn,
                hud.instances_total,
                if hud.instance_culling { "ocultar lejanos" } else { "ver todos" },
            ),
            Color::new(210, 230, 255, 255),
        ),
        (
            format!(
                "Objetos descartados por no verse [X]: {} · {} -> {} triángulos · {}",
                if hud.frustum_culling { "SÍ" } else { "NO" },
                hud.tris_before_cull,
                hud.tris_after_cull,
                hud.ship_lod_summary,
            ),
            Color::new(255, 220, 140, 255),
        ),
        (
            hud.lod_summary.to_string(),
            Color::new(255, 220, 140, 255),
        ),
        (
            hud.selected_summary.to_string(),
            Color::new(150, 225, 255, 255),
        ),
    ];

    let mut all_raw = raw_lines;
    if hud.show_perf {
        all_raw.push((
            format!(
                "Tiempos (ms) — Vértices: {:.2} · Dibujo: {:.2} · Color: {:.2} · Pantalla: {:.2}",
                hud.vertex_ms, hud.raster_ms, hud.shading_ms, hud.present_ms
            ),
            Color::new(255, 195, 110, 255),
        ));
    }
    if let Some(body_name) = hud.collision_body {
        all_raw.push((
            format!("¡CONTACTO CON LA SUPERFICIE DE {}!", body_name),
            Color::new(255, 100, 100, 255),
        ));
    }

    let max_box_w = if hud.minimap.is_some() {
        (overlay.width as i32 - 180).max(260)
    } else {
        (overlay.width as i32 - 20).min(760)
    };
    let box_w = max_box_w;
    let mut lines: Vec<(String, Color)> = Vec::new();
    for (text, color) in all_raw {
        for wrapped in wrap_text(&text, box_w - 16, 1) {
            lines.push((wrapped, color));
        }
    }
    let max_box_h = (overlay.height as i32 - 54).max(32);
    let max_lines = ((max_box_h - 12) / 14).max(1) as usize;
    if lines.len() > max_lines {
        lines.truncate(max_lines);
        if let Some((last, _)) = lines.last_mut() {
            *last = fit_text_with_ellipsis(&format!("{last}…"), box_w - 16, 1);
        }
    }

    let box_h = 12 + lines.len() as i32 * 14;
    overlay.fill_rect(10, 38, box_w, box_h, Color::new(8, 12, 20, 185));
    overlay.stroke_rect(10, 38, box_w, box_h, Color::new(55, 85, 115, 200));
    for (i, (text, color)) in lines.iter().enumerate() {
        overlay.draw_text(text, 18, 44 + i as i32 * 14, 1, *color);
    }

    // Panel de ayuda (H) generado desde CONTROLS.
    if hud.show_help {
        let help_y = 44 + box_h + 6;
        let help_h = (26 + CONTROLS.len() as i32 * 13).min((overlay.height as i32 - help_y - 8).max(40));
        let help_w = (overlay.width as i32 - 20).min(700);
        overlay.fill_rect(10, help_y, help_w, help_h, Color::new(10, 14, 24, 215));
        overlay.stroke_rect(10, help_y, help_w, help_h, Color::new(110, 165, 220, 230));
        let help_title = fit_text_with_ellipsis("AYUDA Y CONTROLES (pulsa H para cerrar)", help_w - 16, 1);
        overlay.draw_text(
            &help_title,
            18,
            help_y + 6,
            1,
            Color::new(255, 230, 140, 255),
        );
        for (i, ctrl) in CONTROLS.iter().enumerate() {
            let row_y = help_y + 22 + i as i32 * 13;
            if row_y + GLYPH_HEIGHT as i32 > help_y + help_h - 4 {
                break;
            }
            let act = fit_text_with_ellipsis(ctrl.action, help_w - 142, 1);
            overlay.draw_text(ctrl.key, 18, row_y, 1, Color::new(140, 245, 175, 255));
            overlay.draw_text(&act, 134, row_y, 1, Color::WHITE);
        }
    }
}

pub fn render_menu_overlay(
    overlay: &mut OverlayBuffer,
    selected: usize,
    favorites_ready: bool,
    show_controls: bool,
) {
    overlay.clear();

    let w = overlay.width as i32;
    let h = overlay.height as i32;
    let box_w = 520.min(w - 40).max(260);
    let box_h = 270;
    let bx = (w - box_w) / 2;
    let by = (h - box_h) / 2;

    overlay.fill_rect(bx, by, box_w, box_h, Color::new(8, 12, 22, 210));
    overlay.stroke_rect(bx, by, box_w, box_h, Color::new(90, 145, 205, 235));

    let title = "SISTEMA SOLAR 3D";
    let (tw, _) = OverlayBuffer::measure_text(title, 3);
    overlay.draw_text(title, (w - tw) / 2, by + 18, 3, Color::new(255, 215, 130, 255));
    let subtitle = "Simulador espacial 3D hecho desde cero en Rust";
    let (sw, _) = OverlayBuffer::measure_text(subtitle, 1);
    overlay.draw_text(subtitle, (w - sw) / 2, by + 46, 1, Color::new(160, 185, 215, 255));

    for (i, entry) in crate::app::MENU_ENTRIES.iter().enumerate() {
        let ey = by + 74 + i as i32 * 32;
        let is_sel = i == selected;
        let disabled = matches!(entry, crate::app::MenuEntry::FavoritesSystem) && !favorites_ready;

        if is_sel {
            overlay.fill_rect(
                bx + 20,
                ey - 4,
                box_w - 40,
                24,
                if disabled {
                    Color::new(45, 45, 58, 200)
                } else {
                    Color::new(32, 66, 110, 225)
                },
            );
            overlay.stroke_rect(
                bx + 20,
                ey - 4,
                box_w - 40,
                24,
                if disabled {
                    Color::new(120, 120, 140, 220)
                } else {
                    Color::new(120, 200, 255, 255)
                },
            );
        }

        let label_color = if disabled {
            Color::new(135, 140, 150, 255)
        } else if is_sel {
            Color::WHITE
        } else {
            Color::new(205, 215, 230, 255)
        };
        let prefix = if is_sel { "> " } else { "  " };
        let line = format!("{}{}", prefix, entry.label());
        overlay.draw_text(&line, bx + 30, ey, 2, label_color);

        if disabled {
            let tag = "en construcción";
            let (tag_w, _) = OverlayBuffer::measure_text(tag, 1);
            overlay.draw_text(
                tag,
                bx + box_w - 28 - tag_w,
                ey + 4,
                1,
                Color::new(220, 175, 95, 255),
            );
        }
    }

    let hint = "W/S o Flechas: moverse · Enter: elegir · Esc: salir";
    let (hw, _) = OverlayBuffer::measure_text(hint, 1);
    overlay.draw_text(
        hint,
        (w - hw) / 2,
        by + box_h - 18,
        1,
        Color::new(150, 170, 195, 255),
    );

    if show_controls {
        let cw = (w - 40).min(700);
        let ch = (CONTROLS.len() as i32 * 13 + 56).min(h - 40);
        let cx = (w - cw) / 2;
        let cy = (h - ch) / 2;
        overlay.fill_rect(cx, cy, cw, ch, Color::new(6, 10, 18, 240));
        overlay.stroke_rect(cx, cy, cw, ch, Color::new(110, 175, 235, 245));
        overlay.draw_text("Controles", cx + 16, cy + 12, 2, Color::new(150, 220, 255, 255));
        for (i, c) in CONTROLS.iter().enumerate() {
            let row_y = cy + 36 + i as i32 * 13;
            if row_y + 12 >= cy + ch - 18 {
                break;
            }
            let act = fit_text_with_ellipsis(c.action, cw - 144, 1);
            overlay.draw_text(c.key, cx + 16, row_y, 1, Color::new(255, 220, 120, 255));
            overlay.draw_text(&act, cx + 132, row_y, 1, Color::new(225, 232, 240, 255));
        }
        overlay.draw_text(
            "Enter o Retroceso: volver",
            cx + 16,
            cy + ch - 16,
            1,
            Color::new(160, 185, 215, 255),
        );
    }
}

pub struct GalleryHudSnapshot<'a> {
    pub entry: &'a crate::showcase::Showcase,
    pub active_showcase_idx: usize,
    pub radius: f32,
    pub tris: usize,
    pub lod_level: usize,
    pub shader_name: &'a str,
    pub live_param_name: &'a str,
    pub live_param_value: f32,
    pub fps: f64,
}

/// Envuelve `text` por palabras para que ninguna línea supere `max_width_px` a la escala dada.
pub fn wrap_text(text: &str, max_width_px: i32, scale: i32) -> Vec<String> {
    let s = scale.max(1);
    let advance = (GLYPH_WIDTH as i32 + 1) * s;
    let char_w = GLYPH_WIDTH as i32 * s;
    let max_chars = if max_width_px < char_w {
        1
    } else {
        ((max_width_px - char_w) / advance + 1) as usize
    };
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        let word_len = word.chars().count();
        if current.is_empty() {
            if word_len <= max_chars {
                current.push_str(word);
            } else {
                let mut chunk = String::new();
                for ch in word.chars() {
                    if chunk.chars().count() >= max_chars {
                        lines.push(chunk);
                        chunk = String::new();
                    }
                    chunk.push(ch);
                }
                current = chunk;
            }
        } else if current.chars().count() + 1 + word_len <= max_chars {
            current.push(' ');
            current.push_str(word);
        } else {
            lines.push(current);
            current = String::new();
            if word_len <= max_chars {
                current.push_str(word);
            } else {
                let mut chunk = String::new();
                for ch in word.chars() {
                    if chunk.chars().count() >= max_chars {
                        lines.push(chunk);
                        chunk = String::new();
                    }
                    chunk.push(ch);
                }
                current = chunk;
            }
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

/// Acorta `text` con `"…"` al final si supera `max_width_px` a la escala dada.
pub fn fit_text_with_ellipsis(text: &str, max_width_px: i32, scale: i32) -> String {
    if OverlayBuffer::measure_text(text, scale).0 <= max_width_px {
        return text.to_string();
    }
    let mut out = String::new();
    for ch in text.chars() {
        let mut candidate = out.clone();
        candidate.push(ch);
        candidate.push('…');
        if OverlayBuffer::measure_text(&candidate, scale).0 > max_width_px {
            break;
        }
        out.push(ch);
    }
    out.push('…');
    out
}

fn wrap_words(text: &str, max_chars: usize) -> Vec<String> {
    let max_px = (max_chars.max(1) as i32) * (GLYPH_WIDTH as i32 + 1) - 1;
    wrap_text(text, max_px, 1)
}

pub fn render_gallery_overlay(overlay: &mut OverlayBuffer, g: &GalleryHudSnapshot<'_>) {
    overlay.clear();

    let w = overlay.width as i32;
    let h = overlay.height as i32;

    // Barra superior
    let bar_w = (w - 20).max(100);
    overlay.fill_rect(10, 8, bar_w, 24, Color::new(8, 12, 20, 200));
    overlay.stroke_rect(10, 8, bar_w, 24, Color::new(70, 110, 150, 225));
    let header_raw = format!(
        "Galería de trofeos — A/D: cambiar objeto · Ratón o flechas: girar · R: reiniciar vista · 4/5: ajustar · Esc: menú · {:.0} FPS",
        g.fps
    );
    let header = fit_text_with_ellipsis(&header_raw, bar_w - 12, 1);
    overlay.draw_text(&header, 16, 15, 1, Color::WHITE);

    // Ficha de datos (esquina inferior izquierda)
    let card_w = 390.min(w - 20);
    let inner_w = card_w - 24;
    let lod_str = if g.lod_level >= 5 {
        "máximo (5 de 5)".to_string()
    } else {
        format!("{} de 5", g.lod_level)
    };
    let mut raw_rows = vec![
        format!("Colección: {}", g.entry.sistema.display_name()),
        format!("Tamaño: {:.2}", g.radius),
        format!("Triángulos: {} · Nivel de detalle: {}", g.tris, lod_str),
        format!("Estilo visual: {}", g.entry.nombre_visible),
        format!(
            "{}  [teclas 4 y 5]: {:.2}",
            g.live_param_name, g.live_param_value
        ),
    ];
    if let Some(insp) = g.entry.inspiracion {
        raw_rows.push(format!("Inspiración: {}", insp));
    }
    let mut rows: Vec<String> = Vec::new();
    for r in raw_rows {
        rows.extend(wrap_text(&r, inner_w, 1));
    }
    let mut desc_lines = wrap_text(g.entry.descripcion_es, inner_w, 1);
    let max_card_h = (h - 50).max(80);
    let max_desc = ((max_card_h - 62 - (rows.len() as i32) * 13) / 11).max(1) as usize;
    if desc_lines.len() > max_desc {
        desc_lines.truncate(max_desc);
        if let Some(last) = desc_lines.last_mut() {
            *last = fit_text_with_ellipsis(&format!("{last}…"), inner_w, 1);
        }
    }
    let card_h = 62 + (rows.len() as i32) * 13 + (desc_lines.len() as i32) * 11;
    let card_x = 10;
    let card_y = (h - card_h - 10).max(40);
    overlay.fill_rect(card_x, card_y, card_w, card_h, Color::new(8, 12, 22, 210));
    overlay.stroke_rect(card_x, card_y, card_w, card_h, Color::new(90, 150, 210, 230));

    let title_scale = if OverlayBuffer::measure_text(g.entry.nombre, 2).0 <= inner_w {
        2
    } else {
        1
    };
    let title_txt = fit_text_with_ellipsis(g.entry.nombre, inner_w, title_scale);
    overlay.draw_text(&title_txt, card_x + 12, card_y + 10, title_scale, Color::new(255, 220, 130, 255));
    let mut ry = card_y + 32;
    for row in &rows {
        overlay.draw_text(row, card_x + 12, ry, 1, Color::new(210, 224, 240, 255));
        ry += 13;
    }
    ry += 4;
    overlay.draw_text("Descripción:", card_x + 12, ry, 1, Color::new(140, 210, 255, 255));
    ry += 12;
    for line in desc_lines {
        overlay.draw_text(&line, card_x + 12, ry, 1, Color::new(225, 232, 240, 255));
        ry += 11;
    }

    // Lista de trofeos (derecha): los Placeholder aparecen atenuados con "en construcción"
    let list_w = 276.min(w - card_w - 30).max(180);
    let entries = crate::showcase::SHOWCASE_ENTRIES;
    let list_h = (entries.len() as i32 * 11 + 26).min(h - 50);
    let list_x = w - list_w - 10;
    let list_y = 40;
    overlay.fill_rect(list_x, list_y, list_w, list_h, Color::new(8, 12, 22, 200));
    overlay.stroke_rect(list_x, list_y, list_w, list_h, Color::new(75, 120, 170, 220));
    overlay.draw_text("Trofeos", list_x + 8, list_y + 6, 1, Color::new(150, 220, 255, 255));

    for (i, item) in entries.iter().enumerate() {
        let iy = list_y + 20 + i as i32 * 11;
        if iy + 10 >= list_y + list_h {
            break;
        }
        let is_active = i == g.active_showcase_idx;
        let is_placeholder = item.estado == crate::showcase::ShowcaseStatus::Placeholder;
        let color = if is_active {
            Color::new(255, 230, 130, 255)
        } else if is_placeholder {
            Color::new(115, 122, 135, 255)
        } else {
            Color::new(210, 222, 235, 255)
        };
        let prefix = if is_active { ">" } else { " " };
        let line = if is_placeholder {
            format!("{} {} (en construcción)", prefix, item.nombre)
        } else {
            format!("{} {}", prefix, item.nombre)
        };
        let truncated = fit_text_with_ellipsis(&line, list_w - 12, 1);
        overlay.draw_text(&truncated, list_x + 6, iy, 1, color);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glyph_table_covers_ascii_and_spanish_characters() {
        for code in 32_u8..=126_u8 {
            let ch = code as char;
            let glyph = glyph_5x7(ch)
                .unwrap_or_else(|| panic!("Falta glifo para ASCII {code} ({ch:?})"));
            if ch == ' ' {
                assert!(glyph.iter().all(|&r| r == 0));
            } else {
                assert!(
                    glyph.iter().any(|&r| r != 0),
                    "Glifo vacío para {ch:?}"
                );
            }
        }
        for ch in [
            'á', 'é', 'í', 'ó', 'ú', 'ü', 'ñ', '¿', '¡', 'Á', 'É', 'Í', 'Ó', 'Ú', 'Ü', 'Ñ',
            '·', '—', '…',
        ] {
            let glyph = glyph_5x7(ch).unwrap_or_else(|| panic!("Falta glifo para {ch:?}"));
            assert!(glyph.iter().any(|&r| r != 0));
        }
    }

    #[test]
    fn all_glyphs_have_distinct_bitmaps() {
        let mut seen: std::collections::HashMap<[u8; 7], char> = std::collections::HashMap::new();
        let extras = [
            'á', 'é', 'í', 'ó', 'ú', 'ü', 'ñ', '¿', '¡', 'Á', 'É', 'Í', 'Ó', 'Ú', 'Ü', 'Ñ',
            '·', '—', '…',
        ];
        for ch in (32_u8..=126_u8).map(|c| c as char).chain(extras) {
            let glyph = glyph_5x7(ch).unwrap_or_else(|| panic!("Falta glifo para {ch:?}"));
            if let Some(&prev) = seen.get(&glyph) {
                panic!("Glifos idénticos para {prev:?} y {ch:?}: {glyph:?}");
            }
            seen.insert(glyph, ch);
        }
    }

    #[test]
    fn all_program_ui_strings_have_glyph_coverage() {
        let mut texts: Vec<&str> = Vec::new();
        for entry in crate::app::MENU_ENTRIES {
            texts.push(entry.label());
        }
        for sys in [crate::app::SystemId::Realista, crate::app::SystemId::Favoritos] {
            texts.push(sys.display_name());
        }
        for c in CONTROLS {
            texts.push(c.key);
            texts.push(c.action);
        }
        for e in crate::showcase::SHOWCASE_ENTRIES {
            texts.push(e.nombre);
            texts.push(e.nombre_visible);
            texts.push(e.tipo);
            texts.push(e.descripcion_es);
            if let Some(insp) = e.inspiracion {
                texts.push(insp);
            }
            if let Some(sh) = e.shader {
                texts.push(crate::shaders::shader_name(sh));
                texts.push(crate::shaders::live_param(sh).name);
            }
        }
        for text in texts {
            for ch in text.chars() {
                assert!(
                    glyph_5x7(ch).is_some(),
                    "Carácter sin glifo {ch:?} en texto {text:?}"
                );
            }
        }
    }

    #[test]
    fn wrap_text_respects_max_width() {
        let lines = wrap_text("Uno dos tres cuatro cinco seis siete", 70, 1);
        assert!(lines.len() >= 3);
        for line in &lines {
            let (w, _) = OverlayBuffer::measure_text(line, 1);
            assert!(w <= 70, "Línea excede 70px ({w}px): {line:?}");
        }
    }

    #[test]
    fn overlay_text_fits_window_and_clips_safely() {
        let mut ov = OverlayBuffer::new(800, 600);
        assert!(ov.text_fits_window("Sistema Solar | FPS: 60", 16, 16, 2));
        assert!(!ov.text_fits_window(&"X".repeat(120), 16, 16, 2));

        ov.draw_text("Hola mundo ¡ñáéíóú!", -10, 595, 2, Color::WHITE);
        ov.draw_text(&"Z".repeat(200), 10, 10, 2, Color::GREEN);
        assert_eq!(ov.pixels.len(), 800 * 600 * 4);
    }

    #[test]
    fn controls_table_fits_help_panel_in_800x600() {
        let ov = OverlayBuffer::new(800, 600);
        assert!(!CONTROLS.is_empty());
        for (i, c) in CONTROLS.iter().enumerate() {
            let y = 160 + i as i32 * 13;
            assert!(ov.text_fits_window(c.key, 18, y, 1), "Key overflow: {}", c.key);
            assert!(
                ov.text_fits_window(c.action, 120, y, 1),
                "Action overflow: {}",
                c.action
            );
        }
    }

    #[test]
    fn menu_overlay_fits_in_800x600() {
        let mut ov = OverlayBuffer::new(800, 600);
        render_menu_overlay(&mut ov, 0, false, false);
        for entry in crate::app::MENU_ENTRIES {
            assert!(ov.text_fits_window(entry.label(), 200, 240, 2));
        }
        render_menu_overlay(&mut ov, 3, false, true);
        assert_eq!(ov.pixels.len(), 800 * 600 * 4);
    }

    #[test]
    fn gallery_overlay_fits_in_800x600() {
        let mut ov = OverlayBuffer::new(800, 600);
        let snap = GalleryHudSnapshot {
            entry: &crate::showcase::SHOWCASE_ENTRIES[1],
            active_showcase_idx: 1,
            radius: 1.25,
            tris: 5120,
            lod_level: 4,
            shader_name: "Rocky",
            live_param_name: "Contraste relieve",
            live_param_value: 1.0,
            fps: 60.0,
        };
        render_gallery_overlay(&mut ov, &snap);
        assert_eq!(ov.pixels.len(), 800 * 600 * 4);
    }

    #[test]
    fn all_ui_screens_and_extreme_hud_fit_boxes_and_800x600_window() {
        let mut ov = OverlayBuffer::new(800, 600);
        let w = 800_i32;
        let _h = 600_i32;

        // 1. Menú con todas sus entradas y subpanel de controles
        let box_w = 520.min(w - 40).max(260);
        let bx = (w - box_w) / 2;
        for entry in crate::app::MENU_ENTRIES {
            let line = format!("> {}", entry.label());
            let (lw, lh) = OverlayBuffer::measure_text(&line, 2);
            assert!(
                bx + 30 + lw <= bx + box_w - 10,
                "Entrada de menú excede recuadro: {line:?}"
            );
            assert!(ov.text_fits_window(&line, bx + 30, 200, 2));
            let _ = lh;
        }
        let cw = (w - 40).min(700);
        for (i, c) in CONTROLS.iter().enumerate() {
            let row_y = 60 + i as i32 * 13;
            let (kw, _) = OverlayBuffer::measure_text(c.key, 1);
            assert!(16 + kw < 132, "Tecla de control invade columna: {}", c.key);
            let act = fit_text_with_ellipsis(c.action, cw - 144, 1);
            let (aw, _) = OverlayBuffer::measure_text(&act, 1);
            assert!(132 + aw <= cw - 8, "Acción excede recuadro: {act:?}");
            assert!(ov.text_fits_window(&act, (w - cw) / 2 + 132, row_y, 1));
        }
        render_menu_overlay(&mut ov, 1, false, false);
        render_menu_overlay(&mut ov, 3, true, true);

        // 2. Galería con CADA cuerpo del registro
        let bar_w = w - 20;
        let card_w = 390.min(w - 20);
        let inner_w = card_w - 24;
        let list_w = 276.min(w - card_w - 30).max(180);
        for (idx, entry) in crate::showcase::SHOWCASE_ENTRIES.iter().enumerate() {
            let sh = entry.shader.unwrap_or(crate::shaders::Shader::Chunky);
            let spec = crate::shaders::live_param(sh);
            let snap = GalleryHudSnapshot {
                entry,
                active_showcase_idx: idx,
                radius: 1.0,
                tris: 999999,
                lod_level: 5,
                shader_name: crate::shaders::shader_name(sh),
                live_param_name: spec.name,
                live_param_value: spec.default,
                fps: 9999.0,
            };
            render_gallery_overlay(&mut ov, &snap);

            let header = fit_text_with_ellipsis(
                "Galería de trofeos — A/D: cambiar objeto · Ratón o flechas: girar · R: reiniciar vista · 4/5: ajustar · Esc: menú · 9999 FPS",
                bar_w - 12,
                1,
            );
            let (hw, _) = OverlayBuffer::measure_text(&header, 1);
            assert!(16 + hw <= 10 + bar_w, "Header galería excede barra");
            assert!(ov.text_fits_window(&header, 16, 15, 1));

            let desc_lines = wrap_text(entry.descripcion_es, inner_w, 1);
            assert!(!desc_lines.is_empty(), "Descripción vacía en {}", entry.id);
            for line in &desc_lines {
                let (lw, _) = OverlayBuffer::measure_text(line, 1);
                assert!(lw <= inner_w, "Línea de ficha excede recuadro en {}: {line:?}", entry.id);
            }
            let list_line = fit_text_with_ellipsis(&format!("> {}", entry.nombre), list_w - 12, 1);
            let (llw, _) = OverlayBuffer::measure_text(&list_line, 1);
            assert!(llw <= list_w - 12, "Lista excede recuadro en {}", entry.id);
        }

        // 3. HUD de vuelo con valores extremos (9999 FPS, 999999 triángulos, nombres largos)
        let extreme_title = "Sistema de favoritos (13 objetos) · Tiempo: 999.9s (2.00x PAUSA) · Cámara [F]: Seguir nave · Ayuda [H]";
        let extreme_hud = HudSnapshot {
            title: extreme_title,
            fps: 9999.9,
            total_ms: 99.9,
            vertex_ms: 12.34,
            raster_ms: 23.45,
            shading_ms: 34.56,
            present_ms: 4.56,
            internal_width: 800,
            internal_height: 600,
            threads: 16,
            tris_before_cull: 999999,
            tris_after_cull: 999999,
            frustum_culling: true,
            lod_summary: "Nivel de detalle [Z]: Automático · Sol:5/5(20480) Cre:5/5(3999) Cas:5/5(2660) La :5/5(3252) Raq:5/5(3800) Meg:5/5(1800)",
            ship_lod_summary: "Nave: nivel 1 de 3 (3000 triángulos)",
            mode_summary: "Cerca de Sol de los Phoenix Suns (Automático)",
            gravity_enabled: true,
            gravity_mag: 9.81,
            speed: 99.99,
            altitude_summary: "999.99 (Cresta del Tottenham)",
            time_scale: 2.0,
            paused: true,
            surface_seed: 999999,
            instances_drawn: 9999,
            instances_total: 9999,
            instance_culling: true,
            selected_summary: "Elegido [Tab]: Sol de los Phoenix Suns · Tamaño [9/0]: 99.99 · Velocidad de giro [4/5]: 2.00",
            collision_body: Some("Sol de los Phoenix Suns"),
            show_perf: true,
            show_help: true,
            show_hud: true,
            minimap: None,
        };
        render_hud(&mut ov, &extreme_hud);
        let hud_box_w = (w - 20).min(760);
        for wrapped in wrap_text(extreme_hud.lod_summary, hud_box_w - 16, 1) {
            let (ww, _) = OverlayBuffer::measure_text(&wrapped, 1);
            assert!(ww <= hud_box_w - 16, "HUD envuelto excede recuadro: {wrapped:?}");
            assert!(ov.text_fits_window(&wrapped, 18, 100, 1));
        }
        let help_w = (w - 20).min(700);
        for c in CONTROLS {
            let act = fit_text_with_ellipsis(c.action, help_w - 142, 1);
            let (aw, _) = OverlayBuffer::measure_text(&act, 1);
            assert!(134 + aw <= 10 + help_w, "Ayuda HUD excede recuadro: {act:?}");
        }
    }
}
