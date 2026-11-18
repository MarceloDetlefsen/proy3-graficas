#![allow(dead_code)]

use raylib::prelude::Vector3;
use std::sync::Once;

use crate::Uniforms;
use crate::fragment::Fragment;
use crate::noise::{self, Noise};
use crate::shaders::{diffuse_factor, surface_view};

// Percentil inferior usado para remapear el campo y evitar una masa gris plana.
pub const H_LOW: f32 = 0.280;
// Percentil superior usado para remapear el campo y reservar contraste para vetas.
pub const H_HIGH: f32 = 0.717;
// Factor anisotrópico: valores mayores alargan las vetas paralelas sobre el eje Y.
pub const VEIN_STRETCH: f32 = 5.5;
// Amplitud del domain warp; valores altos curvan pero también pueden disolver vetas.
pub const WARP_AMP: f32 = 0.13;
// Frecuencia del ruido que deforma el dominio.
pub const WARP_FREQ: f32 = 1.25;
// Frecuencia de la máscara de vegetación de baja frecuencia.
pub const BIOME_FREQ: f32 = 3.2;
// Umbral alto que limita la vegetación a regiones aisladas.
pub const BIOME_THRESHOLD: f32 = 0.58;
// Suavidad del umbral de vegetación.
pub const BIOME_SOFTNESS: f32 = 0.05;
// Umbral de las crestas finas que se muestran como grietas.
pub const CRACK_THRESHOLD: f32 = 0.84;
// Suavidad del umbral de grietas.
pub const CRACK_SOFTNESS: f32 = 0.025;
// Octavas máximas para mantener usable el renderer en CPU.
pub const TERRAIN_OCTAVES: usize = 4;
// Agudeza de cada cresta ridged; valores mayores producen vetas más finas.
pub const RIDGE_SHARPNESS: f32 = 4.0;
// Amplitud de la capa de detalle de alta frecuencia.
pub const RIDGE_DETAIL_AMP: f32 = 0.18;
// Frecuencia de la capa de vetas finas; valores mayores producen líneas numerosas.
pub const RIDGE_DETAIL_FREQ: f32 = 5.0;
// Ángulo del eje de estiramiento para que las vetas crucen el disco en diagonal.
pub const VEIN_ANGLE: f32 = 0.52;
// Nivel de altura por debajo del cual aparecen los mares ácidos, como acento pequeño.
pub const SEA_LEVEL: f32 = 0.03;
// Color de las cuencas más bajas.
pub const COLOR_SEA: [f32; 3] = [0.10, 0.75, 0.55];
// Color de sombra cálida de la roca.
pub const COLOR_ROCK_SHADOW: [f32; 3] = [0.25, 0.10, 0.04];
// Color ámbar dominante de la roca.
pub const COLOR_AMBER: [f32; 3] = [0.75, 0.40, 0.08];
// Color dorado de las crestas claras.
pub const COLOR_GOLD: [f32; 3] = [0.98, 0.72, 0.25];
// Color base del bioma Brinstar.
pub const COLOR_TOXIC_GREEN: [f32; 3] = [0.35, 0.85, 0.20];
// Toque rosado mezclado dentro del bioma por ruido fino.
pub const COLOR_PINK: [f32; 3] = [0.90, 0.35, 0.60];
// Color de las grietas, todavía sin emisión propia en esta fase.
pub const COLOR_CRACK: [f32; 3] = [1.00, 0.65, 0.12];
// Iluminación mínima para conservar detalle en el lado oscuro.
pub const AMBIENT: f32 = 0.15;
// Separación de las muestras de altura usadas para estimar el gradiente.
pub const BUMP_DELTA: f32 = 0.008;
// Fuerza del relieve sobre la normal; valores altos hacen más marcado el modelado.
pub const BUMP_STRENGTH: f32 = 0.28;
// Atenuación adicional del gradiente para que el relieve no produzca dientes.
pub const BUMP_GRADIENT_SCALE: f32 = 0.35;
// Exponente del reflejo concentrado de los mares ácidos.
pub const SEA_SPEC_POWER: f32 = 48.0;
// Intensidad máxima del reflejo especular del mar.
pub const SEA_SPEC_STRENGTH: f32 = 0.8;
// Emisión máxima de las grietas en la cara nocturna.
pub const LAVA_GLOW: f32 = 0.75;
// Centros de los tres remolinos locales que curvan las vetas cercanas.
pub const SWIRL_CENTERS: [[f32; 3]; 3] = [
    [-0.35, 0.35, -0.78],
    [0.35, 0.05, -0.82],
    [0.05, -0.55, -0.68],
];
// Ángulo máximo de rotación de cada remolino en radianes.
pub const SWIRL_STRENGTH: f32 = 1.85;
// Radio de influencia de los remolinos en el dominio normalizado.
pub const SWIRL_RADIUS: f32 = 0.42;
// Frecuencia espacial baja de la capa de nubes.
pub const CLOUD_FREQ: f32 = 1.7;
// Velocidad de rotación de las nubes alrededor del eje Y.
pub const CLOUD_SPEED: f32 = 0.28;
// Umbral de cobertura: subirlo deja menos nubes.
pub const CLOUD_COVERAGE: f32 = 0.52;
// Opacidad máxima de las nubes sobre la superficie.
pub const CLOUD_OPACITY: f32 = 0.50;
// Color gris-violeta oscuro de las nubes.
pub const CLOUD_COLOR: [f32; 3] = [0.28, 0.24, 0.38];

