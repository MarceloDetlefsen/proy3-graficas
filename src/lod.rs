#![allow(dead_code)]
//! Selección de nivel de detalle (LOD) según tamaño proyectado en pantalla.
//!
//! Calcula el radio proyectado en píxeles de una esfera envolvente respecto a
//! la cámara activa y selecciona el nivel de subdivisión de icosfera (`0..=5`)
//! o el nivel de detalle de la nave (`0..=2`), con histéresis para evitar oscilaciones.

use raylib::prelude::Vector3;

use crate::camera::Camera;

/// Umbrales ascendentes de radio en píxeles para pasar de nivel `k` a `k+1` en icosferas.
pub const ICOSPHERE_THRESHOLDS_PX: [f32; 5] = [8.0, 18.0, 40.0, 85.0, 160.0];
/// Umbrales ascendentes de radio en píxeles para la nave (mapeados a LOD 2 -> 1 -> 0).
pub const SHIP_THRESHOLDS_PX: [f32; 2] = [14.0, 45.0];
pub const DEFAULT_HYSTERESIS: f32 = 0.12;

/// Estima el radio proyectado en píxeles de un cuerpo esférico en pantalla.
pub fn projected_radius_px(
    cam: &Camera,
    world_pos: Vector3,
    world_radius: f32,
    viewport_height: f32,
) -> f32 {
    let dist = (world_pos - cam.position).length().max(world_radius * 0.25).max(1e-4);
    let focal_px = 0.5 * viewport_height.max(1.0) / (0.5 * cam.fov_y).tan().max(1e-4);
    (world_radius.max(0.0) / dist) * focal_px
}

/// Elige un nivel en `0..=thresholds_asc.len()` (mayor = más detalle) según `radius_px`.
pub fn select_detail_level(radius_px: f32, thresholds_asc: &[f32]) -> usize {
    let mut level = 0;
    for &t in thresholds_asc {
        if radius_px >= t {
            level += 1;
        } else {
            break;
        }
    }
    level
}

/// Selecciona nivel en `0..=thresholds_asc.len()` con histéresis alrededor de
/// cada umbral para evitar oscilaciones cuando el tamaño proyectado fluctúa.
pub fn select_detail_level_hysteresis(
    prev_level: usize,
    radius_px: f32,
    thresholds_asc: &[f32],
    hysteresis: f32,
) -> usize {
    let max_level = thresholds_asc.len();
    let mut level = prev_level.min(max_level);
    let h = hysteresis.clamp(0.0, 0.45);

    while level < max_level && radius_px > thresholds_asc[level] * (1.0 + h) {
        level += 1;
    }
    while level > 0 && radius_px < thresholds_asc[level - 1] * (1.0 - h) {
        level -= 1;
    }
    level
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lod_hysteresis_prevents_oscillation_near_threshold() {
        let mut level = select_detail_level(10.0, &ICOSPHERE_THRESHOLDS_PX);
        assert_eq!(level, 1);

        // Oscilando alrededor del umbral 18.0 (banda [15.84, 20.16]) se mantiene en 1.
        level = select_detail_level_hysteresis(level, 18.5, &ICOSPHERE_THRESHOLDS_PX, 0.12);
        assert_eq!(level, 1);
        level = select_detail_level_hysteresis(level, 19.5, &ICOSPHERE_THRESHOLDS_PX, 0.12);
        assert_eq!(level, 1);

        // Al superar 18.0 * 1.12 = 20.16 sube a nivel 2.
        level = select_detail_level_hysteresis(level, 20.5, &ICOSPHERE_THRESHOLDS_PX, 0.12);
        assert_eq!(level, 2);

        // Al bajar levemente bajo 18.0 (dentro de la banda) permanece en nivel 2.
        level = select_detail_level_hysteresis(level, 17.0, &ICOSPHERE_THRESHOLDS_PX, 0.12);
        assert_eq!(level, 2);

        // Al bajar de 18.0 * 0.88 = 15.84 regresa a nivel 1.
        level = select_detail_level_hysteresis(level, 15.5, &ICOSPHERE_THRESHOLDS_PX, 0.12);
        assert_eq!(level, 1);
    }
}
