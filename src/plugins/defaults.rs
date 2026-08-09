use bevy::{asset::AssetMetaCheck, prelude::*};

use crate::map::map_pixel_dimensions;

const BACKGROUND_COLOR: Color = Color::srgb(0.015, 0.02, 0.04);

// Sets up the default plugins like windows, assets, etc

pub(crate) fn plugin(app: &mut App) {
    let map_size = map_pixel_dimensions();
    app.insert_resource(ClearColor(BACKGROUND_COLOR))
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    // Wasm builds will check for meta files (that don't exist) if this isn't set.
                    // This causes errors and even panics on web build on itch. You can enable this
                    // if you want to use meta files and are not building for the web
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                })
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Neon WFC District".into(),
                        resizable: false,
                        resolution: (map_size.x as u32, map_size.y as u32).into(),
                        canvas: Some("#bevy".to_owned()),
                        desired_maximum_frame_latency: core::num::NonZero::new(1u32),
                        fit_canvas_to_parent: true,
                        ..default()
                    }),
                    ..default()
                }),
        );
}