static PRINTED_STATS: Once = Once::new();

fn swirl_domain(mut p: Vector3) -> Vector3 {
    for center_values in SWIRL_CENTERS {
        let center = Vector3::new(center_values[0], center_values[1], center_values[2]);
        let offset = p - center;
        let radius = offset.length();
        let falloff = (1.0 - radius / SWIRL_RADIUS).clamp(0.0, 1.0);
        if falloff > 0.0 {
            let angle = SWIRL_STRENGTH * falloff * falloff;
            let (sin_angle, cos_angle) = angle.sin_cos();
            let rotated = Vector3::new(
                offset.x * cos_angle - offset.z * sin_angle,
                offset.y,
                offset.x * sin_angle + offset.z * cos_angle,
            );
            p = center + rotated;
        }
    }
    p
}

fn terrain_domain(noise: &Noise, object_pos: Vector3) -> Vector3 {
    let p = swirl_domain(object_pos.normalize());
    let warp = noise.fbm(p * WARP_FREQ, 3, 2.0, 0.5) * 2.0 - 1.0;
    let (sin_angle, cos_angle) = VEIN_ANGLE.sin_cos();
    let rotated = Vector3::new(
        p.x * cos_angle - p.y * sin_angle,
        p.x * sin_angle + p.y * cos_angle,
        p.z,
    );
    Vector3::new(
        rotated.x + warp * WARP_AMP,
        rotated.y / VEIN_STRETCH + warp * WARP_AMP * 0.5,
        rotated.z + warp * WARP_AMP,
    )
}

fn raw_height_field(noise: &Noise, object_pos: Vector3) -> f32 {
    let warped = terrain_domain(noise, object_pos);
    let broad_ridges = noise.ridged(warped, TERRAIN_OCTAVES, 2.0, 0.5).powf(RIDGE_SHARPNESS);
    let fine_ridges = noise
        .ridged(warped * RIDGE_DETAIL_FREQ, 3, 2.0, 0.5)
        .powf(RIDGE_SHARPNESS);
    (broad_ridges * (1.0 - RIDGE_DETAIL_AMP) + fine_ridges * RIDGE_DETAIL_AMP).clamp(0.0, 1.0)
}

fn fine_ridge_field(noise: &Noise, object_pos: Vector3) -> f32 {
    let warped = terrain_domain(noise, object_pos);
    noise
        .ridged(warped * RIDGE_DETAIL_FREQ, 3, 2.0, 0.5)
        .powf(RIDGE_SHARPNESS)
}

