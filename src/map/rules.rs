use bevy_procedural_tilemaps::prelude::*;

use super::{
    assets::SpawnableAsset,
    models::TerrainModelBuilder,
    sockets::{TerrainSockets, create_sockets},
};

const ROTATIONS: [ModelRotation; 4] = [
    ModelRotation::Rot0,
    ModelRotation::Rot90,
    ModelRotation::Rot180,
    ModelRotation::Rot270,
];

fn add_rotations(
    builder: &mut TerrainModelBuilder,
    template: &ModelTemplate<Cartesian3D>,
    sprites: [&'static str; 4],
) {
    for (rotation, sprite) in ROTATIONS.into_iter().zip(sprites) {
        builder.create_model(
            template.rotated(rotation, Direction::ZForward),
            vec![SpawnableAsset::new(sprite)],
        );
    }
}

fn build_dirt_layer(
    builder: &mut TerrainModelBuilder,
    sockets: &TerrainSockets,
    connections: &mut SocketCollection,
) {
    builder
        .create_model(
            SocketsCartesian3D::Simple {
                x_pos: sockets.dirt.material,
                x_neg: sockets.dirt.material,
                y_pos: sockets.dirt.material,
                y_neg: sockets.dirt.material,
                z_pos: sockets.dirt.layer_up,
                z_neg: sockets.dirt.layer_down,
            },
            vec![SpawnableAsset::new("asphalt")],
        )
        .with_weight(20.0_f32);
    connections.add_connection(sockets.dirt.material, [sockets.dirt.material]);
}

fn build_cyan_layer(
    builder: &mut TerrainModelBuilder,
    sockets: &TerrainSockets,
    connections: &mut SocketCollection,
) {
    builder.create_model(
        SocketsCartesian3D::Simple {
            x_pos: sockets.void,
            x_neg: sockets.void,
            y_pos: sockets.void,
            y_neg: sockets.void,
            z_pos: sockets.cyan.layer_up,
            z_neg: sockets.cyan.layer_down,
        },
        Vec::new(),
    );
    builder
        .create_model(
            SocketsCartesian3D::Multiple {
                x_pos: vec![sockets.cyan.material],
                x_neg: vec![sockets.cyan.material],
                y_pos: vec![sockets.cyan.material],
                y_neg: vec![sockets.cyan.material],
                z_pos: vec![sockets.cyan.layer_up, sockets.cyan.fill_up],
                z_neg: vec![sockets.cyan.layer_down],
            },
            vec![SpawnableAsset::new("cyan_center")],
        )
        .with_weight(5.0_f32);

    let outer = SocketsCartesian3D::Simple {
        x_pos: sockets.cyan.void_and_patch,
        x_neg: sockets.void,
        y_pos: sockets.void,
        y_neg: sockets.cyan.patch_and_void,
        z_pos: sockets.cyan.layer_up,
        z_neg: sockets.cyan.layer_down,
    }
    .to_template();
    let inner = SocketsCartesian3D::Simple {
        x_pos: sockets.cyan.patch_and_void,
        x_neg: sockets.cyan.material,
        y_pos: sockets.cyan.material,
        y_neg: sockets.cyan.void_and_patch,
        z_pos: sockets.cyan.layer_up,
        z_neg: sockets.cyan.layer_down,
    }
    .to_template();
    let side = SocketsCartesian3D::Simple {
        x_pos: sockets.cyan.void_and_patch,
        x_neg: sockets.cyan.patch_and_void,
        y_pos: sockets.void,
        y_neg: sockets.cyan.material,
        z_pos: sockets.cyan.layer_up,
        z_neg: sockets.cyan.layer_down,
    }
    .to_template();

    add_rotations(
        builder,
        &outer,
        [
            "cyan_outer_0",
            "cyan_outer_90",
            "cyan_outer_180",
            "cyan_outer_270",
        ],
    );
    add_rotations(
        builder,
        &inner,
        [
            "cyan_inner_0",
            "cyan_inner_90",
            "cyan_inner_180",
            "cyan_inner_270",
        ],
    );
    add_rotations(
        builder,
        &side,
        [
            "cyan_side_0",
            "cyan_side_90",
            "cyan_side_180",
            "cyan_side_270",
        ],
    );

    connections
        .add_rotated_connection(sockets.dirt.layer_up, vec![sockets.cyan.layer_down])
        .add_connections([
            (sockets.void, vec![sockets.void]),
            (sockets.cyan.material, vec![sockets.cyan.material]),
            (
                sockets.cyan.void_and_patch,
                vec![sockets.cyan.patch_and_void],
            ),
        ]);
}

fn build_magenta_layer(
    builder: &mut TerrainModelBuilder,
    sockets: &TerrainSockets,
    connections: &mut SocketCollection,
) {
    builder.create_model(
        SocketsCartesian3D::Simple {
            x_pos: sockets.void,
            x_neg: sockets.void,
            y_pos: sockets.void,
            y_neg: sockets.void,
            z_pos: sockets.magenta.layer_up,
            z_neg: sockets.magenta.layer_down,
        },
        Vec::new(),
    );
    builder
        .create_model(
            SocketsCartesian3D::Simple {
                x_pos: sockets.cyan.material,
                x_neg: sockets.cyan.material,
                y_pos: sockets.cyan.material,
                y_neg: sockets.cyan.material,
                z_pos: sockets.magenta.layer_up,
                z_neg: sockets.magenta.fill_down,
            },
            vec![SpawnableAsset::new("magenta_center")],
        )
        .with_weight(5.0_f32);

    let outer = SocketsCartesian3D::Simple {
        x_pos: sockets.cyan.void_and_patch,
        x_neg: sockets.void,
        y_pos: sockets.void,
        y_neg: sockets.cyan.patch_and_void,
        z_pos: sockets.magenta.layer_up,
        z_neg: sockets.magenta.fill_down,
    }
    .to_template();
    let inner = SocketsCartesian3D::Simple {
        x_pos: sockets.cyan.patch_and_void,
        x_neg: sockets.cyan.material,
        y_pos: sockets.cyan.material,
        y_neg: sockets.cyan.void_and_patch,
        z_pos: sockets.magenta.layer_up,
        z_neg: sockets.magenta.fill_down,
    }
    .to_template();
    let side = SocketsCartesian3D::Simple {
        x_pos: sockets.cyan.void_and_patch,
        x_neg: sockets.cyan.patch_and_void,
        y_pos: sockets.void,
        y_neg: sockets.cyan.material,
        z_pos: sockets.magenta.layer_up,
        z_neg: sockets.magenta.fill_down,
    }
    .to_template();

    add_rotations(
        builder,
        &outer,
        [
            "magenta_outer_0",
            "magenta_outer_90",
            "magenta_outer_180",
            "magenta_outer_270",
        ],
    );
    add_rotations(
        builder,
        &inner,
        [
            "magenta_inner_0",
            "magenta_inner_90",
            "magenta_inner_180",
            "magenta_inner_270",
        ],
    );
    add_rotations(
        builder,
        &side,
        [
            "magenta_side_0",
            "magenta_side_90",
            "magenta_side_180",
            "magenta_side_270",
        ],
    );

    connections
        .add_rotated_connection(sockets.cyan.layer_up, vec![sockets.magenta.layer_down])
        .add_rotated_connection(sockets.magenta.fill_down, vec![sockets.cyan.fill_up]);
}

fn build_coolant_layer(
    builder: &mut TerrainModelBuilder,
    sockets: &TerrainSockets,
    connections: &mut SocketCollection,
) {
    builder.create_model(
        SocketsCartesian3D::Multiple {
            x_pos: vec![sockets.void],
            x_neg: vec![sockets.void],
            y_pos: vec![sockets.void],
            y_neg: vec![sockets.void],
            z_pos: vec![sockets.coolant.layer_up, sockets.coolant.ground_up],
            z_neg: vec![sockets.coolant.layer_down],
        },
        Vec::new(),
    );

    const COOLANT_WEIGHT: f32 = 0.07;
    builder
        .create_model(
            SocketsCartesian3D::Simple {
                x_pos: sockets.coolant.material,
                x_neg: sockets.coolant.material,
                y_pos: sockets.coolant.material,
                y_neg: sockets.coolant.material,
                z_pos: sockets.coolant.layer_up,
                z_neg: sockets.coolant.layer_down,
            },
            vec![SpawnableAsset::new("coolant_center")],
        )
        .with_weight(10.0 * COOLANT_WEIGHT);

    let outer = SocketsCartesian3D::Simple {
        x_pos: sockets.coolant.void_and_coolant,
        x_neg: sockets.void,
        y_pos: sockets.void,
        y_neg: sockets.coolant.coolant_and_void,
        z_pos: sockets.coolant.layer_up,
        z_neg: sockets.coolant.layer_down,
    }
    .to_template()
    .with_weight(COOLANT_WEIGHT);
    let inner = SocketsCartesian3D::Simple {
        x_pos: sockets.coolant.coolant_and_void,
        x_neg: sockets.coolant.material,
        y_pos: sockets.coolant.material,
        y_neg: sockets.coolant.void_and_coolant,
        z_pos: sockets.coolant.layer_up,
        z_neg: sockets.coolant.layer_down,
    }
    .to_template()
    .with_weight(COOLANT_WEIGHT);
    let side = SocketsCartesian3D::Simple {
        x_pos: sockets.coolant.void_and_coolant,
        x_neg: sockets.coolant.coolant_and_void,
        y_pos: sockets.void,
        y_neg: sockets.coolant.material,
        z_pos: sockets.coolant.layer_up,
        z_neg: sockets.coolant.layer_down,
    }
    .to_template()
    .with_weight(COOLANT_WEIGHT);

    add_rotations(
        builder,
        &outer,
        [
            "coolant_outer_0",
            "coolant_outer_90",
            "coolant_outer_180",
            "coolant_outer_270",
        ],
    );
    add_rotations(
        builder,
        &inner,
        [
            "coolant_inner_0",
            "coolant_inner_90",
            "coolant_inner_180",
            "coolant_inner_270",
        ],
    );
    add_rotations(
        builder,
        &side,
        [
            "coolant_side_0",
            "coolant_side_90",
            "coolant_side_180",
            "coolant_side_270",
        ],
    );

    connections
        .add_connections([
            (sockets.coolant.material, vec![sockets.coolant.material]),
            (
                sockets.coolant.coolant_and_void,
                vec![sockets.coolant.void_and_coolant],
            ),
        ])
        .add_rotated_connection(sockets.magenta.layer_up, vec![sockets.coolant.layer_down]);
}

fn build_props_layer(
    builder: &mut TerrainModelBuilder,
    sockets: &TerrainSockets,
    connections: &mut SocketCollection,
) {
    builder.create_model(
        SocketsCartesian3D::Simple {
            x_pos: sockets.void,
            x_neg: sockets.void,
            y_pos: sockets.void,
            y_neg: sockets.void,
            z_pos: sockets.props.layer_up,
            z_neg: sockets.props.layer_down,
        },
        Vec::new(),
    );

    let prop = SocketsCartesian3D::Simple {
        x_pos: sockets.void,
        x_neg: sockets.void,
        y_pos: sockets.void,
        y_neg: sockets.void,
        z_pos: sockets.props.layer_up,
        z_neg: sockets.props.props_down,
    }
    .to_template();
    let plant = prop.clone().with_weight(0.025_f32);
    let rock = prop.clone().with_weight(0.008_f32);
    let scrap = prop.clone().with_weight(0.012_f32);

    builder.create_model(
        plant.clone(),
        vec![
            SpawnableAsset::new("small_neon_tree_bottom"),
            SpawnableAsset::new("small_neon_tree_top").with_grid_offset(GridDelta::new(0, 1, 0)),
        ],
    );

    add_big_tree(
        builder,
        sockets,
        sockets.props.cyber_tree_1_base,
        [
            "cyber_tree_1_bl",
            "cyber_tree_1_tl",
            "cyber_tree_1_br",
            "cyber_tree_1_tr",
        ],
    );
    add_big_tree(
        builder,
        sockets,
        sockets.props.cyber_tree_2_base,
        [
            "cyber_tree_2_bl",
            "cyber_tree_2_tl",
            "cyber_tree_2_br",
            "cyber_tree_2_tr",
        ],
    );

    for sprite in ["scrap_pile_1", "scrap_pile_2", "scrap_pile_3"] {
        builder.create_model(scrap.clone(), vec![SpawnableAsset::new(sprite)]);
    }
    for sprite in [
        "scrap_rock_1",
        "scrap_rock_2",
        "scrap_rock_3",
        "scrap_rock_4",
    ] {
        builder.create_model(rock.clone(), vec![SpawnableAsset::new(sprite)]);
    }
    for sprite in [
        "holo_plant_1",
        "holo_plant_2",
        "holo_plant_3",
        "holo_plant_4",
        "neon_pylon",
        "power_crate",
        "antenna",
    ] {
        builder.create_model(plant.clone(), vec![SpawnableAsset::new(sprite)]);
    }

    connections
        .add_connections([
            (
                sockets.props.cyber_tree_1_base,
                vec![sockets.props.cyber_tree_1_base],
            ),
            (
                sockets.props.cyber_tree_2_base,
                vec![sockets.props.cyber_tree_2_base],
            ),
        ])
        .add_rotated_connection(sockets.coolant.layer_up, vec![sockets.props.layer_down])
        .add_rotated_connection(sockets.props.props_down, vec![sockets.coolant.ground_up]);
}

fn add_big_tree(
    builder: &mut TerrainModelBuilder,
    sockets: &TerrainSockets,
    base_socket: Socket,
    sprites: [&'static str; 4],
) {
    let [bottom_left, top_left, bottom_right, top_right] = sprites;
    builder
        .create_model(
            SocketsCartesian3D::Simple {
                x_pos: base_socket,
                x_neg: sockets.void,
                y_pos: sockets.void,
                y_neg: sockets.void,
                z_pos: sockets.props.layer_up,
                z_neg: sockets.props.props_down,
            },
            vec![
                SpawnableAsset::new(bottom_left),
                SpawnableAsset::new(top_left).with_grid_offset(GridDelta::new(0, 1, 0)),
            ],
        )
        .with_weight(0.025_f32);
    builder
        .create_model(
            SocketsCartesian3D::Simple {
                x_pos: sockets.void,
                x_neg: base_socket,
                y_pos: sockets.void,
                y_neg: sockets.void,
                z_pos: sockets.props.layer_up,
                z_neg: sockets.props.props_down,
            },
            vec![
                SpawnableAsset::new(bottom_right),
                SpawnableAsset::new(top_right).with_grid_offset(GridDelta::new(0, 1, 0)),
            ],
        )
        .with_weight(0.025_f32);
}

pub fn build_world() -> (
    Vec<Vec<SpawnableAsset>>,
    ModelCollection<Cartesian3D>,
    SocketCollection,
) {
    let mut connections = SocketCollection::new();
    let sockets = create_sockets(&mut connections);
    let mut builder = TerrainModelBuilder::new();

    build_dirt_layer(&mut builder, &sockets, &mut connections);
    build_cyan_layer(&mut builder, &sockets, &mut connections);
    build_magenta_layer(&mut builder, &sockets, &mut connections);
    build_coolant_layer(&mut builder, &sockets, &mut connections);
    build_props_layer(&mut builder, &sockets, &mut connections);

    let (assets, models) = builder.into_parts();
    (assets, models, connections)
}
