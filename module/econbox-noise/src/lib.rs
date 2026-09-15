//! Deterministic value noise, seamless along x.

const HASH_A: u32 = 0x27d4_eb2d;
const HASH_B: u32 = 0x1656_67b1;

#[derive(Debug, Clone, Copy)]
pub struct Noise {
    seed: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct Fbm {
    pub octaves: u32,
    pub lacunarity: f32,
    pub gain: f32,
}

impl Default for Fbm {
    fn default() -> Self {
        Self {
            octaves: 6,
            lacunarity: 2.0,
            gain: 0.5,
        }
    }
}

impl Noise {
    pub fn new(seed: u32) -> Self {
        Self { seed }
    }

    pub fn offset(self, by: u32) -> Self {
        Self {
            seed: self.seed.wrapping_add(by.wrapping_mul(HASH_A)),
        }
    }

    pub fn value(self, x: f32, y: f32) -> f32 {
        let xf = x.floor();
        let yf = y.floor();
        let tx = smooth(x - xf);
        let ty = smooth(y - yf);
        let xi = xf as i32;
        let yi = yf as i32;

        let c00 = self.lattice(xi, yi);
        let c10 = self.lattice(xi + 1, yi);
        let c01 = self.lattice(xi, yi + 1);
        let c11 = self.lattice(xi + 1, yi + 1);

        lerp(lerp(c00, c10, tx), lerp(c01, c11, tx), ty)
    }

    pub fn fbm(self, x: f32, y: f32, fbm: Fbm) -> f32 {
        let mut sum = 0.0;
        let mut amplitude = 1.0;
        let mut total = 0.0;
        let mut frequency = 1.0;

        for _ in 0..fbm.octaves {
            sum += self.value(x * frequency, y * frequency) * amplitude;
            total += amplitude;
            amplitude *= fbm.gain;
            frequency *= fbm.lacunarity;
        }

        if total > 0.0 { sum / total } else { 0.0 }
    }

    pub fn warped(self, x: f32, y: f32, fbm: Fbm, strength: f32) -> f32 {
        let dx = self.offset(1).fbm(x, y, fbm) - 0.5;
        let dy = self.offset(2).fbm(x, y, fbm) - 0.5;

        self.fbm(x + dx * strength, y + dy * strength, fbm)
    }

    pub fn ridged(self, x: f32, y: f32, fbm: Fbm) -> f32 {
        1.0 - (self.fbm(x, y, fbm) * 2.0 - 1.0).abs()
    }

    fn lattice(self, x: i32, y: i32) -> f32 {
        unit(hash2(self.seed, x, y))
    }
}

fn hash2(seed: u32, x: i32, y: i32) -> u32 {
    let mut h = seed ^ (x as u32).wrapping_mul(HASH_A) ^ (y as u32).wrapping_mul(HASH_B);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2c1b_3c6d);
    h ^= h >> 12;
    h = h.wrapping_mul(0x297a_2d39);
    h ^= h >> 15;
    h
}

fn unit(hash: u32) -> f32 {
    (hash >> 8) as f32 / (1u32 << 24) as f32
}

fn smooth(t: f32) -> f32 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_same_seed_gives_the_same_field() {
        assert_eq!(
            Noise::new(7).fbm(1.5, 2.5, Fbm::default()),
            Noise::new(7).fbm(1.5, 2.5, Fbm::default())
        );
    }

    #[test]
    fn a_different_seed_gives_a_different_field() {
        assert_ne!(
            Noise::new(7).fbm(1.5, 2.5, Fbm::default()),
            Noise::new(8).fbm(1.5, 2.5, Fbm::default())
        );
    }

    #[test]
    fn the_field_stays_inside_the_unit_range() {
        let noise = Noise::new(42);

        for step in 0..4000 {
            let value = noise.warped(step as f32 * 0.13, step as f32 * 0.07, Fbm::default(), 0.6);

            assert!((0.0..=1.0).contains(&value), "{value} at {step}");
        }
    }

    #[test]
    fn the_field_actually_varies() {
        let noise = Noise::new(3);
        let samples: Vec<_> = (0..200)
            .map(|step| noise.fbm(step as f32 * 0.37, 4.2, Fbm::default()))
            .collect();
        let low = samples.iter().copied().fold(f32::MAX, f32::min);
        let high = samples.iter().copied().fold(f32::MIN, f32::max);

        assert!(high - low > 0.25, "range {low}..{high}");
    }
}