pub fn height_field(noise: &Noise, object_pos: Vector3) -> f32 {
    noise::remap(raw_height_field(noise, object_pos), H_LOW, H_HIGH, 0.0, 1.0).clamp(0.0, 1.0)
}

pub fn biome_mask(noise: &Noise, object_pos: Vector3) -> f32 {
    let value = noise.fbm(object_pos.normalize() * BIOME_FREQ, 3, 2.0, 0.5);
    noise::smoothstep(BIOME_THRESHOLD, BIOME_THRESHOLD + BIOME_SOFTNESS, value)
}

fn sea_mask(noise: &Noise, object_pos: Vector3, height: f32) -> f32 {
    let basin = 1.0 - noise::smoothstep(SEA_LEVEL, SEA_LEVEL + 0.09, height);
    let patches = noise.fbm(object_pos.normalize() * 4.5, 3, 2.0, 0.5);
    basin * noise::smoothstep(0.46, 0.62, patches)
}

pub fn crack_mask(noise: &Noise, object_pos: Vector3) -> f32 {
    let fine_ridge = fine_ridge_field(noise, object_pos);
    noise::smoothstep(
        CRACK_THRESHOLD,
        CRACK_THRESHOLD + CRACK_SOFTNESS,
        fine_ridge,
    )
}

pub fn cloud_mask(noise: &Noise, object_pos: Vector3, time: f32) -> f32 {
    let p = object_pos.normalize();
    let angle = time * CLOUD_SPEED;
    let (sin_angle, cos_angle) = angle.sin_cos();
    let rotated = Vector3::new(
        p.x * cos_angle - p.z * sin_angle,
        p.y,
        p.x * sin_angle + p.z * cos_angle,
    );
    let cloud_noise = noise.fbm(
        rotated * CLOUD_FREQ + Vector3::new(0.0, time * 0.08, time * 0.05),
        3,
        2.0,
        0.5,
    );
    noise::smoothstep(CLOUD_COVERAGE, CLOUD_COVERAGE + 0.14, cloud_noise)
}

fn percentile(sorted: &[f32], fraction: f32) -> f32 {
    sorted[(fraction * (sorted.len() - 1) as f32) as usize]
}

