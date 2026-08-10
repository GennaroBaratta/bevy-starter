use std::time::Duration;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::config::{AnimationType, CharacterEntry};

pub const DEFAULT_ANIMATION_FRAME_TIME: f32 = 0.1;

pub fn frame_duration(frame_time: f32) -> Duration {
    let seconds = if frame_time.is_finite() && frame_time > 0.0 {
        frame_time
    } else {
        DEFAULT_ANIMATION_FRAME_TIME
    };
    Duration::from_secs_f32(seconds)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Facing {
    Up,
    Left,
    Down,
    Right,
}

impl Facing {
    pub fn from_direction(direction: Vec2) -> Self {
        if direction.x.abs() > direction.y.abs() {
            if direction.x > 0.0 {
                Self::Right
            } else {
                Self::Left
            }
        } else if direction.y > 0.0 {
            Self::Up
        } else {
            Self::Down
        }
    }

    fn direction_index(self) -> usize {
        match self {
            Self::Up => 0,
            Self::Left => 1,
            Self::Down => 2,
            Self::Right => 3,
        }
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnimationController {
    pub current_animation: AnimationType,
    pub facing: Facing,
}

impl Default for AnimationController {
    fn default() -> Self {
        Self {
            current_animation: AnimationType::Walk,
            facing: Facing::Down,
        }
    }
}

impl AnimationController {
    pub fn get_clip(&self, config: &CharacterEntry) -> Option<AnimationClip> {
        let definition = config.animations.get(&self.current_animation)?;
        let row = if definition.directional {
            definition.start_row + self.facing.direction_index()
        } else {
            definition.start_row
        };

        AnimationClip::new(row, definition.frame_count, config.atlas_columns)
    }
}

#[derive(Component, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct AnimationState {
    pub is_moving: bool,
    pub was_moving: bool,
    pub is_jumping: bool,
    pub was_jumping: bool,
}

#[derive(Component, Deref, DerefMut)]
pub struct AnimationTimer(pub Timer);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnimationClip {
    first: usize,
    last: usize,
}

impl AnimationClip {
    pub fn new(row: usize, frame_count: usize, atlas_columns: usize) -> Option<Self> {
        if frame_count == 0 || atlas_columns == 0 || frame_count > atlas_columns {
            return None;
        }

        let first = row.checked_mul(atlas_columns)?;
        let last = first.checked_add(frame_count - 1)?;
        Some(Self { first, last })
    }

    pub fn start(self) -> usize {
        self.first
    }

    pub fn contains(self, index: usize) -> bool {
        (self.first..=self.last).contains(&index)
    }

    pub fn next(self, index: usize) -> usize {
        if index >= self.last {
            self.first
        } else {
            index + 1
        }
    }

    pub fn is_last(self, index: usize) -> bool {
        index >= self.last
    }

    pub fn is_complete(self, current_index: usize, timer_finished: bool) -> bool {
        current_index >= self.last && timer_finished
    }
}

pub fn animate_characters(
    time: Res<Time>,
    mut characters: Query<(
        &AnimationController,
        &AnimationState,
        &mut AnimationTimer,
        &mut Sprite,
        &CharacterEntry,
    )>,
) {
    for (controller, state, mut timer, mut sprite, config) in &mut characters {
        let Some(atlas) = sprite.texture_atlas.as_mut() else {
            continue;
        };
        let Some(clip) = controller.get_clip(config) else {
            continue;
        };
        let Some(definition) = config.animations.get(&controller.current_animation) else {
            continue;
        };

        let frame_duration = frame_duration(definition.frame_time);
        let timing_changed = timer.duration() != frame_duration;
        if timing_changed {
            timer.set_duration(frame_duration);
        }

        let movement_changed = state.is_moving != state.was_moving;
        let jump_changed = state.is_jumping != state.was_jumping;
        if !clip.contains(atlas.index) || movement_changed || jump_changed || timing_changed {
            atlas.index = clip.start();
            timer.reset();
            continue;
        }

        if state.is_moving || state.is_jumping {
            timer.tick(time.delta());
            if timer.just_finished() {
                if controller.current_animation == AnimationType::Jump {
                    if !clip.is_last(atlas.index) {
                        atlas.index = clip.next(atlas.index);

                        // Let the final jump frame remain visible for a complete frame interval.
                        // Resetting here also keeps `just_finished` false until that interval ends.
                        if clip.is_last(atlas.index) {
                            timer.reset();
                        }
                    }
                } else {
                    atlas.index = clip.next(atlas.index);
                }
            }
        } else {
            atlas.index = clip.start();
        }
    }
}

pub fn update_animation_flags(mut states: Query<&mut AnimationState>) {
    for mut state in &mut states {
        state.was_moving = state.is_moving;
        state.was_jumping = state.is_jumping;
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::characters::config::AnimationDefinition;

    fn character_entry() -> CharacterEntry {
        CharacterEntry {
            name: "test".into(),
            max_health: 100.0,
            base_move_speed: 140.0,
            run_speed_multiplier: 1.8,
            texture_path: "test.png".into(),
            tile_size: 64,
            atlas_columns: 9,
            animations: HashMap::from([(
                AnimationType::Walk,
                AnimationDefinition {
                    start_row: 8,
                    frame_count: 9,
                    frame_time: 0.1,
                    directional: true,
                },
            )]),
        }
    }

    #[test]
    fn facing_uses_the_dominant_movement_axis() {
        assert_eq!(Facing::from_direction(Vec2::new(3.0, 1.0)), Facing::Right);
        assert_eq!(Facing::from_direction(Vec2::new(-3.0, 1.0)), Facing::Left);
        assert_eq!(Facing::from_direction(Vec2::new(1.0, 3.0)), Facing::Up);
        assert_eq!(Facing::from_direction(Vec2::new(1.0, -3.0)), Facing::Down);
    }

    #[test]
    fn directional_clip_uses_facing_row_and_loops() {
        let controller = AnimationController::default();
        let clip = controller.get_clip(&character_entry()).unwrap();

        assert_eq!(clip.start(), 90);
        assert!(clip.contains(98));
        assert!(!clip.contains(99));
        assert_eq!(clip.next(98), 90);
        assert!(clip.is_complete(98, true));
    }

    #[test]
    fn invalid_clip_dimensions_are_rejected() {
        assert_eq!(AnimationClip::new(0, 0, 9), None);
        assert_eq!(AnimationClip::new(0, 9, 0), None);
        assert_eq!(AnimationClip::new(0, 10, 9), None);
    }
}
