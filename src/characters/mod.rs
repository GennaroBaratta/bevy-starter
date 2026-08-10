mod animation;
pub mod config;
mod movement;
mod spawn;

use bevy::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;

use config::CharactersList;

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum CharacterSystems {
    Update,
}

pub(crate) struct CharactersPlugin;

impl Plugin for CharactersPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RonAssetPlugin::<CharactersList>::new(&["characters.ron"]))
            .init_resource::<spawn::CurrentCharacterIndex>()
            .add_systems(Startup, spawn::spawn_player)
            .add_systems(
                Update,
                (
                    spawn::initialize_player_character,
                    spawn::switch_character,
                    movement::move_player,
                    animation::animate_characters,
                    movement::update_jump_state,
                    animation::update_animation_flags,
                )
                    .chain()
                    .after(crate::plugins::input::InputSystem::UpdateJoystick)
                    .in_set(CharacterSystems::Update),
            );
    }
}

#[cfg(test)]
mod tests {
    use bevy::asset::{AssetPlugin, LoadState};

    use super::*;

    #[test]
    fn bundled_character_list_loads_all_six_entries() {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            AssetPlugin::default(),
            RonAssetPlugin::<CharactersList>::new(&["characters.ron"]),
        ));

        let asset_server = app.world().resource::<AssetServer>().clone();
        let handle: Handle<CharactersList> = asset_server.load("characters/characters.ron");

        for _ in 0..10_000 {
            app.update();
            match asset_server.load_state(&handle) {
                LoadState::Loaded => break,
                LoadState::Failed(error) => panic!("failed to load characters.ron: {error}"),
                _ => continue,
            }
        }

        let characters = app
            .world()
            .resource::<Assets<CharactersList>>()
            .get(&handle)
            .expect("characters.ron did not finish loading");
        assert_eq!(characters.characters.len(), 6);
        assert_eq!(characters.characters[0].name, "male");
        assert_eq!(characters.characters[5].name, "starlit_oracle");
        assert!(characters.characters.iter().all(|character| {
            [
                config::AnimationType::Walk,
                config::AnimationType::Run,
                config::AnimationType::Jump,
            ]
            .into_iter()
            .all(|animation| character.animations.contains_key(&animation))
        }));
    }
}
