#![allow(unused_imports)]

use bevy::prelude::*;

mod components;
mod plugins;
mod resources;
mod styles;
mod third_party;
mod utils;

#[bevy_main]
pub fn main() {
    App::new().add_plugins(AppPlugin).run();
}

/// Use this module instead of importing the `components`, `plugins`, `resources`, and `utils`
/// modules directly.
mod prelude {
    pub use super::*;
    pub use {components::*, plugins::*, resources::*, styles::*, utils::*};
}

pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            third_party::plugin,
            plugins::camera::plugin,
            plugins::defaults::plugin,
            plugins::fonts::plugin,
            plugins::game::plugin,
            plugins::input::plugin,
            plugins::physics::plugin,
            components::player::PlayerPlugin,
        ));

        // Enable dev tools for dev builds.
        #[cfg(feature = "dev")]
        app.add_plugins(plugins::debug::plugin);
    }
}
