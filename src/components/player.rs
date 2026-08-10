use bevy::prelude::*;

/// Identifies the player entity for movement, camera, and world-streaming systems.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Player;
