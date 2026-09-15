//! World time: whether it runs, how fast, and how much of it has passed.

use bevy::prelude::*;

pub const SPEEDS: [u32; 4] = [1, 2, 3, 5];
pub const TICK: f32 = 0.2;

#[derive(Resource, Debug, Clone, Copy, PartialEq, Default)]
pub struct WorldClock {
    pub paused: bool,
    pub rung: usize,
    pub elapsed: f32,
    pub ticks: u64,
}

impl WorldClock {
    pub fn speed(&self) -> u32 {
        SPEEDS[self.rung.min(SPEEDS.len() - 1)]
    }

    pub fn rate(&self) -> f32 {
        if self.paused {
            0.0
        } else {
            self.speed() as f32
        }
    }

    pub fn toggle(&mut self) {
        self.paused = !self.paused;
    }

    pub fn faster(&mut self) {
        self.rung = (self.rung + 1) % SPEEDS.len();
    }

    pub fn advance(&mut self, seconds: f32) {
        self.elapsed += seconds * self.rate();
        self.ticks = (self.elapsed / TICK) as u64;
    }

    pub fn label(&self) -> String {
        if self.paused {
            "paused".to_string()
        } else {
            format!("x{}", self.speed())
        }
    }
}

pub struct ClockPlugin;

impl Plugin for ClockPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorldClock>()
            .add_systems(Update, advance);
    }
}

fn advance(time: Res<Time>, mut clock: ResMut<WorldClock>) {
    if clock.rate() == 0.0 {
        return;
    }

    clock.advance(time.delta_secs());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fresh_clock_runs_at_one_times_speed() {
        let clock = WorldClock::default();

        assert!(!clock.paused);
        assert_eq!(clock.speed(), 1);
        assert_eq!(clock.rate(), 1.0);
        assert_eq!(clock.label(), "x1");
    }

    #[test]
    fn a_paused_clock_does_not_move() {
        let mut clock = WorldClock::default();
        clock.toggle();
        clock.advance(10.0);

        assert_eq!(clock.rate(), 0.0);
        assert_eq!(clock.elapsed, 0.0);
        assert_eq!(clock.ticks, 0);
        assert_eq!(clock.label(), "paused");
    }

    #[test]
    fn the_speed_button_cycles_back_round_to_one() {
        let mut clock = WorldClock::default();
        let seen: Vec<u32> = (0..SPEEDS.len())
            .map(|_| {
                let speed = clock.speed();
                clock.faster();
                speed
            })
            .collect();

        assert_eq!(seen, SPEEDS);
        assert_eq!(clock.speed(), SPEEDS[0]);
    }

    #[test]
    fn faster_time_runs_further_in_the_same_second() {
        let mut slow = WorldClock::default();
        let mut fast = WorldClock::default();
        fast.faster();

        slow.advance(1.0);
        fast.advance(1.0);

        assert!(fast.elapsed > slow.elapsed);
        assert_eq!(fast.elapsed, slow.elapsed * fast.speed() as f32);
    }

    #[test]
    fn ticks_count_the_elapsed_world_seconds() {
        let mut clock = WorldClock::default();
        clock.advance(1.0);

        assert_eq!(clock.ticks, (1.0 / TICK) as u64);

        clock.advance(TICK);
        assert_eq!(clock.ticks, (1.0 / TICK) as u64 + 1);
    }

    #[test]
    fn many_small_steps_add_up_to_the_same_stretch_of_time() {
        let mut clock = WorldClock::default();

        for _ in 0..100 {
            clock.advance(0.01);
        }

        assert!((clock.elapsed - 1.0).abs() < 1e-3, "{}", clock.elapsed);
    }

    #[test]
    fn unpausing_picks_up_where_it_stopped() {
        let mut clock = WorldClock::default();
        clock.advance(1.0);
        let held = clock.elapsed;

        clock.toggle();
        clock.advance(5.0);
        assert_eq!(clock.elapsed, held);

        clock.toggle();
        clock.advance(1.0);
        assert!(clock.elapsed > held);
    }
}
