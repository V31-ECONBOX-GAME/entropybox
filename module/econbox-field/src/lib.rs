//! Scalar fields the generator bands into tiles: height, temperature, moisture.

use econbox_noise::{Fbm, Noise};

#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    pub width: u32,
    pub height: u32,
    pub values: Vec<f32>,
}

impl Field {
    pub fn filled(width: u32, height: u32, value: f32) -> Self {
        Self {
            width,
            height,
            values: vec![value; (width * height) as usize],
        }
    }

    pub fn index(&self, x: u32, y: u32) -> usize {
        (y.min(self.height - 1) * self.width + x.min(self.width - 1)) as usize
    }

    pub fn at(&self, x: u32, y: u32) -> f32 {
        self.values[self.index(x, y)]
    }

    pub fn quantile(&self, share: f32) -> f32 {
        const BUCKETS: usize = 4096;

        let mut histogram = [0_u32; BUCKETS];
        for value in &self.values {
            histogram[(value.clamp(0.0, 1.0) * (BUCKETS - 1) as f32) as usize] += 1;
        }

        let target = (self.values.len() as f32 * share.clamp(0.0, 1.0)) as u32;
        let mut seen = 0;

        for (bucket, count) in histogram.iter().enumerate() {
            seen += count;
            if seen >= target {
                return bucket as f32 / (BUCKETS - 1) as f32;
            }
        }

        1.0
    }

    pub fn share_below(&self, level: f32) -> f32 {
        let below = self.values.iter().filter(|value| **value < level).count();
        below as f32 / self.values.len().max(1) as f32
    }
}

#[derive(Debug, Clone, Copy)]
pub struct HeightConfig {
    pub seed: u32,
    pub continent_scale: f32,
    pub detail_scale: f32,
    pub warp: f32,
    pub detail_weight: f32,
    pub edge_falloff: f32,
    pub edge_depth: f32,
    pub contrast: f32,
    pub ridge_scale: f32,
    pub ridge_weight: f32,
}

impl Default for HeightConfig {
    fn default() -> Self {
        Self {
            seed: 0x_5eed_1234,
            continent_scale: 5.5,
            detail_scale: 17.0,
            warp: 0.85,
            detail_weight: 0.34,
            edge_falloff: 0.22,
            edge_depth: 0.55,
            contrast: 1.45,
            ridge_scale: 9.0,
            ridge_weight: 0.42,
        }
    }
}

pub fn height(width: u32, height: u32, config: HeightConfig) -> Field {
    let shape = Noise::new(config.seed);
    let detail = Noise::new(config.seed).offset(17);
    let ridge = Noise::new(config.seed).offset(53);
    let broad = Fbm {
        octaves: 7,
        ..default_fbm()
    };
    let fine = Fbm {
        octaves: 5,
        ..default_fbm()
    };

    let mut values = Vec::with_capacity((width * height) as usize);

    for y in 0..height {
        let v = y as f32 / height as f32;

        for x in 0..width {
            let u = x as f32 / width as f32;

            let land = shape.warped(
                u * config.continent_scale,
                v * config.continent_scale,
                broad,
                config.warp,
            );
            let grain = detail.fbm(u * config.detail_scale, v * config.detail_scale, fine);
            let raw = land * (1.0 - config.detail_weight) + grain * config.detail_weight;
            let lifted = ((raw - 0.5) * config.contrast + 0.5).clamp(0.0, 1.0);
            let spine = ridge.ridged(u * config.ridge_scale, v * config.ridge_scale, fine);
            let peaked = lifted + (spine - 0.5) * lifted.powi(3) * config.ridge_weight;
            let penalty = (1.0 - coast(u, v, config.edge_falloff)) * config.edge_depth;

            values.push((peaked - penalty).clamp(0.0, 1.0));
        }
    }

    Field {
        width,
        height,
        values,
    }
}

