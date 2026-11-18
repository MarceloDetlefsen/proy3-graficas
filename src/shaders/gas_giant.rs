#![allow(dead_code)]

use raylib::prelude::Vector3;
use std::sync::Once;

use crate::Uniforms;
use crate::fragment::Fragment;
use crate::noise::{self, Noise};
use crate::shaders::{diffuse_factor, surface_mu};

// Frecuencia del seno latitudinal; valores altos producen más bandas.
pub const BAND_FREQ: f32 = 6.0;
// Frecuencia del primer warp, responsable de curvas amplias.
pub const WARP1_FREQ: f32 = 1.35;
// Amplitud del primer warp.
pub const WARP1_AMP: f32 = 0.48;
// Frecuencia del segundo warp, responsable del marmoleado líquido fino.
pub const WARP2_FREQ: f32 = 7.6;
// Amplitud menor de la segunda pasada de warp.
pub const WARP2_AMP: f32 = 0.20;
// Frecuencia de la tercera pasada, usada solo para filamentos finos.
pub const WARP3_FREQ: f32 = 14.0;
// Amplitud baja de la tercera pasada para evitar bandas rígidas.
pub const WARP3_AMP: f32 = 0.055;
// Percentil bajo del patrón usado para expandir el contraste a [0,1].
pub const PATTERN_LOW: f32 = 0.18;
// Percentil alto del patrón usado para expandir el contraste a [0,1].
pub const PATTERN_HIGH: f32 = 0.82;
// Centro normalizado del vórtice, ubicado hacia el hemisferio frontal (+z).
pub const VORTEX_CENTER: [f32; 3] = [-0.22, -0.22, 0.95];
// Radio angular aproximado del gran vórtice naranja.
pub const VORTEX_RADIUS: f32 = 0.60;
// Radio de influencia de la rotación sobre las bandas vecinas.
pub const VORTEX_INFLUENCE_RADIUS: f32 = 1.20;
// Giro de las coordenadas dentro del vórtice; valores altos enrollan más las bandas.
pub const VORTEX_STRENGTH: f32 = 3.4;
// Frecuencia del ruido que rompe el borde circular del vórtice.
pub const VORTEX_EDGE_FREQ: f32 = 4.5;
// Amplitud de la perturbación de distancia que vuelve orgánico el borde.
pub const VORTEX_EDGE_NOISE: f32 = 0.22;
// Verde muy pálido de las zonas de patrón más claras.
pub const COLOR_GREEN_PALE: [f32; 3] = [0.55, 0.82, 0.28];
// Verde brillante dominante.
pub const COLOR_GREEN_BRIGHT: [f32; 3] = [0.30, 0.72, 0.10];
// Verde oliva de las bandas intermedias.
pub const COLOR_GREEN_OLIVE: [f32; 3] = [0.18, 0.35, 0.07];
// Verde profundo de las sombras.
pub const COLOR_GREEN_DARK: [f32; 3] = [0.07, 0.14, 0.04];
// Naranja rojizo del vórtice.
pub const COLOR_VORTEX: [f32; 3] = [0.95, 0.40, 0.10];
// Sombra rojiza interna que da volumen y lectura líquida al vórtice.
pub const COLOR_VORTEX_DARK: [f32; 3] = [0.48, 0.10, 0.035];
// Frecuencia del marmoleado interno del vórtice.
pub const VORTEX_DETAIL_FREQ: f32 = 7.0;
// Ambiente mínimo para conservar detalle en el lado oscuro.
pub const AMBIENT: f32 = 0.12;
// Intensidad del tinte verde brillante en el borde atmosférico.
pub const RIM_INTENSITY: f32 = 0.28;
// Velocidad temporal del primer warp, lenta para mover masas atmosféricas amplias.
pub const WARP1_SPEED: f32 = 0.035;
// Velocidad temporal del segundo warp, más rápida para mezclar el marmoleado.
pub const WARP2_SPEED: f32 = 0.085;
// Velocidad de rotación propia del vórtice.
pub const VORTEX_SPIN: f32 = 0.060;

static PRINTED_STATS: Once = Once::new();

fn time_shift(t: f32) -> Vector3 {
    Vector3::new(t * 0.5, t * 0.3, t * 0.4)
}

