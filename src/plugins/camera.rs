use crate::{characters::CharacterSystems, components::player::Player};
use bevy::prelude::*;

#[derive(Component)]
#[require(Camera2d)]
pub struct MainCamera;

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(Startup, initialize_camera)
        .add_systems(Update, follow_player.after(CharacterSystems::Update));
}

fn initialize_camera(mut commands: Commands) {
    commands.spawn(MainCamera);
}

//follow the player
fn follow_player(
    player_transform: Single<&Transform, (With<Player>, Without<MainCamera>)>,
    mut camera_transform: Single<&mut Transform, (With<MainCamera>, Without<Player>)>,
) {
    let player_position = player_transform.translation;
    let mut camera_position = camera_transform.translation;

    // Smoothly interpolate the camera position towards the player position
    let lerp_factor = 0.12; // Adjust this value for smoother or snappier movement
    camera_position.x += (player_position.x - camera_position.x) * lerp_factor;
    camera_position.y += (player_position.y - camera_position.y) * lerp_factor;

    camera_transform.translation = camera_position;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn camera_and_player_transform_queries_are_disjoint() {
        let mut app = App::new();
        app.add_systems(Update, follow_player);
        app.world_mut()
            .spawn((Player, Transform::from_xyz(10.0, 0.0, 0.0)));
        let camera = app
            .world_mut()
            .spawn((MainCamera, Transform::default()))
            .id();

        app.update();

        let camera_x = app.world().get::<Transform>(camera).unwrap().translation.x;
        assert!((camera_x - 1.2).abs() < 0.000_01);
    }
}
