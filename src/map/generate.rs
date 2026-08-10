use bevy::prelude::*;
use bevy_procedural_tilemaps::{
    prelude::*,
    proc_gen::generator::{model::ModelInstance, rules::Rules},
    spawner::spawn_node,
};
use std::{collections::HashSet, sync::Arc};

use crate::{characters::CharacterSystems, components::player::Player};

use super::{
    assets::{load_assets, prepare_tilemap_handles},
    rules::build_world,
};

pub const GRID_X: u32 = 25;
pub const GRID_Y: u32 = 18;
pub const TILE_SIZE: f32 = 32.0;
const GRID_Z: u32 = 5;
const NODE_SIZE: Vec3 = Vec3::new(TILE_SIZE, TILE_SIZE, 1.0);
const SUBGRID_X: u32 = 5;
const SUBGRID_Y: u32 = 6;
const SUBGRID_COLUMNS: u32 = GRID_X / SUBGRID_X;
const SUBGRID_ROWS: u32 = GRID_Y / SUBGRID_Y;
// Tune these budgets on the slowest supported browser.
const GENERATION_STEPS_PER_FRAME: usize = 64;
const MAX_STREAMED_ENTITIES_PER_FRAME: usize = 128;

type ChunkGenerator = Generator<Cartesian3D, CartesianGrid<Cartesian3D>>;

#[derive(Component)]
struct MapChunk;

#[derive(Component)]
struct MapSubChunk;

#[derive(Component, Default)]
struct ChunkBuildQueue {
    next: u32,
}

#[derive(Component)]
struct ChunkSpawnQueue {
    nodes: Vec<ModelInstance>,
    next: usize,
}

#[derive(Resource)]
struct StreamingWorld {
    rules: Arc<Rules<Cartesian3D>>,
    spawner: NodesSpawner<Sprite>,
    generated_chunks: HashSet<IVec2>,
}

pub(crate) struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_streaming_world).add_systems(
            Update,
            (
                stream_chunks.after(CharacterSystems::Update),
                start_subchunk_generation,
                advance_chunk_generation,
                spawn_chunk_nodes,
            )
                .chain(),
        );
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

    // ponytail: 5x6 WFC slices cap WASM setup work; share edge constraints if seams become visible.
    // ponytail: visited chunks stay loaded; despawn distant chunks if memory growth becomes measurable.
    let map_size = map_pixel_dimensions();
    let center = chunk.as_vec2() * map_size;

    commands.spawn((
        MapChunk,
        ChunkBuildQueue::default(),
        Transform::from_xyz(
            center.x - map_size.x / 2.0,
            center.y - map_size.y / 2.0,
            0.0,
        ),
    ));
}

fn start_subchunk_generation(
    mut commands: Commands,
    world: Res<StreamingWorld>,
    mut chunks: Query<(Entity, &mut ChunkBuildQueue), With<MapChunk>>,
) {
    let Some((parent, mut queue)) = chunks.iter_mut().next() else {
        return;
    };

    let subgrid_x = queue.next % SUBGRID_COLUMNS;
    let subgrid_y = queue.next / SUBGRID_COLUMNS;
    let grid = CartesianGrid::new_cartesian_3d(SUBGRID_X, SUBGRID_Y, GRID_Z, false, false, false);
    let generator = build_generator(world.rules.clone(), grid.clone());
    commands.spawn((
        MapSubChunk,
        ChildOf(parent),
        Transform::from_xyz(
            subgrid_x as f32 * SUBGRID_X as f32 * TILE_SIZE,
            subgrid_y as f32 * SUBGRID_Y as f32 * TILE_SIZE,
            0.0,
        ),
        grid,
        generator,
        world.spawner.clone(),
    ));

    queue.next += 1;
    if queue.next == SUBGRID_COLUMNS * SUBGRID_ROWS {
        commands.entity(parent).remove::<ChunkBuildQueue>();
    }
}

fn build_generator(
    rules: Arc<Rules<Cartesian3D>>,
    grid: CartesianGrid<Cartesian3D>,
) -> ChunkGenerator {
    GeneratorBuilder::new()
        .with_shared_rules(rules)
        .with_grid(grid)
        .with_rng(RngMode::RandomSeed)
        .with_node_heuristic(NodeSelectionHeuristic::MinimumRemainingValue)
        .with_model_heuristic(ModelSelectionHeuristic::WeightedProbability)
        .build()
        .expect("cyberpunk chunk generator should initialize")
}

