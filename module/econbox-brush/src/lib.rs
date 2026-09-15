//! The painting tools: what the pointer puts on the map, and how to take it back.

use bevy::prelude::*;
use econbox_art::Icon;
use econbox_camera::Panning;
use econbox_creature::{Spawn, Species};
use econbox_cursor::Hovered;
use econbox_tile::{Ground, Overlay, Tile};
use econbox_tilemap::{Painted, TileMap, TilePos};
use std::collections::{HashSet, VecDeque};

pub const LADDER: [Ground; 8] = [
    Ground::DeepOcean,
    Ground::CloseOcean,
    Ground::ShallowWaters,
    Ground::Sand,
    Ground::SoilLow,
    Ground::SoilHigh,
    Ground::Hills,
    Ground::Mountains,
];

pub const MIN_RADIUS: u32 = 0;
pub const MAX_RADIUS: u32 = 24;
pub const FLOOD_LIMIT: usize = 60_000;
pub const UNDO_LIMIT: usize = 64;
pub const SPAWN_GAP: f32 = 0.12;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Shape {
    #[default]
    Round,
    Square,
}

impl Shape {
    pub fn flipped(self) -> Self {
        match self {
            Self::Round => Self::Square,
            Self::Square => Self::Round,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Round => "round",
            Self::Square => "square",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Tool {
    #[default]
    Inspect,
    Ground(Ground),
    Overlay(Overlay),
    Raise,
    Lower,
    Strip,
    Spawn(Species),
}

impl Tool {
    pub fn edits(self) -> bool {
        !matches!(self, Self::Inspect | Self::Spawn(_))
    }

    pub fn holds(self) -> bool {
        !matches!(self, Self::Inspect)
    }

    pub fn name(self) -> String {
        match self {
            Self::Inspect => "inspect".to_string(),
            Self::Ground(ground) => ground.name().to_string(),
            Self::Overlay(overlay) => overlay.name().to_string(),
            Self::Raise => "raise".to_string(),
            Self::Lower => "lower".to_string(),
            Self::Strip => "strip".to_string(),
            Self::Spawn(species) => species.name().to_string(),
        }
    }

    pub fn swatch(self) -> Option<[u8; 3]> {
        match self {
            Self::Ground(ground) => Some(ground.color()),
            Self::Overlay(overlay) => overlay.color(false),
            Self::Spawn(species) => Some(species.color()),
            _ => None,
        }
    }

    pub fn icon(self) -> Option<(Icon, Option<[u8; 3]>)> {
        Some(match self {
            Self::Ground(ground) if ground.is_water() => (Icon::Droplet, Some(ground.color())),
            Self::Ground(ground) => (Icon::Block, Some(ground.color())),
            Self::Overlay(overlay) => (Icon::Sprout, overlay.color(false)),
            Self::Raise => (Icon::Raise, None),
            Self::Lower => (Icon::Lower, None),
            Self::Strip => (Icon::Eraser, None),
            Self::Inspect => (Icon::Lens, None),
            Self::Spawn(_) => return None,
        })
    }

    pub fn applied(self, tile: Tile) -> Option<Tile> {
        let next = match self {
            Self::Inspect | Self::Spawn(_) => return None,
            Self::Ground(ground) => Tile::new(ground, tile.overlay),
            Self::Overlay(overlay) => Tile::new(tile.ground, overlay),
            Self::Raise => Tile::new(step(tile.ground, 1), tile.overlay),
            Self::Lower => Tile::new(step(tile.ground, -1), tile.overlay),
            Self::Strip => Tile::bare(tile.ground),
        };

        (next != tile).then_some(next)
    }
}

pub fn step(ground: Ground, by: i32) -> Ground {
    let Some(rung) = LADDER.iter().position(|&rung| rung == ground) else {
        return ground;
    };
    let next = (rung as i32 + by).clamp(0, LADDER.len() as i32 - 1);

    LADDER[next as usize]
}

pub fn cells(map: &TileMap, centre: TilePos, shape: Shape, radius: u32) -> Vec<TilePos> {
    let reach = radius.clamp(MIN_RADIUS, MAX_RADIUS) as i32;
    let mut out = Vec::new();

    for dy in -reach..=reach {
        for dx in -reach..=reach {
            if shape == Shape::Round && dx * dx + dy * dy > reach * reach {
                continue;
            }

            let x = centre.x as i32 + dx;
            let y = centre.y as i32 + dy;

            if x < 0 || y < 0 || x as u32 >= map.width || y as u32 >= map.height {
                continue;
            }

            out.push(TilePos::new(x as u32, y as u32));
        }
    }

    out
}

pub fn flood(map: &TileMap, origin: TilePos) -> Vec<TilePos> {
    let Some(start) = map.get(origin) else {
        return Vec::new();
    };

    let mut seen = vec![false; map.cells()];
    let mut queue = VecDeque::from([origin]);
    let mut out = Vec::new();
    seen[map.index(origin.x, origin.y)] = true;

    while let Some(pos) = queue.pop_front() {
        out.push(pos);

        if out.len() >= FLOOD_LIMIT {
            break;
        }

        for (dx, dy) in [(1_i32, 0_i32), (-1, 0), (0, 1), (0, -1)] {
            let x = pos.x as i32 + dx;
            let y = pos.y as i32 + dy;

            if x < 0 || y < 0 || x as u32 >= map.width || y as u32 >= map.height {
                continue;
            }

            let (x, y) = (x as u32, y as u32);
            let index = map.index(x, y);

            if seen[index] || map.at(x, y).ground != start.ground {
                continue;
            }

            seen[index] = true;
            queue.push_back(TilePos::new(x, y));
        }
    }

    out
}

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Brush {
    pub tool: Tool,
    pub shape: Shape,
    pub radius: u32,
    pub flood: bool,
}

impl Default for Brush {
    fn default() -> Self {
        Self {
            tool: Tool::default(),
            shape: Shape::default(),
            radius: 3,
            flood: false,
        }
    }
}

impl Brush {
    pub fn grow(&mut self) {
        self.radius = (self.radius + 1).min(MAX_RADIUS);
    }

    pub fn shrink(&mut self) {
        self.radius = self.radius.saturating_sub(1);
    }

    pub fn label(&self) -> String {
        format!(
            "{} {}{}",
            self.shape.name(),
            self.radius,
            if self.flood { " flood" } else { "" }
        )
    }

    pub fn targets(&self, map: &TileMap, centre: TilePos) -> Vec<TilePos> {
        if self.flood {
            flood(map, centre)
        } else {
            cells(map, centre, self.shape, self.radius)
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Stroke {
    pub before: Vec<(TilePos, Tile)>,
}

#[derive(Debug, Default)]
struct Open {
    stroke: Stroke,
    seen: HashSet<(u32, u32)>,
}

#[derive(Resource, Debug, Default)]
pub struct UndoStack {
    done: Vec<Stroke>,
    open: Option<Open>,
}

impl UndoStack {
    pub fn begin(&mut self) {
        if self.open.is_none() {
            self.open = Some(Open::default());
        }
    }

    pub fn record(&mut self, pos: TilePos, tile: Tile) {
        let Some(open) = &mut self.open else { return };

        if open.seen.insert((pos.x, pos.y)) {
            open.stroke.before.push((pos, tile));
        }
    }

    pub fn commit(&mut self) {
        let Some(open) = self.open.take() else { return };

        if open.stroke.before.is_empty() {
            return;
        }

        self.done.push(open.stroke);

        if self.done.len() > UNDO_LIMIT {
            self.done.remove(0);
        }
    }

    pub fn take(&mut self) -> Option<Stroke> {
        self.done.pop()
    }

    pub fn depth(&self) -> usize {
        self.done.len()
    }
}

#[derive(Message, Debug, Clone, Copy)]
pub struct Undo;

#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct PointerOverUi(pub bool);

#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct SpawnRest(pub f32);

pub struct BrushPlugin;

impl Plugin for BrushPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Brush>()
            .init_resource::<UndoStack>()
            .init_resource::<PointerOverUi>()
            .init_resource::<SpawnRest>()
            .add_message::<Painted>()
            .add_message::<Undo>()
            .add_systems(
                Update,
                (hold_pan, paint, drop_in, rewind)
                    .chain()
                    .run_if(resource_exists::<TileMap>),
            );
    }
}

fn hold_pan(brush: Res<Brush>, mut panning: ResMut<Panning>) {
    let with_left = !brush.tool.holds();

    if panning.with_left != with_left {
        panning.with_left = with_left;
    }
}

fn paint(
    buttons: Res<ButtonInput<MouseButton>>,
    hovered: Res<Hovered>,
    brush: Res<Brush>,
    over_ui: Res<PointerOverUi>,
    mut map: ResMut<TileMap>,
    mut undo: ResMut<UndoStack>,
    mut painted: MessageWriter<Painted>,
) {
    if !buttons.pressed(MouseButton::Left) {
        undo.commit();
        return;
    }

    if over_ui.0 || !brush.tool.edits() {
        return;
    }

    let Some(centre) = hovered.pos else { return };
    let targets = brush.targets(&map, centre);
    let mut changed = Vec::new();

    undo.begin();

    for pos in targets {
        let Some(tile) = map.get(pos) else { continue };
        let Some(next) = brush.tool.applied(tile) else {
            continue;
        };

        undo.record(pos, tile);
        map.set(pos.x, pos.y, next);
        changed.push(pos);
    }

    if !changed.is_empty() {
        painted.write(Painted::new(changed));
    }
}

fn drop_in(
    buttons: Res<ButtonInput<MouseButton>>,
    hovered: Res<Hovered>,
    brush: Res<Brush>,
    over_ui: Res<PointerOverUi>,
    time: Res<Time>,
    mut rest: ResMut<SpawnRest>,
    mut spawn: MessageWriter<Spawn>,
) {
    let Tool::Spawn(species) = brush.tool else {
        return;
    };

    if buttons.just_pressed(MouseButton::Left) {
        rest.0 = 0.0;
    }

    rest.0 = (rest.0 - time.delta_secs()).max(0.0);

    if over_ui.0 || !buttons.pressed(MouseButton::Left) || rest.0 > 0.0 {
        return;
    }

    let Some(pos) = hovered.pos else { return };

    spawn.write(Spawn { species, pos });
    rest.0 = SPAWN_GAP;
}

fn rewind(
    mut asked: MessageReader<Undo>,
    mut undo: ResMut<UndoStack>,
    mut map: ResMut<TileMap>,
    mut painted: MessageWriter<Painted>,
) {
    for _ in 0..asked.read().count() {
        let Some(stroke) = undo.take() else { break };
        let mut cells = Vec::with_capacity(stroke.before.len());

        for (pos, tile) in stroke.before.iter().rev() {
            map.set(pos.x, pos.y, *tile);
            cells.push(*pos);
        }

        painted.write(Painted::new(cells));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map() -> TileMap {
        TileMap::filled(32, 32, Tile::new(Ground::SoilLow, Overlay::Grass))
    }

    #[test]
    fn the_inspect_tool_leaves_the_map_alone() {
        let tile = Tile::new(Ground::SoilLow, Overlay::Grass);

        assert!(!Tool::Inspect.edits());
        assert!(!Tool::Inspect.holds());
        assert_eq!(Tool::Inspect.applied(tile), None);
    }

    #[test]
    fn painting_the_same_ground_twice_changes_nothing_the_second_time() {
        let tile = Tile::bare(Ground::Sand);

        assert_eq!(Tool::Ground(Ground::Sand).applied(tile), None);
        assert!(Tool::Ground(Ground::Mountains).applied(tile).is_some());
    }

    #[test]
    fn water_refuses_an_overlay_so_the_brush_reports_no_change() {
        let sea = Tile::bare(Ground::DeepOcean);

        assert_eq!(Tool::Overlay(Overlay::Grass).applied(sea), None);
    }

    #[test]
    fn raising_walks_up_the_elevation_ladder_and_stops_at_the_peak() {
        assert_eq!(step(Ground::DeepOcean, 1), Ground::CloseOcean);
        assert_eq!(step(Ground::Sand, 1), Ground::SoilLow);
        assert_eq!(step(Ground::Mountains, 1), Ground::Mountains);
    }

    #[test]
    fn lowering_walks_back_down_and_stops_at_the_deep() {
        assert_eq!(step(Ground::Mountains, -1), Ground::Hills);
        assert_eq!(step(Ground::CloseOcean, -1), Ground::DeepOcean);
        assert_eq!(step(Ground::DeepOcean, -1), Ground::DeepOcean);
    }

    #[test]
    fn lava_is_off_the_ladder_and_does_not_move() {
        assert_eq!(step(Ground::Lava2, 1), Ground::Lava2);
        assert_eq!(step(Ground::Lava3, -1), Ground::Lava3);
    }

    #[test]
    fn raising_keeps_the_overlay_where_the_new_ground_can_hold_it() {
        let grass = Tile::new(Ground::SoilLow, Overlay::Grass);
        let raised = Tool::Raise.applied(grass).expect("a change");

        assert_eq!(raised.ground, Ground::SoilHigh);
        assert_eq!(raised.overlay, Overlay::Grass);

        let rock = Tool::Raise
            .applied(Tile::new(Ground::SoilHigh, Overlay::Grass))
            .expect("a change");

        assert_eq!(rock.ground, Ground::Hills);
        assert_eq!(rock.overlay, Overlay::Bare);
    }

    #[test]
    fn stripping_takes_the_overlay_and_leaves_the_ground() {
        let jungle = Tile::new(Ground::SoilLow, Overlay::Jungle);
        let stripped = Tool::Strip.applied(jungle).expect("a change");

        assert_eq!(stripped.ground, Ground::SoilLow);
        assert_eq!(stripped.overlay, Overlay::Bare);
        assert_eq!(Tool::Strip.applied(stripped), None);
    }

    #[test]
    fn a_round_brush_covers_fewer_cells_than_a_square_one() {
        let map = map();
        let centre = TilePos::new(16, 16);
        let round = cells(&map, centre, Shape::Round, 4).len();
        let square = cells(&map, centre, Shape::Square, 4).len();

        assert_eq!(square, 9 * 9);
        assert!(round < square, "{round} vs {square}");
    }

    #[test]
    fn a_brush_of_no_radius_is_a_single_cell() {
        let map = map();

        assert_eq!(cells(&map, TilePos::new(0, 0), Shape::Round, 0).len(), 1);
    }

    #[test]
    fn a_brush_at_the_edge_is_clipped_to_the_map() {
        let map = map();
        let corner = cells(&map, TilePos::new(0, 0), Shape::Square, 3);

        assert_eq!(corner.len(), 4 * 4);
        assert!(corner.iter().all(|pos| pos.x < 32 && pos.y < 32));
    }

    #[test]
    fn flooding_a_uniform_map_reaches_every_cell() {
        let map = map();

        assert_eq!(flood(&map, TilePos::new(5, 5)).len(), map.cells());
    }

    #[test]
    fn flooding_stops_at_a_different_ground() {
        let mut map = map();

        for y in 0..32 {
            map.set(16, y, Tile::bare(Ground::Mountains));
        }

        let left = flood(&map, TilePos::new(0, 0));

        assert_eq!(left.len(), 16 * 32);
        assert!(left.iter().all(|pos| pos.x < 16));
    }

    #[test]
    fn flooding_outside_the_map_finds_nothing() {
        let map = map();

        assert!(flood(&map, TilePos::new(99, 99)).is_empty());
    }

    #[test]
    fn the_brush_radius_stays_inside_its_range() {
        let mut brush = Brush::default();

        for _ in 0..100 {
            brush.grow();
        }
        assert_eq!(brush.radius, MAX_RADIUS);

        for _ in 0..100 {
            brush.shrink();
        }
        assert_eq!(brush.radius, MIN_RADIUS);
    }

    #[test]
    fn an_undo_stroke_remembers_each_cell_once() {
        let mut undo = UndoStack::default();
        let tile = Tile::bare(Ground::Sand);
        undo.begin();

        for _ in 0..5 {
            undo.record(TilePos::new(1, 1), tile);
        }
        undo.record(TilePos::new(2, 1), tile);
        undo.commit();

        let stroke = undo.take().expect("a stroke");

        assert_eq!(stroke.before.len(), 2);
    }

    #[test]
    fn an_empty_stroke_is_not_worth_undoing() {
        let mut undo = UndoStack::default();
        undo.begin();
        undo.commit();

        assert_eq!(undo.depth(), 0);
        assert!(undo.take().is_none());
    }

    #[test]
    fn the_undo_stack_forgets_the_oldest_stroke_once_it_is_full() {
        let mut undo = UndoStack::default();

        for step in 0..UNDO_LIMIT + 10 {
            undo.begin();
            undo.record(TilePos::new(step as u32, 0), Tile::default());
            undo.commit();
        }

        assert_eq!(undo.depth(), UNDO_LIMIT);
    }

    #[test]
    fn a_stroke_undone_puts_every_tile_back() {
        let mut map = map();
        let brush = Brush {
            tool: Tool::Ground(Ground::Mountains),
            shape: Shape::Square,
            radius: 2,
            flood: false,
        };
        let centre = TilePos::new(10, 10);
        let before = map.clone();
        let mut undo = UndoStack::default();
        undo.begin();

        for pos in brush.targets(&map, centre) {
            let tile = map.get(pos).expect("inside");
            if let Some(next) = brush.tool.applied(tile) {
                undo.record(pos, tile);
                map.set(pos.x, pos.y, next);
            }
        }
        undo.commit();

        assert_ne!(map, before);

        let stroke = undo.take().expect("a stroke");
        for (pos, tile) in stroke.before.iter().rev() {
            map.set(pos.x, pos.y, *tile);
        }

        assert_eq!(map, before);
    }

    #[test]
    fn a_flood_brush_ignores_the_radius() {
        let map = map();
        let brush = Brush {
            tool: Tool::Ground(Ground::Sand),
            shape: Shape::Round,
            radius: 1,
            flood: true,
        };

        assert_eq!(brush.targets(&map, TilePos::new(0, 0)).len(), map.cells());
    }

    #[test]
    fn only_the_inspect_tool_gives_the_left_button_back_to_the_camera() {
        assert!(Tool::Ground(Ground::Sand).holds());
        assert!(Tool::Raise.holds());
        assert!(Tool::Spawn(Species::Human).holds());
        assert!(!Tool::Inspect.holds());
    }

    #[test]
    fn a_spawn_tool_places_a_creature_instead_of_editing_the_ground() {
        let tool = Tool::Spawn(Species::Human);

        assert!(!tool.edits());
        assert_eq!(tool.applied(Tile::bare(Ground::Sand)), None);
        assert_eq!(tool.name(), "human");
        assert_eq!(tool.swatch(), Some(Species::Human.color()));
    }

    #[test]
    fn every_tool_but_a_spawn_has_an_icon_of_its_own() {
        for ground in econbox_tile::GROUNDS {
            assert!(Tool::Ground(ground).icon().is_some(), "{}", ground.name());
        }

        assert_eq!(Tool::Raise.icon().map(|found| found.0), Some(Icon::Raise));
        assert_eq!(Tool::Lower.icon().map(|found| found.0), Some(Icon::Lower));
        assert_eq!(Tool::Inspect.icon().map(|found| found.0), Some(Icon::Lens));
        assert_eq!(Tool::Strip.icon().map(|found| found.0), Some(Icon::Eraser));
        assert!(Tool::Spawn(Species::Cat).icon().is_none());
    }

    #[test]
    fn water_gets_a_droplet_and_solid_ground_gets_a_slab() {
        assert_eq!(
            Tool::Ground(Ground::DeepOcean).icon(),
            Some((Icon::Droplet, Some(Ground::DeepOcean.color())))
        );
        assert_eq!(
            Tool::Ground(Ground::Mountains).icon(),
            Some((Icon::Block, Some(Ground::Mountains.color())))
        );
    }

    #[test]
    fn a_palette_tool_carries_the_colour_it_paints() {
        assert_eq!(
            Tool::Ground(Ground::Sand).swatch(),
            Some(Ground::Sand.color())
        );
        assert_eq!(Tool::Overlay(Overlay::Bare).swatch(), None);
        assert_eq!(Tool::Raise.swatch(), None);
        assert_eq!(
            Tool::Spawn(Species::Fox).swatch(),
            Some(Species::Fox.color())
        );
    }
}