fn coast(u: f32, v: f32, falloff: f32) -> f32 {
    let edge = u.min(1.0 - u).min(v).min(1.0 - v) / falloff.max(f32::EPSILON);

    smoothstep(edge.clamp(0.0, 1.0))
}

pub fn temperature(height_field: &Field, equator_bias: f32) -> Field {
    let mut values = Vec::with_capacity(height_field.values.len());

    for y in 0..height_field.height {
        let from_equator = (y as f32 / height_field.height as f32 - 0.5).abs() * 2.0;
        let band = 1.0 - from_equator.powf(equator_bias.max(0.1));

        for x in 0..height_field.width {
            let lapse = (height_field.at(x, y) - 0.5).max(0.0) * 0.9;
            values.push((band - lapse).clamp(0.0, 1.0));
        }
    }

    Field {
        width: height_field.width,
        height: height_field.height,
        values,
    }
}

pub fn moisture(width: u32, height: u32, seed: u32, scale: f32) -> Field {
    let noise = Noise::new(seed).offset(91);
    let fbm = Fbm {
        octaves: 5,
        ..default_fbm()
    };
    let mut values = Vec::with_capacity((width * height) as usize);

    for y in 0..height {
        for x in 0..width {
            let u = x as f32 / width as f32 * scale;
            let v = y as f32 / height as f32 * scale;
            values.push(noise.fbm(u, v, fbm).clamp(0.0, 1.0));
        }
    }

    Field {
        width,
        height,
        values,
    }
}

fn default_fbm() -> Fbm {
    Fbm::default()
}

fn smoothstep(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_edges_of_the_map_sink_below_the_waterline() {
        let field = height(256, 256, HeightConfig::default());
        let sea = field.quantile(0.68);

        for step in 0..256 {
            for (x, y) in [(step, 0), (step, 255), (0, step), (255, step)] {
                assert!(
                    field.at(x, y) < sea,
                    "{x},{y} at {} vs sea {sea}",
                    field.at(x, y)
                );
            }
        }
    }

    #[test]
    fn the_middle_of_the_map_rises_above_the_sea() {
        let field = height(256, 256, HeightConfig::default());
        let middle: Vec<_> = (96..160)
            .flat_map(|y| (96..160).map(move |x| (x, y)))
            .map(|(x, y)| field.at(x, y))
            .collect();
        let highest = middle.iter().copied().fold(f32::MIN, f32::max);

        assert!(highest > 0.4, "highest {highest}");
    }

    #[test]
    fn the_same_seed_builds_the_same_world() {
        let config = HeightConfig::default();

        assert_eq!(height(128, 128, config), height(128, 128, config));
    }

    #[test]
    fn a_different_seed_builds_a_different_world() {
        let a = height(128, 128, HeightConfig::default());
        let b = height(
            128,
            128,
            HeightConfig {
                seed: 99,
                ..HeightConfig::default()
            },
        );

        assert_ne!(a, b);
    }

    #[test]
    fn the_quantile_cuts_the_share_it_is_asked_for() {
        let field = height(256, 256, HeightConfig::default());

        for share in [0.4_f32, 0.6, 0.7, 0.85] {
            let level = field.quantile(share);
            let actual = field.share_below(level);

            assert!((actual - share).abs() < 0.05, "{share} -> {actual}");
        }
    }

    #[test]
    fn it_is_colder_at_the_poles_than_at_the_equator() {
        let field = height(128, 128, HeightConfig::default());
        let warmth = temperature(&field, 1.4);

        assert!(warmth.at(64, 64) > warmth.at(64, 2));
        assert!(warmth.at(64, 64) > warmth.at(64, 125));
    }

    #[test]
    fn moisture_covers_the_whole_range_without_leaving_it() {
        let damp = moisture(256, 256, 5, 4.0);
        let low = damp.values.iter().copied().fold(f32::MAX, f32::min);
        let high = damp.values.iter().copied().fold(f32::MIN, f32::max);

        assert!(low >= 0.0 && high <= 1.0);
        assert!(high - low > 0.3, "range {low}..{high}");
    }
}