fn advance_chunk_generation(
    mut commands: Commands,
    world: Res<StreamingWorld>,
    mut chunks: Query<
        (Entity, &CartesianGrid<Cartesian3D>, &mut ChunkGenerator),
        With<MapSubChunk>,
    >,
) {
    // ponytail: one pending chunk advances per frame; add a FIFO if teleports can queue chunks.
    let Some((entity, grid, mut generator)) = chunks.iter_mut().next() else {
        return;
    };

    for _ in 0..GENERATION_STEPS_PER_FRAME {
        match generator.select_and_propagate() {
            Ok(GenerationStatus::Ongoing) => {}
            Ok(GenerationStatus::Done) => {
                let nodes = generator
                    .to_grid_data()
                    .expect("completed generator should contain grid data")
                    .iter()
                    .copied()
                    .collect();
                commands
                    .entity(entity)
                    .remove::<ChunkGenerator>()
                    .insert(ChunkSpawnQueue { nodes, next: 0 });
                break;
            }
            Err(error) => {
                warn!("Cyberpunk chunk generation contradicted; retrying: {error}");
                *generator = build_generator(world.rules.clone(), grid.clone());
                break;
            }
        }
    }
}

fn spawn_chunk_nodes(
    mut commands: Commands,
    mut chunks: Query<
        (
            Entity,
            &CartesianGrid<Cartesian3D>,
            &NodesSpawner<Sprite>,
            &mut ChunkSpawnQueue,
        ),
        With<MapSubChunk>,
    >,
) {
    let Some((entity, grid, spawner, mut queue)) = chunks.iter_mut().next() else {
        return;
    };

    let mut examined_nodes = 0;
    let mut spawned_entities = 0;
    while queue.next < queue.nodes.len() && examined_nodes < MAX_STREAMED_ENTITIES_PER_FRAME {
        let model = queue.nodes[queue.next];
        let asset_count = spawner.assets.get(&model.model_index).map_or(0, Vec::len);
        if spawned_entities > 0 && spawned_entities + asset_count > MAX_STREAMED_ENTITIES_PER_FRAME
        {
            break;
        }

        spawn_node(&mut commands, entity, grid, spawner, &model, queue.next);
        queue.next += 1;
        examined_nodes += 1;
        spawned_entities += asset_count;
    }

    if queue.next == queue.nodes.len() {
        commands.entity(entity).remove::<ChunkSpawnQueue>();
    }
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
        assert_eq!(GRID_X % SUBGRID_X, 0);
        assert_eq!(GRID_Y % SUBGRID_Y, 0);
        assert_eq!(SUBGRID_COLUMNS * SUBGRID_ROWS, 15);
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

    #[test]
    fn streaming_work_is_spread_across_frames() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(AssetPlugin::default())
            .init_asset::<Image>()
            .init_asset::<TextureAtlasLayout>()
            .add_plugins(MapPlugin);
        let player = app.world_mut().spawn((Player, Transform::default())).id();

        let mut initial_completed = false;
        for _ in 0..160 {
            app.update();
            if streaming_settled(&mut app, 1) {
                initial_completed = true;
                break;
            }
        }
        assert!(initial_completed);

        let initial_sprite_count = sprite_count(&mut app);
        app.world_mut()
            .get_mut::<Transform>(player)
            .unwrap()
            .translation
            .x = map_pixel_dimensions().x;

        let mut previous_sprite_count = initial_sprite_count;
        let mut completed = false;
        for frame in 0..160 {
            app.update();
            let sprite_count = sprite_count(&mut app);
            let spawned_this_frame = sprite_count - previous_sprite_count;
            assert!(
                spawned_this_frame <= MAX_STREAMED_ENTITIES_PER_FRAME,
                "frame {frame} spawned {spawned_this_frame} entities"
            );
            previous_sprite_count = sprite_count;

            if streaming_settled(&mut app, 2) {
                completed = true;
                break;
            }
        }
        assert!(completed);

        assert!(sprite_count(&mut app) > initial_sprite_count);
    }

    fn chunk_count(app: &mut App) -> usize {
        app.world_mut()
            .query_filtered::<Entity, With<MapChunk>>()
            .iter(app.world())
            .count()
    }

    fn sprite_count(app: &mut App) -> usize {
        app.world_mut()
            .query_filtered::<Entity, With<Sprite>>()
            .iter(app.world())
            .count()
    }

    fn component_count<T: Component>(app: &mut App) -> usize {
        app.world_mut()
            .query_filtered::<Entity, With<T>>()
            .iter(app.world())
            .count()
    }

    fn streaming_settled(app: &mut App, expected_chunks: usize) -> bool {
        chunk_count(app) == expected_chunks
            && component_count::<ChunkBuildQueue>(app) == 0
            && component_count::<ChunkGenerator>(app) == 0
            && component_count::<ChunkSpawnQueue>(app) == 0
    }
}
