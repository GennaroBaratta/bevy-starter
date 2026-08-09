use bevy::prelude::*;

#[derive(Resource, Default)]
pub(crate) struct MovementInput(pub Vec2);

#[derive(Component)]
struct JoystickBase;

#[derive(Component)]
struct JoystickStick;

#[derive(Resource, Default)]
struct JoystickTouch(Option<u64>);

const JOYSTICK_SIZE: f32 = 160.0;
const JOYSTICK_RADIUS: f32 = 80.0;
const STICK_SIZE: f32 = 64.0;
const STICK_CENTER: f32 = (JOYSTICK_SIZE - STICK_SIZE) / 2.0;

pub(crate) fn plugin(app: &mut App) {
    app.init_resource::<MovementInput>()
        .init_resource::<JoystickTouch>()
        .add_systems(Startup, spawn_joystick)
        .add_systems(Update, update_joystick);
}

fn spawn_joystick(mut commands: Commands) {
    commands.spawn((
        Node {
            width: px(JOYSTICK_SIZE),
            height: px(JOYSTICK_SIZE),
            position_type: PositionType::Absolute,
            border: UiRect::all(px(3)),
            border_radius: BorderRadius::MAX,
            ..default()
        },
        BackgroundColor(Color::srgba(0.55, 0.82, 0.9, 0.28)),
        BorderColor::all(Color::srgba(0.35, 0.7, 0.82, 0.8)),
        Visibility::Hidden,
        GlobalZIndex(10),
        JoystickBase,
        children![(
            Node {
                width: px(STICK_SIZE),
                height: px(STICK_SIZE),
                position_type: PositionType::Absolute,
                left: px(STICK_CENTER),
                top: px(STICK_CENTER),
                border: UiRect::all(px(3)),
                border_radius: BorderRadius::MAX,
                ..default()
            },
            BackgroundColor(Color::srgba(0.92, 0.97, 0.98, 0.95)),
            BorderColor::all(Color::srgba(0.35, 0.7, 0.82, 1.0)),
            JoystickStick,
        )],
    ));
}

fn update_joystick(
    touches: Res<Touches>,
    mut movement: ResMut<MovementInput>,
    mut active_touch: ResMut<JoystickTouch>,
    base: Single<(&mut Node, &mut Visibility), With<JoystickBase>>,
    mut stick: Single<&mut Node, (With<JoystickStick>, Without<JoystickBase>)>,
) {
    if active_touch.0.is_none() {
        active_touch.0 = touches.iter_just_pressed().next().map(|touch| touch.id());
    }

    let (mut base, mut visibility) = base.into_inner();
    let Some(touch) = active_touch.0.and_then(|id| touches.get_pressed(id)) else {
        active_touch.0 = None;
        movement.0 = Vec2::ZERO;
        stick.left = px(STICK_CENTER);
        stick.top = px(STICK_CENTER);
        *visibility = Visibility::Hidden;
        return;
    };

    let offset = joystick_offset(touch.position() - touch.start_position());
    movement.0 = Vec2::new(offset.x, -offset.y) / JOYSTICK_RADIUS;
    base.left = px(touch.start_position().x - JOYSTICK_RADIUS);
    base.top = px(touch.start_position().y - JOYSTICK_RADIUS);
    stick.left = px(STICK_CENTER + offset.x);
    stick.top = px(STICK_CENTER + offset.y);
    *visibility = Visibility::Visible;
}

fn joystick_offset(drag: Vec2) -> Vec2 {
    drag.clamp_length_max(JOYSTICK_RADIUS)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;

    fn setup() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, plugin));
        app
    }

    #[test]
    fn plugin_registers_resources() {
        let app = setup();

        assert!(app.world().contains_resource::<MovementInput>());
    }

    #[test]
    fn joystick_stick_follows_drag_and_stays_inside_base() {
        assert_eq!(
            joystick_offset(Vec2::new(24.0, -12.0)),
            Vec2::new(24.0, -12.0)
        );
        assert_eq!(
            joystick_offset(Vec2::new(JOYSTICK_RADIUS * 2.0, 0.0)),
            Vec2::new(JOYSTICK_RADIUS, 0.0)
        );
    }
}
