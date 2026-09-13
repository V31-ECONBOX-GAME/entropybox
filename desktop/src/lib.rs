pub mod constant;
pub mod properties;
pub mod util;

use constant::keys;
use entropybox_starter_simulation::prelude::*;
use properties::{ApplicationProperties, LoggingProperties};
use util::{logs, windows};

pub fn app() -> App {
    app_with(&properties::load().expect("load properties"))
}

pub fn app_with(source: &Config) -> App {
    let application: ApplicationProperties = source.get(keys::APPLICATION).unwrap_or_default();
    let logging: LoggingProperties = source.get(keys::LOGGING).unwrap_or_default();

    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(windows::plugin(application))
            .set(logs::plugin(&logging)),
    )
    .add_plugins(SimulationStarter)
    .add_systems(Startup, setup)
    .add_systems(Update, spin);

    #[cfg(feature = "dev")]
    app.add_plugins(bevy::dev_tools::fps_overlay::FpsOverlayPlugin::default());

    app
}

pub fn headless_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(SimulationStarter);
    app
}

#[derive(Component)]
struct Example;

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.spawn((
        Example,
        Sprite::from_color(Color::srgb(0.35, 0.7, 0.9), Vec2::splat(64.0)),
    ));
}

fn spin(time: Res<Time>, mut examples: Query<&mut Transform, With<Example>>) {
    for mut transform in &mut examples {
        transform.rotate_z(time.delta_secs());
    }
}
