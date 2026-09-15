//! Turns the pointer into the tile it is resting on.

use bevy::prelude::*;
use econbox_camera::{TILE_SIZE, WorldCamera};
use econbox_tile::Tile;
use econbox_tilemap::{TileMap, TilePos};

#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct Hovered {
    pub pos: Option<TilePos>,
    pub tile: Option<Tile>,
}

pub struct CursorPlugin;

impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Hovered>()
            .add_systems(Update, track.run_if(resource_exists::<TileMap>));
    }
}

pub fn tile_at(map: &TileMap, world: Vec2) -> Option<TilePos> {
    let half = Vec2::new(map.width as f32, map.height as f32) * TILE_SIZE * 0.5;
    let local = world + half;

    if local.x < 0.0 || local.y < 0.0 {
        return None;
    }

    let x = (local.x / TILE_SIZE) as u32;
    let y = map
        .height
        .checked_sub(1 + (local.y / TILE_SIZE) as u32)
        .unwrap_or(u32::MAX);

    (x < map.width && y < map.height).then_some(TilePos::new(x, y))
}

fn track(
    map: Res<TileMap>,
    windows: Query<&Window>,
    camera: Option<Single<(&Camera, &GlobalTransform), With<WorldCamera>>>,
    mut hovered: ResMut<Hovered>,
) {
    let Some(camera) = camera else { return };
    let (camera, transform) = camera.into_inner();

    let found = windows
        .iter()
        .find_map(|window| window.cursor_position())
        .and_then(|cursor| camera.viewport_to_world_2d(transform, cursor).ok())
        .and_then(|world| tile_at(&map, world));

    hovered.pos = found;
    hovered.tile = found.and_then(|pos| map.get(pos));
}

#[cfg(test)]
mod tests {
    use super::*;
    use econbox_tile::{Ground, Tile};

    fn map() -> TileMap {
        TileMap::filled(448, 448, Tile::bare(Ground::DeepOcean))
    }

    #[test]
    fn the_centre_of_the_screen_is_the_middle_of_the_map() {
        let map = map();
        let pos = tile_at(&map, Vec2::ZERO).expect("inside");

        assert_eq!(pos, TilePos::new(224, 223));
    }

    #[test]
    fn the_top_left_tile_sits_at_the_top_left_corner() {
        let map = map();
        let half = 448.0 * TILE_SIZE * 0.5;
        let pos = tile_at(&map, Vec2::new(-half + 1.0, half - 1.0)).expect("inside");

        assert_eq!(pos, TilePos::new(0, 0));
    }

    #[test]
    fn the_bottom_right_tile_sits_at_the_bottom_right_corner() {
        let map = map();
        let half = 448.0 * TILE_SIZE * 0.5;
        let pos = tile_at(&map, Vec2::new(half - 1.0, -half + 1.0)).expect("inside");

        assert_eq!(pos, TilePos::new(447, 447));
    }

    #[test]
    fn pointing_off_the_map_finds_nothing() {
        let map = map();
        let half = 448.0 * TILE_SIZE * 0.5;

        assert!(tile_at(&map, Vec2::new(-half - 10.0, 0.0)).is_none());
        assert!(tile_at(&map, Vec2::new(0.0, half + 10.0)).is_none());
        assert!(tile_at(&map, Vec2::new(half + 10.0, 0.0)).is_none());
    }

    #[test]
    fn every_tile_can_be_pointed_at() {
        let map = TileMap::filled(16, 16, Tile::default());
        let half = 16.0 * TILE_SIZE * 0.5;

        for y in 0..16 {
            for x in 0..16 {
                let world = Vec2::new(
                    -half + (x as f32 + 0.5) * TILE_SIZE,
                    half - (y as f32 + 0.5) * TILE_SIZE,
                );

                assert_eq!(tile_at(&map, world), Some(TilePos::new(x, y)), "{x},{y}");
            }
        }
    }
}
