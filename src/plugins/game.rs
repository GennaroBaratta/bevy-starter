#![allow(unused)]

use avian2d::{math::*, prelude::*};
use bevy::{app::App, prelude::*};

use crate::prelude::random_number;

const BACKGROUND_SHAPE_COUNT: usize = 80;

// This is an example of the most simple plugin you can write, without
// having to implement any traits.
//
// If you wanted to toggle this plugin or configure it for the outside
// you can reach for a `PluginGroup`.

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn_background);
}

#[derive(Component)]
struct BackgroundShape;

fn spawn_background(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let shapes = [
        meshes.add(Circle::new(36.0)),
        meshes.add(Rectangle::new(64.0, 64.0)),
        meshes.add(RegularPolygon::new(42.0, 3)),
    ];
    let materials = [
        materials.add(Color::srgba(0.05, 0.30, 0.34, 0.28)),
        materials.add(Color::srgba(0.18, 0.72, 0.67, 0.22)),
        materials.add(Color::srgba(0.55, 0.90, 0.78, 0.18)),
    ];

    for index in 0..BACKGROUND_SHAPE_COUNT {
        let kind = index % shapes.len();
        commands.spawn((
            Mesh2d(shapes[kind].clone()),
            MeshMaterial2d(materials[kind].clone()),
            Transform {
                translation: vec3(
                    random_number(-1_600.0, 1_600.0),
                    random_number(-1_200.0, 1_200.0),
                    -10.0,
                ),
                rotation: Quat::from_rotation_z(random_number(0.0, TAU)),
                scale: Vec3::splat(random_number(0.4, 1.4)),
            },
            BackgroundShape,
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spawns_the_expected_number_of_background_shapes() {
        let mut app = App::new();
        app.insert_resource(Assets::<Mesh>::default())
            .insert_resource(Assets::<ColorMaterial>::default())
            .add_systems(Startup, spawn_background);

        app.update();

        assert_eq!(
            app.world_mut()
                .query_filtered::<Entity, With<BackgroundShape>>()
                .iter(app.world())
                .count(),
            BACKGROUND_SHAPE_COUNT
        );
    }
}