fn print_statistics_once(noise: &Noise) {
    PRINTED_STATS.call_once(|| {
        let sample_count = 4096;
        let golden_angle = std::f32::consts::PI * (3.0 - 5.0_f32.sqrt());
        let mut raw_values = Vec::with_capacity(sample_count);
        let mut biome_values = Vec::with_capacity(sample_count);
        let mut fine_ridges = Vec::with_capacity(sample_count);
        let mut height_values = Vec::with_capacity(sample_count);
        let mut sea_values = Vec::with_capacity(sample_count);

        for i in 0..sample_count {
            let y = 1.0 - (i as f32 + 0.5) * 2.0 / sample_count as f32;
            let radius = (1.0 - y * y).sqrt();
            let angle = golden_angle * i as f32;
            let point = Vector3::new(radius * angle.cos(), y, radius * angle.sin());
            let raw = raw_height_field(noise, point);
            raw_values.push(raw);
            let height = height_field(noise, point);
            height_values.push(height);
            sea_values.push(sea_mask(noise, point, height));
            biome_values.push(biome_mask(noise, point));
            fine_ridges.push(fine_ridge_field(noise, point));
        }

        raw_values.sort_by(f32::total_cmp);
        println!(
            "Rocky h raw: min={:.3}, p02={:.3}, p05={:.3}, p25={:.3}, p50={:.3}, p75={:.3}, p95={:.3}, max={:.3}",
            raw_values[0],
            percentile(&raw_values, 0.02),
            percentile(&raw_values, 0.05),
            percentile(&raw_values, 0.25),
            percentile(&raw_values, 0.50),
            percentile(&raw_values, 0.75),
            percentile(&raw_values, 0.95),
            raw_values[raw_values.len() - 1],
        );
        biome_values.sort_by(f32::total_cmp);
        fine_ridges.sort_by(f32::total_cmp);
        height_values.sort_by(f32::total_cmp);
        sea_values.sort_by(f32::total_cmp);
        println!(
            "Rocky bioma mask: p50={:.3}, p75={:.3}, p95={:.3}, max={:.3}",
            percentile(&biome_values, 0.50),
            percentile(&biome_values, 0.75),
            percentile(&biome_values, 0.95),
            biome_values[biome_values.len() - 1],
        );
        println!(
            "Rocky grietas field: p90={:.3}, p95={:.3}, p97={:.3}, p98={:.3}, p99={:.3}, max={:.3}",
            percentile(&fine_ridges, 0.90),
            percentile(&fine_ridges, 0.95),
            percentile(&fine_ridges, 0.97),
            percentile(&fine_ridges, 0.98),
            percentile(&fine_ridges, 0.99),
            fine_ridges[fine_ridges.len() - 1],
        );
        let biome_coverage = biome_values.iter().filter(|&&value| value > 0.5).count();
        let crack_coverage = fine_ridges
            .iter()
            .filter(|&&value| noise::smoothstep(CRACK_THRESHOLD, CRACK_THRESHOLD + CRACK_SOFTNESS, value) > 0.5)
            .count();
        let sea_coverage = sea_values.iter().filter(|&&value| value > 0.5).count();
        let low_height_coverage = height_values.iter().filter(|&&value| value < 0.2).count();
        println!(
            "Rocky cobertura: mar={:.1}%, bioma={:.1}%, grietas={:.1}%, h<0.2={:.1}%",
            sea_coverage as f32 * 100.0 / sample_count as f32,
            biome_coverage as f32 * 100.0 / sample_count as f32,
            crack_coverage as f32 * 100.0 / sample_count as f32,
            low_height_coverage as f32 * 100.0 / sample_count as f32,
        );
    });
}

fn color(values: [f32; 3]) -> Vector3 {
    Vector3::new(values[0], values[1], values[2])
}

fn height_color(height: f32) -> Vector3 {
    if height < 0.45 {
        noise::mix_color(
            color(COLOR_ROCK_SHADOW),
            color(COLOR_AMBER),
            noise::smoothstep(SEA_LEVEL, 0.45, height),
        )
    } else {
        noise::mix_color(
            color(COLOR_AMBER),
            color(COLOR_GOLD),
            noise::smoothstep(0.45, 1.0, height),
        )
    }
}

pub fn sample_static(noise: &Noise, dir: Vector3) -> [f32; 7] {
    let p = dir.normalize();
    let height = height_field(noise, p);
    let sea = sea_mask(noise, p, height);
    let biome = biome_mask(noise, p);
    let crack = crack_mask(noise, p);
    let base = height_color(height);
    let with_sea = noise::mix_color(base, color(COLOR_SEA), sea);
    let pink_noise = noise.fbm(p * 8.0, 3, 2.0, 0.5);
    let pink_amount = noise::smoothstep(0.55, 0.72, pink_noise);
    let biome_color = noise::mix_color(color(COLOR_TOXIC_GREEN), color(COLOR_PINK), pink_amount);
    let with_biome = noise::mix_color(with_sea, biome_color, biome);
    let albedo = noise::mix_color(with_biome, color(COLOR_CRACK), crack);
    [albedo.x, albedo.y, albedo.z, height, sea, crack, biome]
}

fn tangent_pair(p: Vector3) -> (Vector3, Vector3) {
    let ref_axis = if p.y.abs() < 0.9 {
        Vector3::new(0.0, 1.0, 0.0)
    } else {
        Vector3::new(1.0, 0.0, 0.0)
    };
    let t1 = ref_axis.cross(p).normalize();
    let t2 = p.cross(t1).normalize();
    (t1, t2)
}

