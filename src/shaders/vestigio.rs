#![allow(dead_code)]
//! Shader Vestigio (película vieja / VHS): dunas anisotrópicas, remolinos de polvo y grano.

use raylib::prelude::Vector3;

use crate::Uniforms;
use crate::fragment::Fragment;
use crate::noise::{self, Noise};
use crate::shaders::{diffuse_factor, surface_mu};

const OLIVE: Vector3 = Vector3 { x: 0.420, y: 0.478, z: 0.294 }; // #6B7A4B
const DUSTY_ROSE: Vector3 = Vector3 { x: 0.788, y: 0.561, z: 0.541 }; // #C98F8A
const CREAM: Vector3 = Vector3 { x: 0.922, y: 0.867, z: 0.749 }; // #EBDDBF
const SEPIA: Vector3 = Vector3 { x: 0.710, y: 0.545, z: 0.353 }; // #B58B5A

pub fn height_at(noise: &Noise, dir: Vector3, frequency: f32) -> f32 {
    let p = dir.normalize();
    let warp = noise.fbm(p * (frequency * 0.35), 2, 2.0, 0.5) * 0.22;
    // Dunas estiradas ~8x a lo largo de los paralelos (frecuencia alta en Y, baja en X/Z).
    let aniso = Vector3::new(p.x * 0.25, p.y * 2.0 + warp, p.z * 0.25) * frequency;
    let raw = noise.ridged(aniso, 3, 2.1, 0.52).clamp(0.0, 1.0);
    noise::smoothstep(0.15, 0.88, raw)
}

#[inline]
fn tangent_basis(p: Vector3) -> (Vector3, Vector3) {
    let up = if p.y.abs() > 0.95 {
        Vector3::new(1.0, 0.0, 0.0)
    } else {
        Vector3::new(0.0, 1.0, 0.0)
    };
    let t = up.cross(p).normalize();
    let b = p.cross(t).normalize();
    (t, b)
}

pub fn sample_static(noise: &Noise, dir: Vector3, frequency: f32) -> [f32; 4] {
    let p = dir.normalize();
    let dunes = height_at(noise, p, frequency);
    let (t, b) = tangent_basis(p);
    let eps = 0.015;
    let h_t = height_at(noise, (p + t * eps).normalize(), frequency);
    let h_b = height_at(noise, (p + b * eps).normalize(), frequency);
    let slope_t = ((h_t - dunes) / eps).clamp(-3.0, 3.0);
    let slope_b = ((h_b - dunes) / eps).clamp(-3.0, 3.0);
    let erosion = noise.fbm(p * 2.4 + Vector3::new(1.9, -3.2, 0.7), 3, 2.0, 0.5);
    [dunes, slope_t, slope_b, erosion]
}

#[inline]
fn film_grain(x: i32, y: i32, tick: u32) -> f32 {
    let mut h = (x as u32).wrapping_mul(374_761_393)
        ^ (y as u32).wrapping_mul(668_265_263)
        ^ tick.wrapping_mul(2_147_483_647);
    h = (h ^ (h >> 13)).wrapping_mul(1_274_126_177);
    ((h ^ (h >> 16)) & 0xFF) as f32 / 255.0 - 0.5
}

fn surface_albedo(dunes: f32, erosion: f32) -> Vector3 {
    let valley_mix = noise::smoothstep(0.0, 0.32, dunes);
    let base_biome = noise::mix_color(OLIVE, DUSTY_ROSE, noise::smoothstep(0.32, 0.70, erosion));
    let with_valley = noise::mix_color(SEPIA * 0.72, base_biome, valley_mix);
    let crest = noise::smoothstep(0.62, 0.92, dunes);
    noise::mix_color(with_valley, CREAM, crest * 0.85)
}

