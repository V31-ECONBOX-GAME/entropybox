//! Application properties translated into what WindowPlugin takes.

use crate::config::properties::ApplicationProperties;
use bevy::prelude::*;

pub fn plugin(application: ApplicationProperties) -> WindowPlugin {
    WindowPlugin {
        primary_window: Some(Window {
            title: application.name,
            ..default()
        }),
        ..default()
    }
}
