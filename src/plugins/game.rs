use bevy::prelude::*;

use crate::map::MapPlugin;

pub(crate) fn plugin(app: &mut App) {
    app.add_plugins(MapPlugin);
}