pub fn shade(fragment: &Fragment, uniforms: &Uniforms) -> Vector3 {
    let p = fragment.object_pos.normalize();
    let (t_vec, _) = tangent_basis(p);
    let [dunes, slope_t, slope_b, erosion] = if let Some(baked) = uniforms.baked.as_ref() {
        baked.vestigio.sample_dir(p)
    } else {
        sample_static(&uniforms.noise, p, 5.0)
    };

    // Perturbar la normal con el gradiente de las dunas para sombrear el lado de sotavento.
    let n_base = fragment.normal.normalize();
    let world_t = n_base.cross(Vector3::new(0.0, 1.0, 0.0));
    let wt = if world_t.length() > 1e-4 {
        world_t.normalize()
    } else {
        Vector3::new(1.0, 0.0, 0.0)
    };
    let wb = n_base.cross(wt).normalize();
    let perturbed_normal = (n_base - wt * (slope_t * 0.42) - wb * (slope_b * 0.42)).normalize();

    let mut base = surface_albedo(dunes, erosion);

    // 4 dust devils grandes (~3x) con estelas curvas que se desvanecen.
    let time = uniforms.time;
    let devils: [(f32, f32, f32, f32); 4] = [
        (0.45, 0.22, 0.65, 0.0),
        (-0.38, -0.35, 0.58, 1.6),
        (0.52, 0.48, 0.62, 3.1),
        (-0.44, 0.10, 0.70, 4.7),
    ];
    let mut devil_mask = 0.0_f32;
    for (speed, lat, radius, phase) in devils {
        let ang = time * speed + phase;
        let cos_l = (1.0 - lat * lat).max(0.0).sqrt();
        let center = Vector3::new(ang.cos() * cos_l, lat, ang.sin() * cos_l).normalize();
        let diff = p - center;
        let dist = diff.length();
        let core = (1.0 - (dist / radius).min(1.0)).powi(2);
        let swirl = (diff.x * 12.0 - diff.z * 12.0 + time * 4.0).sin() * 0.5 + 0.5;
        // Estela curva detrás del remolino.
        let trail_ang = ang - speed.signum() * 0.48;
        let trail_center = Vector3::new(trail_ang.cos() * cos_l, lat * 0.92, trail_ang.sin() * cos_l).normalize();
        let trail_dist = (p - trail_center).length();
        let trail = (1.0 - (trail_dist / (radius * 1.15)).min(1.0)).powi(2) * 0.55;
        devil_mask = devil_mask.max((core * (0.65 + 0.35 * swirl) + trail).min(1.0));
    }
    base = noise::mix_color(base, noise::mix_color(DUSTY_ROSE, CREAM, 0.45), devil_mask * 0.85);

    let diffuse = diffuse_factor(perturbed_normal, uniforms);
    let shade_curve = 0.14 + 0.86 * diffuse;
    let mut color = base * shade_curve;

    // Desfase cromático marcado hacia el borde (limb) estilo VHS.
    let mu = surface_mu(fragment, uniforms);
    let limb = 1.0 - mu;
    let ch_offset = 0.008 + 0.042 * limb * limb;
    if let Some(baked) = uniforms.baked.as_ref() {
        let [d_r, _, _, e_r] = baked.vestigio.sample_dir((p + t_vec * ch_offset).normalize());
        let [d_b, _, _, e_b] = baked.vestigio.sample_dir((p - t_vec * ch_offset).normalize());
        let c_r = surface_albedo(d_r, e_r).x * shade_curve;
        let c_b = surface_albedo(d_b, e_b).z * shade_curve;
        color.x = color.x * 0.35 + c_r * 0.65;
        color.z = color.z * 0.35 + c_b * 0.65;
    } else {
        color.x = (color.x + 0.09 * limb).min(1.0);
        color.z = (color.z - 0.06 * limb).max(0.0);
    }

    // Estilización VHS marcada: scanlines visibles, grano intenso, viñeta fuerte y cuantización ~6 niveles.
    let scanline = 1.0 - 0.16 * (0.5 + 0.5 * (fragment.position.y * 1.5 + time * 10.0).sin());
    let tick = (time * 24.0) as u32;
    let grain = film_grain(fragment.position.x as i32, fragment.position.y as i32, tick) * 0.16;
    let vignette = 0.42 + 0.58 * mu.powf(0.8);
    color = color * scanline * vignette + Vector3::new(grain, grain * 0.9, grain * 0.75);

    let levels = 6.0_f32;
    Vector3::new(
        (color.x.clamp(0.0, 1.0) * levels).round() / levels,
        (color.y.clamp(0.0, 1.0) * levels).round() / levels,
        (color.z.clamp(0.0, 1.0) * levels).round() / levels,
    )
}
