#![allow(unused_imports)]

use bevy::prelude::*;

mod characters;
mod components;
mod map;
mod plugins;
mod resources;
mod styles;
mod third_party;

#[bevy_main]
pub fn main() {
    App::new().add_plugins(AppPlugin).run();
}

/// Use this module instead of importing the `components`, `plugins`, `resources`, and `utils`
/// modules directly.
mod prelude {
    pub use super::*;
    pub use {components::*, plugins::*, resources::*, styles::*};
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
            characters::CharactersPlugin,
        ));

        // Enable dev tools for dev builds.
        #[cfg(feature = "dev")]
        app.add_plugins(plugins::debug::plugin);
    }
}
