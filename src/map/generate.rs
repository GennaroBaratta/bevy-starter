use bevy::prelude::*;
use bevy_procedural_tilemaps::{prelude::*, proc_gen::generator::rules::Rules};
use std::{collections::HashSet, sync::Arc};

use crate::components::player::Player;

use super::{
    assets::{load_assets, prepare_tilemap_handles},
    rules::build_world,
};

pub const GRID_X: u32 = 25;
pub const GRID_Y: u32 = 18;
pub const TILE_SIZE: f32 = 32.0;
const GRID_Z: u32 = 5;
const NODE_SIZE: Vec3 = Vec3::new(TILE_SIZE, TILE_SIZE, 1.0);

#[derive(Component)]
struct MapChunk;

#[derive(Resource)]
struct StreamingWorld {
    rules: Arc<Rules<Cartesian3D>>,
    spawner: NodesSpawner<Sprite>,
    generated_chunks: HashSet<IVec2>,
}

pub(crate) struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ProcGenSimplePlugin::<Cartesian3D, Sprite>::default())
            .add_systems(Startup, setup_streaming_world)
            .add_systems(Update, stream_chunks);
    }
}

pub fn map_pixel_dimensions() -> Vec2 {
    Vec2::new(TILE_SIZE * GRID_X as f32, TILE_SIZE * GRID_Y as f32)
}

fn setup_streaming_world(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let (asset_definitions, models, socket_collection) = build_world();
    let rules = RulesBuilder::new_cartesian_3d(models, socket_collection)
        .with_rotation_axis(Direction::ZForward)
        .build()
        .expect("cyberpunk tile rules should be valid");
    let handles = prepare_tilemap_handles(&asset_server, &mut atlas_layouts);
    let model_assets = load_assets(&handles, asset_definitions);

    commands.insert_resource(StreamingWorld {
        rules: Arc::new(rules),
        spawner: NodesSpawner::new(model_assets, NODE_SIZE, Vec3::ONE).with_z_offset_from_y(true),
        generated_chunks: HashSet::new(),
    });
}

fn stream_chunks(
    mut commands: Commands,
    player: Single<&Transform, With<Player>>,
    mut world: ResMut<StreamingWorld>,
) {
    let chunk = chunk_at(player.translation.truncate());
    if !world.generated_chunks.insert(chunk) {
        return;
    }

    // ponytail: chunks generate independently; share border constraints if seams become visible.
    // ponytail: visited chunks stay loaded; despawn distant chunks if memory growth becomes measurable.
    let grid = CartesianGrid::new_cartesian_3d(GRID_X, GRID_Y, GRID_Z, false, false, false);
    let generator = GeneratorBuilder::new()
        .with_shared_rules(world.rules.clone())
        .with_grid(grid.clone())
        .with_rng(RngMode::RandomSeed)
        .with_node_heuristic(NodeSelectionHeuristic::MinimumRemainingValue)
        .with_model_heuristic(ModelSelectionHeuristic::WeightedProbability)
        .build()
        .expect("cyberpunk chunk generator should initialize");
    let map_size = map_pixel_dimensions();
    let center = chunk.as_vec2() * map_size;

    commands.spawn((
        MapChunk,
        Transform::from_xyz(
            center.x - map_size.x / 2.0,
            center.y - map_size.y / 2.0,
            0.0,
        ),
        grid,
        generator,
        world.spawner.clone(),
    ));
}

fn chunk_at(position: Vec2) -> IVec2 {
    let map_size = map_pixel_dimensions();
    ((position + map_size / 2.0) / map_size).floor().as_ivec2()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_dimensions_match_the_chapter_two_grid() {
        assert_eq!(map_pixel_dimensions(), Vec2::new(800.0, 576.0));
        assert_eq!(GRID_Z, 5);
        assert_eq!(chunk_at(Vec2::new(399.0, 287.0)), IVec2::ZERO);
        assert_eq!(chunk_at(Vec2::new(401.0, 0.0)), IVec2::X);
        assert_eq!(chunk_at(Vec2::new(0.0, 289.0)), IVec2::Y);
    }

    #[test]
    fn rules_generate_a_complete_five_layer_grid() {
        let (_, models, sockets) = build_world();
        let rules = RulesBuilder::new_cartesian_3d(models, sockets)
            .with_rotation_axis(Direction::ZForward)
            .build()
            .unwrap();
        let grid = CartesianGrid::new_cartesian_3d(GRID_X, GRID_Y, GRID_Z, false, false, false);
        let mut generator = GeneratorBuilder::new()
            .with_rules(rules)
            .with_grid(grid)
            .with_rng(RngMode::Seeded(7))
            .build()
            .unwrap();

        let (_, generated) = generator.generate_grid().unwrap();

        assert_eq!(
            generated.iter().count(),
            (GRID_X * GRID_Y * GRID_Z) as usize
        );
    }

    #[test]
    fn entering_a_new_region_spawns_one_chunk_once() {
        let mut app = App::new();
        let (_, models, sockets) = build_world();
        let rules = RulesBuilder::new_cartesian_3d(models, sockets)
            .with_rotation_axis(Direction::ZForward)
            .build()
            .unwrap();
        app.insert_resource(StreamingWorld {
            rules: Arc::new(rules),
            spawner: NodesSpawner::new(ModelsAssets::new(), NODE_SIZE, Vec3::ONE),
            generated_chunks: HashSet::new(),
        })
        .add_systems(Update, stream_chunks);
        let player = app.world_mut().spawn((Player, Transform::default())).id();

        app.update();
        app.update();
        assert_eq!(chunk_count(&mut app), 1);

        app.world_mut()
            .get_mut::<Transform>(player)
            .unwrap()
            .translation
            .x = map_pixel_dimensions().x;
        app.update();

        assert_eq!(chunk_count(&mut app), 2);
    }

    fn chunk_count(app: &mut App) -> usize {
        app.world_mut()
            .query_filtered::<Entity, With<MapChunk>>()
            .iter(app.world())
            .count()
    }
}
