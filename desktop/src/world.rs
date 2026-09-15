//! What one tile means, and what the whole map adds up to.

use crate::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Land {
    pub pos: TilePos,
    pub tile: Tile,
}

impl Land {
    pub fn at(map: &TileMap, pos: TilePos) -> Option<Self> {
        map.get(pos).map(|tile| Self { pos, tile })
    }

    pub fn is_settleable(&self) -> bool {
        self.tile.is_land() && self.tile.fertility() > 0.0
    }

    pub fn carrying_capacity(&self) -> f32 {
        if self.tile.ground.is_high() {
            self.tile.fertility() * 0.7
        } else {
            self.tile.fertility()
        }
    }

    pub fn chunk(&self) -> (u32, u32) {
        self.pos.chunk()
    }
}

#[derive(Resource, Debug, Clone, Copy, PartialEq, Default)]
pub struct Survey {
    pub cells: usize,
    pub land: usize,
    pub settleable: usize,
}

impl Survey {
    pub fn of(map: &TileMap) -> Self {
        let mut land = 0;
        let mut settleable = 0;

        for tile in &map.tiles {
            if tile.is_land() {
                land += 1;

                if tile.fertility() > 0.0 {
                    settleable += 1;
                }
            }
        }

        Self {
            cells: map.cells(),
            land,
            settleable,
        }
    }

    pub fn water(&self) -> usize {
        self.cells.saturating_sub(self.land)
    }

    pub fn water_share(&self) -> f32 {
        self.water() as f32 / self.cells.max(1) as f32
    }

    pub fn land_share(&self) -> f32 {
        self.land as f32 / self.cells.max(1) as f32
    }

    pub fn settleable_share(&self) -> f32 {
        self.settleable as f32 / self.cells.max(1) as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn grass() -> Tile {
        Tile::new(Ground::SoilLow, Overlay::Grass)
    }

    fn ocean(width: u32, height: u32) -> TileMap {
        TileMap::filled(width, height, Tile::bare(Ground::DeepOcean))
    }

    #[test]
    fn a_tile_off_the_map_is_no_land_at_all() {
        let map = ocean(16, 16);

        assert!(Land::at(&map, TilePos::new(9999, 9999)).is_none());
        assert!(Land::at(&map, TilePos::new(15, 15)).is_some());
    }

    #[test]
    fn deep_water_supports_nobody() {
        let map = ocean(16, 16);
        let land = Land::at(&map, TilePos::new(5, 5)).expect("a tile");

        assert_eq!(land.tile.name(), "deep_ocean");
        assert_eq!(land.carrying_capacity(), 0.0);
        assert!(!land.is_settleable());
    }

    #[test]
    fn a_grass_tile_keeps_its_worldbox_name_and_chunk() {
        let mut map = ocean(128, 128);
        map.set(100, 70, grass());
        let land = Land::at(&map, TilePos::new(100, 70)).expect("a tile");

        assert_eq!(land.tile.name(), "soil_low:grass_low");
        assert_eq!(land.chunk(), (1, 1));
        assert!(land.is_settleable());
    }

    #[test]
    fn high_ground_carries_less_than_low_ground() {
        let low = Land {
            pos: TilePos::new(0, 0),
            tile: grass(),
        };
        let high = Land {
            pos: TilePos::new(0, 0),
            tile: Tile::new(Ground::SoilHigh, Overlay::Grass),
        };

        assert!(low.carrying_capacity() > high.carrying_capacity());
    }

    #[test]
    fn a_survey_counts_what_the_map_holds() {
        let mut map = ocean(10, 10);

        for x in 0..30 {
            map.set(x % 10, x / 10, grass());
        }

        let survey = Survey::of(&map);

        assert_eq!(survey.cells, 100);
        assert_eq!(survey.land, 30);
        assert_eq!(survey.settleable, 30);
        assert_eq!(survey.water(), 70);
        assert!((survey.land_share() - 0.3).abs() < f32::EPSILON);
        assert!((survey.water_share() - 0.7).abs() < f32::EPSILON);
        assert!((survey.settleable_share() - 0.3).abs() < f32::EPSILON);
    }

    #[test]
    fn an_empty_survey_does_not_divide_by_zero() {
        let survey = Survey::default();

        assert_eq!(survey.water(), 0);
        assert_eq!(survey.land_share(), 0.0);
        assert_eq!(survey.water_share(), 0.0);
    }
}
