#![allow(unused)]

use avian2d::{math::*, prelude::*};
use bevy::{app::App, prelude::*};
use std::collections::HashSet;

use crate::{components::player::Player, prelude::random_number};

const CHUNK_SIZE: f32 = 800.0;
const SHAPES_PER_CHUNK: usize = 12;

// This is an example of the most simple plugin you can write, without
// having to implement any traits.
//
// If you wanted to toggle this plugin or configure it for the outside
// you can reach for a `PluginGroup`.

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(Startup, setup_background)
        .add_systems(Update, stream_background);
}

#[derive(Component)]
struct BackgroundShape;

#[derive(Resource)]
struct BackgroundWorld {
    generated_chunks: HashSet<IVec2>,
    shapes: [Handle<Mesh>; 3],
    materials: [Handle<ColorMaterial>; 3],
}

fn setup_background(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.insert_resource(BackgroundWorld {
        generated_chunks: HashSet::new(),
        shapes: [
            meshes.add(Circle::new(36.0)),
            meshes.add(Rectangle::new(64.0, 64.0)),
            meshes.add(RegularPolygon::new(42.0, 3)),
        ],
        materials: [
            materials.add(Color::srgba(0.05, 0.30, 0.34, 0.28)),
            materials.add(Color::srgba(0.18, 0.72, 0.67, 0.22)),
            materials.add(Color::srgba(0.55, 0.90, 0.78, 0.18)),
        ],
    });
}

fn stream_background(
    mut commands: Commands,
    player: Single<&Transform, With<Player>>,
    mut world: ResMut<BackgroundWorld>,
) {
    let chunk = chunk_at(player.translation.truncate());
    if !world.generated_chunks.insert(chunk) {
        return;
    }

    let center = chunk.as_vec2() * CHUNK_SIZE;
    for index in 0..SHAPES_PER_CHUNK {
        let kind = index % world.shapes.len();
        commands.spawn((
            Mesh2d(world.shapes[kind].clone()),
            MeshMaterial2d(world.materials[kind].clone()),
            Transform {
                translation: vec3(
                    center.x + random_number(-CHUNK_SIZE / 2.0, CHUNK_SIZE / 2.0),
                    center.y + random_number(-CHUNK_SIZE / 2.0, CHUNK_SIZE / 2.0),
                    -10.0,
                ),
                rotation: Quat::from_rotation_z(random_number(0.0, TAU)),
                scale: Vec3::splat(random_number(0.4, 1.4)),
            },
            BackgroundShape,
        ));
    }
}

fn chunk_at(position: Vec2) -> IVec2 {
    ((position + Vec2::splat(CHUNK_SIZE / 2.0)) / CHUNK_SIZE)
        .floor()
        .as_ivec2()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entering_a_new_chunk_spawns_it_once() {
        let mut app = App::new();
        app.insert_resource(Assets::<Mesh>::default())
            .insert_resource(Assets::<ColorMaterial>::default())
            .add_systems(Startup, setup_background)
            .add_systems(Update, stream_background);
        let player = app.world_mut().spawn((Player, Transform::default())).id();

        app.update();
        assert_eq!(shape_count(&mut app), SHAPES_PER_CHUNK);

        app.world_mut()
            .get_mut::<Transform>(player)
            .unwrap()
            .translation
            .x = CHUNK_SIZE;
        app.update();
        app.update();

        assert_eq!(shape_count(&mut app), SHAPES_PER_CHUNK * 2);
    }

    fn shape_count(app: &mut App) -> usize {
        app.world_mut()
            .query_filtered::<Entity, With<BackgroundShape>>()
            .iter(app.world())
            .count()
    }
}
