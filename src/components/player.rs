use crate::plugins::input::MovementInput;
use bevy::prelude::*;

// Atlas constants
const TILE_SIZE: u32 = 64; // 64x64 tiles
const ATLAS_COLUMNS: usize = 13;
const ATLAS_ROWS: usize = 54;
const WALK_FRAMES: usize = 9; // 9 columns per walking row
const MOVE_SPEED: f32 = 140.0; // pixels per second
const ANIM_DT: f32 = 0.1; // seconds per frame (~10 FPS)

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
enum Facing {
    Up,
    Left,
    Down,
    Right,
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);
#[derive(Component)]
struct AnimationState {
    facing: Facing,
    moving: bool,
    was_moving: bool,
}

#[derive(Component)]
pub struct Player;

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_player)
            .add_systems(Update, (move_player, animate_player).chain());
    }
}

fn atlas_index_for(facing: Facing, frame: usize) -> usize {
    let row = match facing {
        Facing::Up => 8,
        Facing::Left => 9,
        Facing::Down => 10,
        Facing::Right => 11,
    };
    row * ATLAS_COLUMNS + frame
}

fn spawn_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let texture = asset_server.load("sprites/character.png");
    let layout = atlas_layouts.add(TextureAtlasLayout::from_grid(
        UVec2::splat(TILE_SIZE),
        ATLAS_COLUMNS as u32,
        ATLAS_ROWS as u32,
        None,
        None,
    ));
    // Start facing down (towards user), idle on first frame of that row
    let facing = Facing::Down;
    let start_index = atlas_index_for(facing, 0);
    commands.spawn((
        Sprite::from_atlas_image(
            texture,
            TextureAtlas {
                layout,
                index: start_index,
            },
        ),
        Transform::from_translation(Vec3::ZERO),
        Player,
        AnimationState {
            facing,
            moving: false,
            was_moving: false,
        },
        AnimationTimer(Timer::from_seconds(ANIM_DT, TimerMode::Repeating)),
    ));
}

fn move_player(
    input: Res<ButtonInput<KeyCode>>,
    movement: Res<MovementInput>,
    time: Res<Time>,
    player: Single<(&mut Transform, &mut AnimationState), With<Player>>,
) {
    let (mut transform, mut animation) = player.into_inner();
    let mut direction = movement.0;
    if input.pressed(KeyCode::ArrowLeft) {
        direction.x -= 1.0;
    }
    if input.pressed(KeyCode::ArrowRight) {
        direction.x += 1.0;
    }
    if input.pressed(KeyCode::ArrowUp) {
        direction.y += 1.0;
    }
    if input.pressed(KeyCode::ArrowDown) {
        direction.y -= 1.0;
    }
    animation.moving = direction != Vec2::ZERO;
    if animation.moving {
        animation.facing = if direction.x.abs() > direction.y.abs() {
            if direction.x < 0.0 {
                Facing::Left
            } else {
                Facing::Right
            }
        } else if direction.y < 0.0 {
            Facing::Down
        } else {
            Facing::Up
        };

        let delta = direction.clamp_length_max(1.0) * MOVE_SPEED * time.delta_secs();
        transform.translation.x += delta.x;
        transform.translation.y += delta.y;
    }
}

fn animate_player(
    time: Res<Time>,
    player: Single<(&mut Sprite, &mut AnimationTimer, &mut AnimationState), With<Player>>,
) {
    let (mut sprite, mut timer, mut animation) = player.into_inner();
    let Some(atlas) = sprite.texture_atlas.as_mut() else {
        return;
    };

    timer.tick(time.delta());
    let first_frame = atlas_index_for(animation.facing, 0);

    if !animation.moving {
        atlas.index = first_frame;
        animation.was_moving = false;
        return;
    }

    if !animation.was_moving {
        timer.reset();
        atlas.index = first_frame;
    } else if !(first_frame..first_frame + WALK_FRAMES).contains(&atlas.index) {
        timer.reset();
        atlas.index = first_frame;
    } else if timer.just_finished() {
        let frame = (atlas.index - first_frame + 1) % WALK_FRAMES;
        atlas.index = atlas_index_for(animation.facing, frame);
    }

    animation.was_moving = true;
}
