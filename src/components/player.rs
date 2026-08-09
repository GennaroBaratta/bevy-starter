use bevy::prelude::*;

#[derive(Component)]
struct Player;

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(Startup, initialize_player)
    .add_systems(Update, move_player);
}

fn initialize_player(mut commands: Commands) {
    commands.spawn((
        Text2d::new("@"),
        TextFont {
            font_size: FontSize::Px(12.0),
            font: default(),
            ..default()
        },
        TextColor(Color::WHITE),
        Transform::from_translation(Vec3::ZERO),
        Player,
    ));
}

fn move_player(
    // "Bevy, give me keyboard input"
    input: Res<ButtonInput<KeyCode>>,
    // "Bevy, give me the game timer"
    time: Res<Time>,
    // "Bevy, give me the player's position"
    mut player_transform: Single<&mut Transform, With<Player>>,
) {
    let mut direction = Vec2::ZERO;
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
    if direction != Vec2::ZERO {
        let speed = 300.0; // pixels per second
        let delta = direction.normalize() * speed * time.delta_secs();
        player_transform.translation.x += delta.x;
        player_transform.translation.y += delta.y;
    }
}