fn domain_warp(noise: &Noise, object_pos: Vector3, time: f32) -> Vector3 {
    let p = vortex_domain(object_pos.normalize(), time);
    let warp1 = noise.fbm(p * WARP1_FREQ + time_shift(time * WARP1_SPEED), 3, 2.0, 0.5) * 2.0 - 1.0;
    let first = p + Vector3::new(warp1, warp1 * 0.72, -warp1 * 0.58) * WARP1_AMP;
    let warp2 = noise.fbm(first * WARP2_FREQ + time_shift(time * WARP2_SPEED), 3, 2.0, 0.5) * 2.0 - 1.0;
    let second = first + Vector3::new(-warp2 * 0.62, warp2, warp2 * 0.48) * WARP2_AMP;
    let warp3 = noise.fbm(
        second * WARP3_FREQ + time_shift(time * WARP2_SPEED * 1.45),
        3,
        2.0,
        0.5,
    ) * 2.0
        - 1.0;
    second + Vector3::new(warp3 * 0.75, -warp3 * 0.45, warp3 * 0.35) * WARP3_AMP
}

fn vortex_domain(p: Vector3, time: f32) -> Vector3 {
    let center = Vector3::new(VORTEX_CENTER[0], VORTEX_CENTER[1], VORTEX_CENTER[2]).normalize();
    let offset = p - center;
    let distance = offset.length();
    let falloff = (-distance / VORTEX_INFLUENCE_RADIUS).exp();
    let angle = VORTEX_STRENGTH * falloff + time * VORTEX_SPIN * falloff;
    let (sin_angle, cos_angle) = angle.sin_cos();
    let cross = Vector3::new(
        center.y * offset.z - center.z * offset.y,
        center.z * offset.x - center.x * offset.z,
        center.x * offset.y - center.y * offset.x,
    );
    offset * cos_angle + cross * sin_angle + center * center.dot(offset) * (1.0 - cos_angle)
}

fn raw_pattern(noise: &Noise, object_pos: Vector3, time: f32) -> f32 {
    let warped = domain_warp(noise, object_pos, time);
    let latitude = warped.y.clamp(-1.0, 1.0);
    let bands = (latitude * BAND_FREQ).sin() * 0.5 + 0.5;
    let fluid_noise = noise.fbm(warped * 2.2 + time_shift(time * WARP1_SPEED), 3, 2.0, 0.5);
    bands * 0.52 + fluid_noise * 0.48
}

pub fn band_pattern_at(noise: &Noise, object_pos: Vector3, time: f32) -> f32 {
    noise::remap(
        raw_pattern(noise, object_pos, time),
        PATTERN_LOW,
        PATTERN_HIGH,
        0.0,
        1.0,
    )
    .clamp(0.0, 1.0)
}

pub fn band_pattern(noise: &Noise, object_pos: Vector3) -> f32 {
    band_pattern_at(noise, object_pos, 0.0)
}

fn color(values: [f32; 3]) -> Vector3 {
    Vector3::new(values[0], values[1], values[2])
}

fn palette_color(pattern: f32) -> Vector3 {
    let value = pattern.clamp(0.0, 1.0);
    if value < 0.33 {
        noise::mix_color(
            color(COLOR_GREEN_DARK),
            color(COLOR_GREEN_OLIVE),
            noise::smoothstep(0.0, 0.33, value),
        )
    } else if value < 0.68 {
        noise::mix_color(
            color(COLOR_GREEN_OLIVE),
            color(COLOR_GREEN_BRIGHT),
            noise::smoothstep(0.33, 0.68, value),
        )
    } else {
        noise::mix_color(
            color(COLOR_GREEN_BRIGHT),
            color(COLOR_GREEN_PALE),
            noise::smoothstep(0.68, 1.0, value),
        )
    }
}

fn vortex_mask(noise: &Noise, object_pos: Vector3) -> f32 {
    let p = object_pos.normalize();
    let center = Vector3::new(VORTEX_CENTER[0], VORTEX_CENTER[1], VORTEX_CENTER[2]).normalize();
    let distance = (p - center).length();
    let edge_noise = noise.fbm(p * VORTEX_EDGE_FREQ, 3, 2.0, 0.5);
    let perturbed_distance = distance + (edge_noise - 0.5) * VORTEX_EDGE_NOISE;
    1.0 - noise::smoothstep(
        VORTEX_RADIUS * 0.42,
        VORTEX_RADIUS * 1.30,
        perturbed_distance,
    )
}

fn surface_color(noise: &Noise, object_pos: Vector3, pattern: f32, time: f32) -> Vector3 {
    let base = palette_color(pattern);
    let vortex = vortex_mask(noise, object_pos);
    let detail = noise.fbm(
        domain_warp(noise, object_pos, time) * VORTEX_DETAIL_FREQ + time_shift(time * WARP2_SPEED),
        3,
        2.0,
        0.5,
    );
    let vortex_color = noise::mix_color(
        color(COLOR_VORTEX_DARK),
        color(COLOR_VORTEX),
        noise::smoothstep(0.30, 0.70, detail),
    );
    let green_filaments = noise::smoothstep(0.40, 0.68, detail);
    let orange_mix = vortex * (0.35 + 0.65 * green_filaments);
    noise::mix_color(base, vortex_color, orange_mix)
}

