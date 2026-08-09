use bevy::prelude::*;

#[derive(Component)]
struct Player;

pub(crate) fn plugin(app: &mut App) {
    app
    .add_systems(Startup, 
        initialize_player)
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
    // commands.spawn((
    //     Text2d::new("@"),
    //     TextFont {
    //         font_size: FontSize::Px(12.0),
    //         font: default(),
    //         ..default()
    //     },
    //     TextColor(Color::hsla(0.5, 1.0, 0.5, 1.0)),
    //     Transform::from_translation(vec3(1.0, 1.0, 1.0)),
    //     Player,
    // ));
}


fn move_player(
    // "Bevy, give me keyboard input"
    input: Res<ButtonInput<KeyCode>>,
    touches: Res<Touches>,
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
    touches.iter().for_each(|touch| {
        let drag = touch.position() - touch.start_position();
        direction += Vec2::new(drag.x, -drag.y) / 80.0;
    });
    if direction != Vec2::ZERO {
        let speed = 300.0; // pixels per second
        let delta = direction.clamp_length_max(1.0) * speed * time.delta_secs();
        player_transform.translation.x += delta.x;
        player_transform.translation.y += delta.y;
    }
}
