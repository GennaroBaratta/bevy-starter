use bevy::{asset::LoadState, prelude::*};

use crate::components::player::Player;

use super::{
    animation::{
        AnimationController, AnimationState, AnimationTimer, DEFAULT_ANIMATION_FRAME_TIME,
        frame_duration,
    },
    config::{AnimationType, CharacterEntry, CharactersList},
};

const PLAYER_SCALE: f32 = 0.8;
const PLAYER_Z_POSITION: f32 = 20.0;

#[derive(Resource, Default)]
pub struct CurrentCharacterIndex {
    pub index: usize,
}

#[derive(Resource)]
pub struct CharactersListResource {
    pub handle: Handle<CharactersList>,
}

fn create_character_atlas_layout(
    atlas_layouts: &mut Assets<TextureAtlasLayout>,
    character: &CharacterEntry,
) -> Option<Handle<TextureAtlasLayout>> {
    if character.tile_size == 0 || character.atlas_columns == 0 {
        error!(
            "character '{}' has invalid atlas dimensions",
            character.name
        );
        return None;
    }

    let rows = character.calculate_max_animation_row().saturating_add(1);
    let Ok(columns) = u32::try_from(character.atlas_columns) else {
        error!("character '{}' has too many atlas columns", character.name);
        return None;
    };
    let Ok(rows) = u32::try_from(rows) else {
        error!("character '{}' has too many atlas rows", character.name);
        return None;
    };
    Some(atlas_layouts.add(TextureAtlasLayout::from_grid(
        UVec2::splat(character.tile_size),
        columns,
        rows,
        None,
        None,
    )))
}

fn animation_frame_duration(
    character: &CharacterEntry,
    animation: AnimationType,
) -> std::time::Duration {
    character
        .animations
        .get(&animation)
        .map(|definition| frame_duration(definition.frame_time))
        .unwrap_or_else(|| std::time::Duration::from_secs_f32(DEFAULT_ANIMATION_FRAME_TIME))
}

fn sprite_for_character(
    asset_server: &AssetServer,
    atlas_layouts: &mut Assets<TextureAtlasLayout>,
    character: &CharacterEntry,
    controller: AnimationController,
) -> Option<Sprite> {
    let layout = create_character_atlas_layout(atlas_layouts, character)?;
    let Some(clip) = controller.get_clip(character) else {
        error!(
            "character '{}' has an invalid default animation",
            character.name
        );
        return None;
    };

    Some(Sprite::from_atlas_image(
        asset_server.load(character.texture_path.clone()),
        TextureAtlas {
            layout,
            index: clip.start(),
        },
    ))
}

pub fn spawn_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut current_character: ResMut<CurrentCharacterIndex>,
) {
    let handle = asset_server.load("characters/characters.ron");
    commands.insert_resource(CharactersListResource { handle });
    current_character.index = 0;

    commands.spawn((
        Player,
        Transform::from_xyz(0.0, 0.0, PLAYER_Z_POSITION).with_scale(Vec3::splat(PLAYER_SCALE)),
        Sprite::default(),
    ));
}

pub fn initialize_player_character(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    character_lists: Res<Assets<CharactersList>>,
    current_character: Res<CurrentCharacterIndex>,
    list_resource: Option<Res<CharactersListResource>>,
    players: Query<Entity, (With<Player>, Without<AnimationController>)>,
    mut load_failure_reported: Local<bool>,
) {
    let Some(list_resource) = list_resource else {
        return;
    };
    let Some(character_list) = character_lists.get(&list_resource.handle) else {
        if !*load_failure_reported
            && let Some(LoadState::Failed(error)) =
                asset_server.get_load_state(&list_resource.handle)
        {
            error!(?error, "failed to load character definitions");
            *load_failure_reported = true;
        }
        return;
    };
    let Some(character) = character_list.characters.get(current_character.index) else {
        return;
    };

    let controller = AnimationController::default();
    let Some(sprite) =
        sprite_for_character(&asset_server, &mut atlas_layouts, character, controller)
    else {
        return;
    };
    let frame_duration = animation_frame_duration(character, controller.current_animation);

    for entity in &players {
        commands.entity(entity).insert((
            controller,
            AnimationState::default(),
            AnimationTimer(Timer::new(frame_duration, TimerMode::Repeating)),
            character.clone(),
            sprite.clone(),
        ));
    }
}

fn requested_character_index(input: &ButtonInput<KeyCode>) -> Option<usize> {
    const DIGIT_KEYS: [KeyCode; 9] = [
        KeyCode::Digit1,
        KeyCode::Digit2,
        KeyCode::Digit3,
        KeyCode::Digit4,
        KeyCode::Digit5,
        KeyCode::Digit6,
        KeyCode::Digit7,
        KeyCode::Digit8,
        KeyCode::Digit9,
    ];

    DIGIT_KEYS.iter().position(|key| input.just_pressed(*key))
}

pub fn switch_character(
    input: Res<ButtonInput<KeyCode>>,
    mut current_character: ResMut<CurrentCharacterIndex>,
    character_lists: Res<Assets<CharactersList>>,
    list_resource: Option<Res<CharactersListResource>>,
    mut players: Query<
        (
            &mut CharacterEntry,
            &mut Sprite,
            &mut AnimationController,
            &mut AnimationState,
            &mut AnimationTimer,
        ),
        With<Player>,
    >,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    asset_server: Res<AssetServer>,
) {
    let Some(new_index) = requested_character_index(&input) else {
        return;
    };
    let Some(list_resource) = list_resource else {
        return;
    };
    let Some(character_list) = character_lists.get(&list_resource.handle) else {
        return;
    };
    let Some(character) = character_list.characters.get(new_index) else {
        return;
    };
    let Ok((mut old_character, mut sprite, mut controller, mut state, mut timer)) =
        players.single_mut()
    else {
        return;
    };

    let new_controller = AnimationController::default();
    let Some(new_sprite) =
        sprite_for_character(&asset_server, &mut atlas_layouts, character, new_controller)
    else {
        return;
    };

    current_character.index = new_index;
    *old_character = character.clone();
    *sprite = new_sprite;
    *controller = new_controller;
    *state = AnimationState::default();
    timer.set_duration(animation_frame_duration(
        character,
        new_controller.current_animation,
    ));
    timer.reset();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn digit_keys_map_to_zero_based_character_indices() {
        let mut input = ButtonInput::default();
        input.press(KeyCode::Digit6);

        assert_eq!(requested_character_index(&input), Some(5));
    }
}