fn bumped_normal_from_heights(
    fragment: &Fragment,
    uniforms: &Uniforms,
    height: f32,
    h1: f32,
    h2: f32,
) -> Vector3 {
    let p = fragment.object_pos.normalize();
    let (t1, t2) = tangent_pair(p);
    let g1 = (h1 - height) / BUMP_DELTA;
    let g2 = (h2 - height) / BUMP_DELTA;
    let gradient = (t1 * g1 + t2 * g2) * BUMP_GRADIENT_SCALE;
    // La altura aumenta en la dirección del gradiente; la normal de una superficie
    // elevada se inclina contra él. Este signo mantiene las crestas claras hacia la luz.
    let bump_fade = noise::smoothstep(0.12, 0.48, crate::shaders::surface_mu(fragment, uniforms));
    let object_normal = (p - gradient * BUMP_STRENGTH * bump_fade).normalize();
    uniforms.model_matrix.transform_normal(object_normal)
}

fn lit_cloud_color(fragment: &Fragment, uniforms: &Uniforms) -> Vector3 {
    let diffuse = diffuse_factor(fragment.normal, uniforms);
    color(CLOUD_COLOR) * (AMBIENT + (1.0 - AMBIENT) * diffuse)
}

pub fn shade_from_static(
    fragment: &Fragment,
    uniforms: &Uniforms,
    s: [f32; 7],
    bump_heights: (f32, f32),
) -> Vector3 {
    let [r, g, b, height, sea, crack, biome] = s;
    match uniforms.rocky_debug_view {
        1 => Vector3::new(biome, biome, biome),
        2 => Vector3::new(crack, crack, crack),
        3 => {
            let base = Vector3::new(r, g, b);
            let normal = if sea > 0.5 {
                fragment.normal.normalize()
            } else {
                bumped_normal_from_heights(fragment, uniforms, height, bump_heights.0, bump_heights.1)
            };
            let diffuse = diffuse_factor(normal, uniforms);
            let mut surface = base * (AMBIENT + (1.0 - AMBIENT) * diffuse);

            let view = surface_view(fragment, uniforms);
            let half_vector = (uniforms.light_dir + view).normalize();
            let specular =
                normal.dot(half_vector).max(0.0).powf(SEA_SPEC_POWER) * SEA_SPEC_STRENGTH * sea;
            surface += Vector3::new(1.0, 1.0, 1.0) * specular;

            let n_dot_l = diffuse_factor(fragment.normal, uniforms);
            let lava = color(COLOR_CRACK) * (crack * LAVA_GLOW * (1.0 - n_dot_l));
            surface += lava;

            let clouds = cloud_mask(&uniforms.noise, fragment.object_pos, uniforms.time);
            let cloud_alpha = clouds * CLOUD_OPACITY;
            let cloud_color = lit_cloud_color(fragment, uniforms);
            let mut result = noise::mix_color(surface, cloud_color, cloud_alpha);
            result += lava * (clouds * 0.18);
            result
        }
        _ => Vector3::new(height, height, height),
    }
}

pub fn shade(fragment: &Fragment, uniforms: &Uniforms) -> Vector3 {
    print_statistics_once(&uniforms.noise);
    let p = fragment.object_pos.normalize();
    let (t1, t2) = tangent_pair(p);
    if let Some(baked) = uniforms.baked.as_ref() {
        let s = baked.rocky.sample_dir(p);
        let h1 = baked.rocky.sample_dir(p + t1 * BUMP_DELTA)[3];
        let h2 = baked.rocky.sample_dir(p + t2 * BUMP_DELTA)[3];
        shade_from_static(fragment, uniforms, s, (h1, h2))
    } else {
        let s = sample_static(&uniforms.noise, p);
        let h1 = height_field(&uniforms.noise, p + t1 * BUMP_DELTA);
        let h2 = height_field(&uniforms.noise, p + t2 * BUMP_DELTA);
        shade_from_static(fragment, uniforms, s, (h1, h2))
    }
}
