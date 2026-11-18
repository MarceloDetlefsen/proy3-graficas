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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_repeats_sequence_and_different_seeds_diverge() {
        let mut a = SplitMix64::new(12345);
        let mut b = SplitMix64::new(12345);
        let mut c = SplitMix64::new(54321);

        let mut diffs = 0usize;
        for _ in 0..64 {
            let va = a.next_u64();
            let vb = b.next_u64();
            let vc = c.next_u64();
            assert_eq!(va, vb);
            if va != vc {
                diffs += 1;
            }
            let f = a.next_f32();
            assert!((0.0..1.0).contains(&f));
            let _ = b.next_f32();
            let _ = c.next_f32();
        }
        assert!(diffs > 60);
    }
}