fn percentile(sorted: &[f32], fraction: f32) -> f32 {
    sorted[(fraction * (sorted.len() - 1) as f32) as usize]
}

fn print_statistics_once(noise: &Noise) {
    PRINTED_STATS.call_once(|| {
        let sample_count = 4096;
        let golden_angle = std::f32::consts::PI * (3.0 - 5.0_f32.sqrt());
        let mut values = Vec::with_capacity(sample_count);
        for i in 0..sample_count {
            let y = 1.0 - (i as f32 + 0.5) * 2.0 / sample_count as f32;
            let radius = (1.0 - y * y).sqrt();
            let angle = golden_angle * i as f32;
            let point = Vector3::new(radius * angle.cos(), y, radius * angle.sin());
            values.push(raw_pattern(noise, point, 0.0));
        }
        values.sort_by(f32::total_cmp);
        println!(
            "Gas pattern: min={:.3}, p05={:.3}, p50={:.3}, p95={:.3}, max={:.3}",
            values[0],
            percentile(&values, 0.05),
            percentile(&values, 0.50),
            percentile(&values, 0.95),
            values[values.len() - 1],
        );
    });
}

pub fn sample_static(noise: &Noise, dir: Vector3) -> [f32; 5] {
    let p = dir.normalize();
    let pattern = band_pattern_at(noise, p, 0.0);
    let albedo = surface_color(noise, p, pattern, 0.0);
    let vortex = vortex_mask(noise, p);
    [albedo.x, albedo.y, albedo.z, pattern, vortex]
}

pub fn shade_from_static(fragment: &Fragment, uniforms: &Uniforms, s: [f32; 5]) -> Vector3 {
    let [r, g, b, pattern, _vortex] = s;
    if uniforms.rocky_debug_view == 0 {
        return Vector3::new(pattern, pattern, pattern);
    }

    let mut result = Vector3::new(r, g, b);
    // Capa animada ligera en vivo sobre el mapa estático para mantener fluidez atmosférica.
    if uniforms.time.abs() > 1e-4 {
        let p = fragment.object_pos.normalize();
        let swirl = uniforms.noise.fbm(
            p * 3.5 + time_shift(uniforms.time * WARP2_SPEED),
            2,
            2.0,
            0.5,
        );
        let shift = (swirl - 0.5) * 0.14;
        result = Vector3::new(
            (result.x + shift * 0.4).clamp(0.0, 1.0),
            (result.y + shift * 0.8).clamp(0.0, 1.0),
            (result.z + shift * 0.2).clamp(0.0, 1.0),
        );
    }

    let diffuse = diffuse_factor(fragment.normal, uniforms);
    let mu = surface_mu(fragment, uniforms);
    let limb = 0.22 + 0.78 * mu.powf(0.72);
    result *= AMBIENT + (1.0 - AMBIENT) * diffuse;
    result *= limb;

    let rim = (1.0 - mu).powf(3.0) * RIM_INTENSITY;
    result += Vector3::new(0.18, 0.90, 0.35) * rim;
    result
}

fn animated_vortex_dir(p: Vector3, time: f32) -> Vector3 {
    if time.abs() <= 1e-5 {
        return p;
    }
    let center = Vector3::new(VORTEX_CENTER[0], VORTEX_CENTER[1], VORTEX_CENTER[2]).normalize();
    let distance = (p - center).length();
    let falloff = (-distance / VORTEX_INFLUENCE_RADIUS).exp();
    let angle = -time * (VORTEX_SPIN * 4.0) * falloff;
    let (sin_a, cos_a) = angle.sin_cos();
    let cross = Vector3::new(
        center.y * p.z - center.z * p.y,
        center.z * p.x - center.x * p.z,
        center.x * p.y - center.y * p.x,
    );
    (p * cos_a + cross * sin_a + center * center.dot(p) * (1.0 - cos_a)).normalize()
}

pub fn shade(fragment: &Fragment, uniforms: &Uniforms) -> Vector3 {
    print_statistics_once(&uniforms.noise);
    let sample_dir = animated_vortex_dir(fragment.object_pos.normalize(), uniforms.time);
    let s = if let Some(baked) = uniforms.baked.as_ref() {
        baked.gas_giant.sample_dir(sample_dir)
    } else {
        sample_static(&uniforms.noise, sample_dir)
    };
    shade_from_static(fragment, uniforms, s)
}
