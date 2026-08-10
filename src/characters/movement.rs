use bevy::prelude::*;

use crate::{components::player::Player, plugins::input::MovementInput};

use super::{
    animation::{AnimationController, AnimationState, AnimationTimer, Facing},
    config::{AnimationType, CharacterEntry},
};

fn read_keyboard_movement(input: &ButtonInput<KeyCode>) -> Vec2 {
    const MOVEMENT_KEYS: [(KeyCode, Vec2); 4] = [
        (KeyCode::ArrowLeft, Vec2::NEG_X),
        (KeyCode::ArrowRight, Vec2::X),
        (KeyCode::ArrowUp, Vec2::Y),
        (KeyCode::ArrowDown, Vec2::NEG_Y),
    ];

    MOVEMENT_KEYS
        .iter()
        .filter(|(key, _)| input.pressed(*key))
        .map(|(_, direction)| *direction)
        .sum()
}

fn calculate_movement_speed(character: &CharacterEntry, is_running: bool) -> f32 {
    if is_running {
        character.base_move_speed * character.run_speed_multiplier
    } else {
        character.base_move_speed
    }
}

fn movement_direction(input: &ButtonInput<KeyCode>, analog: Vec2) -> Vec2 {
    (read_keyboard_movement(input) + analog).clamp_length_max(1.0)
}

pub fn move_player(
    input: Res<ButtonInput<KeyCode>>,
    movement: Res<MovementInput>,
    time: Res<Time>,
    mut player: Query<
        (
            &mut Transform,
            &mut AnimationController,
            &mut AnimationState,
            &CharacterEntry,
        ),
        With<Player>,
    >,
) {
    let Ok((mut transform, mut controller, mut state, character)) = player.single_mut() else {
        return;
    };

    let direction = movement_direction(&input, movement.0);

    if input.just_pressed(KeyCode::Space) && !state.is_jumping {
        state.is_jumping = true;
        controller.current_animation = AnimationType::Jump;
    }

    let is_running = input.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
    if direction != Vec2::ZERO {
        let move_speed = calculate_movement_speed(character, is_running);
        transform.translation += (direction * move_speed * time.delta_secs()).extend(0.0);
        controller.facing = Facing::from_direction(direction);

        if !state.is_jumping {
            state.is_moving = true;
            controller.current_animation = if is_running {
                AnimationType::Run
            } else {
                AnimationType::Walk
            };
        }
    } else if !state.is_jumping {
        state.is_moving = false;
        controller.current_animation = AnimationType::Walk;
    }
}

pub fn update_jump_state(
    mut players: Query<
        (
            &mut AnimationController,
            &mut AnimationState,
            &AnimationTimer,
            &Sprite,
            &CharacterEntry,
        ),
        With<Player>,
    >,
) {
    for (mut controller, mut state, timer, sprite, config) in &mut players {
        if !state.is_jumping {
            continue;
        }
        let Some(atlas) = sprite.texture_atlas.as_ref() else {
            continue;
        };
        let Some(clip) = controller.get_clip(config) else {
            continue;
        };

        if clip.is_complete(atlas.index, timer.just_finished()) {
            state.is_jumping = false;
            controller.current_animation = AnimationType::Walk;
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::time::Duration;

    use bevy::time::TimeUpdateStrategy;

    use super::*;
    use crate::characters::{
        animation::{animate_characters, update_animation_flags},
        config::AnimationDefinition,
    };

    fn character_entry() -> CharacterEntry {
        CharacterEntry {
            name: "test".into(),
            max_health: 100.0,
            base_move_speed: 140.0,
            run_speed_multiplier: 1.8,
            texture_path: "test.png".into(),
            tile_size: 64,
            atlas_columns: 9,
            animations: HashMap::new(),
        }
    }

    #[test]
    fn keyboard_directions_are_combined() {
        let mut input = ButtonInput::default();
        input.press(KeyCode::ArrowUp);
        input.press(KeyCode::ArrowRight);

        assert_eq!(read_keyboard_movement(&input), Vec2::new(1.0, 1.0));
    }

    #[test]
    fn running_uses_the_configured_multiplier() {
        let character = character_entry();

        assert_eq!(calculate_movement_speed(&character, false), 140.0);
        assert_eq!(calculate_movement_speed(&character, true), 252.0);
    }

    #[test]
    fn analog_movement_preserves_partial_joystick_magnitude() {
        let input = ButtonInput::default();

        assert_eq!(
            movement_direction(&input, Vec2::new(0.25, 0.0)),
            Vec2::new(0.25, 0.0)
        );
        assert_eq!(movement_direction(&input, Vec2::new(2.0, 0.0)), Vec2::X);
    }

    #[test]
    fn jump_holds_its_last_frame_then_returns_to_walk() {
        let mut character = character_entry();
        character.animations = HashMap::from([
            (
                AnimationType::Walk,
                AnimationDefinition {
                    start_row: 1,
                    frame_count: 1,
                    frame_time: 0.1,
                    directional: false,
                },
            ),
            (
                AnimationType::Jump,
                AnimationDefinition {
                    start_row: 0,
                    frame_count: 3,
                    frame_time: 0.1,
                    directional: false,
                },
            ),
        ]);
        character.atlas_columns = 3;

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
                110,
            )))
            .add_systems(
                Update,
                (
                    animate_characters,
                    update_jump_state,
                    update_animation_flags,
                )
                    .chain(),
            );
        let player = app
            .world_mut()
            .spawn((
                Player,
                AnimationController {
                    current_animation: AnimationType::Jump,
                    facing: Facing::Down,
                },
                AnimationState {
                    is_jumping: true,
                    was_jumping: true,
                    ..default()
                },
                AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
                Sprite {
                    texture_atlas: Some(TextureAtlas {
                        layout: Handle::default(),
                        index: 0,
                    }),
                    ..default()
                },
                character,
            ))
            .id();

        app.update(); // Time starts at a zero delta.
        app.update();
        app.update();

        let sprite = app.world().get::<Sprite>(player).unwrap();
        assert_eq!(sprite.texture_atlas.as_ref().unwrap().index, 2);
        assert!(
            app.world()
                .get::<AnimationState>(player)
                .unwrap()
                .is_jumping
        );

        app.update();

        let state = app.world().get::<AnimationState>(player).unwrap();
        let controller = app.world().get::<AnimationController>(player).unwrap();
        assert!(!state.is_jumping);
        assert_eq!(controller.current_animation, AnimationType::Walk);
    }
}
