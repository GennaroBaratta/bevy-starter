use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AnimationType {
    Walk,
    Run,
    Jump,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationDefinition {
    pub start_row: usize,
    pub frame_count: usize,
    pub frame_time: f32,
    pub directional: bool,
}

#[derive(Component, Asset, TypePath, Debug, Clone, Serialize, Deserialize)]
pub struct CharacterEntry {
    pub name: String,
    pub max_health: f32,
    pub base_move_speed: f32,
    pub run_speed_multiplier: f32,
    pub texture_path: String,
    pub tile_size: u32,
    pub atlas_columns: usize,
    pub animations: HashMap<AnimationType, AnimationDefinition>,
}

impl CharacterEntry {
    pub fn calculate_max_animation_row(&self) -> usize {
        self.animations
            .values()
            .map(|animation| {
                animation
                    .start_row
                    .saturating_add(if animation.directional { 3 } else { 0 })
            })
            .max()
            .unwrap_or(0)
    }
}

#[derive(Asset, TypePath, Debug, Clone, Serialize, Deserialize)]
pub struct CharactersList {
    pub characters: Vec<CharacterEntry>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maximum_animation_row_accounts_for_directional_clips() {
        let entry = CharacterEntry {
            name: "test".into(),
            max_health: 100.0,
            base_move_speed: 100.0,
            run_speed_multiplier: 2.0,
            texture_path: "test.png".into(),
            tile_size: 64,
            atlas_columns: 9,
            animations: HashMap::from([
                (
                    AnimationType::Walk,
                    AnimationDefinition {
                        start_row: 8,
                        frame_count: 9,
                        frame_time: 0.1,
                        directional: true,
                    },
                ),
                (
                    AnimationType::Jump,
                    AnimationDefinition {
                        start_row: 26,
                        frame_count: 5,
                        frame_time: 0.1,
                        directional: false,
                    },
                ),
            ]),
        };

        assert_eq!(entry.calculate_max_animation_row(), 26);
    }
}
