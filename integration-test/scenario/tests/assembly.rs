mod common;

use econbox_worldgen::{WorldConfig, generate};

#[test]
fn a_generated_world_is_mostly_sea_with_islands_in_it() {
    let map = generate(WorldConfig::with_seed(11));

    assert_eq!(map.chunks(), (7, 7));
    assert!((map.water_share() - 0.68).abs() < 0.06);
    assert!(map.land_cells() > 10_000);
}

#[test]
fn the_headless_app_ticks_without_a_window() {
    let app = common::run(8);

    assert!(app.world().entities().len() < 4096);
}
