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
