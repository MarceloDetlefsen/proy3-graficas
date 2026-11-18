#![allow(dead_code)]
//! Generador pseudoaleatorio propio (SplitMix64) con semilla reproducible.
//!
//! Provee secuencias deterministas de enteros de 64 bits y flotantes en `[0, 1)`
//! para la distribución procedural de elementos en superficie sin depender de
//! crates externos.

#[derive(Clone, Copy, Debug)]
pub struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    #[inline]
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Devuelve un flotante uniforme en `[0.0, 1.0)`.
    #[inline]
    pub fn next_f32(&mut self) -> f32 {
        ((self.next_u64() >> 40) as f32) / ((1_u32 << 24) as f32)
    }

    #[inline]
    pub fn range_f32(&mut self, min: f32, max: f32) -> f32 {
        min + (max - min) * self.next_f32()
    }
}
