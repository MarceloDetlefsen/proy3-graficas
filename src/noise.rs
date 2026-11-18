#![allow(dead_code)]

use noise::{Fbm, MultiFractal, NoiseFn, OpenSimplex};
use raylib::prelude::Vector3;
use std::sync::OnceLock;

struct NoiseTools {
    source: Fbm<OpenSimplex>,
}

/// Generador inmutable de ruido de gradiente 3D en `f32` con tabla de permutación.
#[derive(Clone, Debug)]
pub struct Noise {
    perm: [u8; 512],
}

impl Noise {
    pub fn new(seed: u32) -> Self {
        let mut p = [0u8; 256];
        for (i, v) in p.iter_mut().enumerate() {
            *v = i as u8;
        }
        let mut state = seed.wrapping_add(0x9E37_79B9);
        for i in (1..256).rev() {
            state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            let j = ((state >> 16) as usize) % (i + 1);
            p.swap(i, j);
        }
        let mut perm = [0u8; 512];
        for i in 0..512 {
            perm[i] = p[i & 255];
        }
        Self { perm }
    }

    #[inline]
    fn fade(t: f32) -> f32 {
        t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
    }

    #[inline]
    fn grad(hash: u8, x: f32, y: f32, z: f32) -> f32 {
        let h = hash & 15;
        let u = if h < 8 { x } else { y };
        let v = if h < 4 {
            y
        } else if h == 12 || h == 14 {
            x
        } else {
            z
        };
        let a = if (h & 1) == 0 { u } else { -u };
        let b = if (h & 2) == 0 { v } else { -v };
        a + b
    }

    /// Ruido de gradiente 3D normalizado en `[0.0, 1.0]`.
    pub fn noise3(&self, p: Vector3) -> f32 {
        let xi = p.x.floor() as i32;
        let yi = p.y.floor() as i32;
        let zi = p.z.floor() as i32;

        let xf = p.x - xi as f32;
        let yf = p.y - yi as f32;
        let zf = p.z - zi as f32;

        let x0 = (xi & 255) as usize;
        let y0 = (yi & 255) as usize;
        let z0 = (zi & 255) as usize;

        let u = Self::fade(xf);
        let v = Self::fade(yf);
        let w = Self::fade(zf);

        let a = self.perm[x0] as usize + y0;
        let aa = self.perm[a] as usize + z0;
        let ab = self.perm[a + 1] as usize + z0;
        let b = self.perm[x0 + 1] as usize + y0;
        let ba = self.perm[b] as usize + z0;
        let bb = self.perm[b + 1] as usize + z0;

        let x1 = lerp(
            Self::grad(self.perm[aa], xf, yf, zf),
            Self::grad(self.perm[ba], xf - 1.0, yf, zf),
            u,
        );
        let x2 = lerp(
            Self::grad(self.perm[ab], xf, yf - 1.0, zf),
            Self::grad(self.perm[bb], xf - 1.0, yf - 1.0, zf),
            u,
        );
        let y1 = lerp(x1, x2, v);

        let x3 = lerp(
            Self::grad(self.perm[aa + 1], xf, yf, zf - 1.0),
            Self::grad(self.perm[ba + 1], xf - 1.0, yf, zf - 1.0),
            u,
        );
        let x4 = lerp(
            Self::grad(self.perm[ab + 1], xf, yf - 1.0, zf - 1.0),
            Self::grad(self.perm[bb + 1], xf - 1.0, yf - 1.0, zf - 1.0),
            u,
        );
        let y2 = lerp(x3, x4, v);

        (lerp(y1, y2, w) * 0.5 + 0.5).clamp(0.0, 1.0)
    }
}

static NOISE: OnceLock<NoiseTools> = OnceLock::new();

fn noise_tools() -> &'static NoiseTools {
    NOISE.get_or_init(|| NoiseTools {
        source: Fbm::<OpenSimplex>::new(0).set_octaves(1),
    })
}

fn normalize_noise(value: f64) -> f32 {
    ((value * 0.5 + 0.5) as f32).clamp(0.0, 1.0)
}

pub fn noise3(p: Vector3, freq: f32) -> f32 {
    let source = &noise_tools().source;
    normalize_noise(source.get([
        p.x as f64 * freq as f64,
        p.y as f64 * freq as f64,
        p.z as f64 * freq as f64,
    ]))
}

pub fn noise4(p: Vector3, t: f32, freq: f32) -> f32 {
    let source = &noise_tools().source;
    normalize_noise(source.get([
        p.x as f64 * freq as f64,
        p.y as f64 * freq as f64,
        p.z as f64 * freq as f64,
        t as f64,
    ]))
}

fn fractal<F>(octaves: usize, lacunarity: f32, gain: f32, mut sample: F) -> f32
where
    F: FnMut(f32) -> f32,
{
    let octaves = octaves.max(1);
    let lacunarity = lacunarity.max(0.0);
    let gain = gain.max(0.0);
    let mut amplitude = 1.0;
    let mut frequency = 1.0;
    let mut total = 0.0;
    let mut weight = 0.0;

    for _ in 0..octaves {
        total += sample(frequency) * amplitude;
        weight += amplitude;
        frequency *= lacunarity;
        amplitude *= gain;
    }

    if weight > 0.0 {
        (total / weight).clamp(0.0, 1.0)
    } else {
        0.5
    }
}

pub fn fbm(p: Vector3, octaves: usize, lacunarity: f32, gain: f32) -> f32 {
    fractal(octaves, lacunarity, gain, |frequency| noise3(p, frequency))
}

pub fn fbm4(p: Vector3, t: f32, octaves: usize, lacunarity: f32, gain: f32) -> f32 {
    fractal(octaves, lacunarity, gain, |frequency| {
        noise4(p, t, frequency)
    })
}

pub fn ridged(p: Vector3, octaves: usize, lacunarity: f32, gain: f32) -> f32 {
    fractal(octaves, lacunarity, gain, |frequency| {
        1.0 - (noise3(p, frequency) * 2.0 - 1.0).abs()
    })
}

pub fn turbulence(p: Vector3, octaves: usize, lacunarity: f32, gain: f32) -> f32 {
    fractal(octaves, lacunarity, gain, |frequency| {
        (noise3(p, frequency) * 2.0 - 1.0).abs()
    })
}

pub fn clamp01(x: f32) -> f32 {
    x.clamp(0.0, 1.0)
}

pub fn smoothstep(a: f32, b: f32, x: f32) -> f32 {
    let t = clamp01((x - a) / (b - a));
    t * t * (3.0 - 2.0 * t)
}

pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

pub fn mix_color(a: Vector3, b: Vector3, t: f32) -> Vector3 {
    a + (b - a) * t
}

pub fn remap(x: f32, a: f32, b: f32, c: f32, d: f32) -> f32 {
    if (b - a).abs() <= f32::EPSILON {
        c
    } else {
        lerp(c, d, (x - a) / (b - a))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noise_helpers_stay_in_normalized_range() {
        let p = Vector3::new(0.37, -1.2, 2.4);
        for value in [
            noise3(p, 2.0),
            noise4(p, 1.5, 2.0),
            fbm(p, 5, 2.0, 0.5),
            fbm4(p, 1.5, 5, 2.0, 0.5),
            ridged(p, 5, 2.0, 0.5),
            turbulence(p, 5, 2.0, 0.5),
        ] {
            assert!((0.0..=1.0).contains(&value));
        }
    }
}
