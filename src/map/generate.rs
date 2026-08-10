use bevy::{
    platform::time::Instant,
    prelude::*,
    sprite_render::{AlphaMode2d, TileData, TileOrientation, TilemapChunk, TilemapChunkTileData},
};
use bevy_procedural_tilemaps::{
    prelude::*,
    proc_gen::generator::{
        model::{ModelInstance, ModelRotation},
        rules::Rules,
    },
};
use std::{collections::HashSet, sync::Arc, time::Duration};

use crate::{characters::CharacterSystems, components::player::Player};

use super::{
    assets::{TileAssets, load_assets, load_tileset},
    rules::build_world,
};

pub const GRID_X: u32 = 25;
pub const GRID_Y: u32 = 18;
pub const TILE_SIZE: f32 = 32.0;
const GRID_Z: u32 = 5;
const SUBGRID_X: u32 = 5;
const SUBGRID_Y: u32 = 6;
const SUBGRID_COLUMNS: u32 = GRID_X / SUBGRID_X;
const SUBGRID_ROWS: u32 = GRID_Y / SUBGRID_Y;
const STREAM_RADIUS: i32 = 3;
// Tune this budget on the slowest supported browser.
const GENERATION_STEPS_PER_FRAME: usize = 8;
const GENERATION_TIME_BUDGET: Duration = Duration::from_millis(2);

type ChunkGenerator = Generator<Cartesian3D, CartesianGrid<Cartesian3D>>;

#[derive(Component)]
struct MapChunk {
    coordinate: IVec2,
    tiles: [Vec<Option<TileData>>; GRID_Z as usize],
    completed_subchunks: u32,
}

#[derive(Component)]
struct MapSubChunk {
    chunk: Entity,
    origin: UVec2,
}

#[derive(Component, Default)]
struct ChunkBuildQueue {
    next: u32,
}

#[derive(Resource)]
struct StreamingWorld {
    rules: Arc<Rules<Cartesian3D>>,
    tileset: Handle<Image>,
    assets: TileAssets,
    generated_chunks: HashSet<IVec2>,
}

pub(crate) struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_streaming_world).add_systems(
            Update,
            (
                stream_chunks, //.after(CharacterSystems::Update),
                advance_chunk_generation,
                start_subchunk_generation,
            )
                .chain(),
        );
    }
}

pub fn map_pixel_dimensions() -> Vec2 {
    Vec2::new(TILE_SIZE * GRID_X as f32, TILE_SIZE * GRID_Y as f32)
}

fn setup_streaming_world(mut commands: Commands, asset_server: Res<AssetServer>) {
    let (asset_definitions, models, socket_collection) = build_world();
    let rules = RulesBuilder::new_cartesian_3d(models, socket_collection)
        .with_rotation_axis(Direction::ZForward)
        .build()
        .expect("cyberpunk tile rules should be valid");

    commands.insert_resource(StreamingWorld {
        rules: Arc::new(rules),
        tileset: load_tileset(&asset_server),
        assets: load_assets(asset_definitions),
        generated_chunks: HashSet::new(),
    });
}

fn stream_chunks(
    mut commands: Commands,
    player: Single<&Transform, With<Player>>,
    mut world: ResMut<StreamingWorld>,
) {
    let center = chunk_at(player.translation.truncate());
    for distance in 0..=STREAM_RADIUS {
        for y in -distance..=distance {
            for x in -distance..=distance {
                if x.abs().max(y.abs()) != distance {
                    continue;
                }
                let chunk = center + IVec2::new(x, y);
                if world.generated_chunks.insert(chunk) {
                    spawn_chunk(&mut commands, chunk);
                }
            }
        }
    }
}

fn spawn_chunk(commands: &mut Commands, chunk: IVec2) {
    // ponytail: 5x6 WFC slices cap WASM setup work; share edge constraints if seams become visible.
    // ponytail: visited chunks stay loaded; despawn distant chunks if memory growth becomes measurable.
    let map_size = map_pixel_dimensions();
    let center = chunk.as_vec2() * map_size;

    let tile_count = (GRID_X * GRID_Y) as usize;
    commands.spawn((
        MapChunk {
            coordinate: chunk,
            tiles: std::array::from_fn(|_| vec![None; tile_count]),
            completed_subchunks: 0,
        },
        ChunkBuildQueue::default(),
        Transform::from_xyz(
            center.x - map_size.x / 2.0,
            center.y - map_size.y / 2.0,
            0.0,
        ),
        Visibility::default(),
    ));
}

fn start_subchunk_generation(
    mut commands: Commands,
    world: Res<StreamingWorld>,
    player: Single<&Transform, With<Player>>,
    mut chunks: Query<(Entity, &MapChunk, &mut ChunkBuildQueue)>,
    generators: Query<(), With<ChunkGenerator>>,
) {
    if !generators.is_empty() {
        return;
    }

    let player_chunk = chunk_at(player.translation.truncate());
    let Some((parent, _, mut queue)) = chunks.iter_mut().min_by_key(|(_, chunk, _)| {
        let offset = chunk.coordinate - player_chunk;
        offset.x.abs() + offset.y.abs()
    }) else {
        return;
    };

    let subgrid_x = queue.next % SUBGRID_COLUMNS;
    let subgrid_y = queue.next / SUBGRID_COLUMNS;
    let grid = CartesianGrid::new_cartesian_3d(SUBGRID_X, SUBGRID_Y, GRID_Z, false, false, false);
    let generator = build_generator(world.rules.clone(), grid.clone());
    commands.spawn((
        MapSubChunk {
            chunk: parent,
            origin: UVec2::new(subgrid_x * SUBGRID_X, subgrid_y * SUBGRID_Y),
        },
        ChildOf(parent),
        grid,
        generator,
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
    mut chunks: Query<(
        Entity,
        &MapSubChunk,
        &CartesianGrid<Cartesian3D>,
        &mut ChunkGenerator,
    )>,
    mut map_chunks: Query<&mut MapChunk>,
) {
    let Some((entity, subchunk, grid, mut generator)) = chunks.iter_mut().next() else {
        return;
    };

    let started = Instant::now();
    for _ in 0..GENERATION_STEPS_PER_FRAME {
        match generator.select_and_propagate() {
            Ok(GenerationStatus::Ongoing) => {}
            Ok(GenerationStatus::Done) => {
                let nodes = generator
                    .to_grid_data()
                    .expect("completed generator should contain grid data")
                    .iter()
                    .copied()
                    .collect::<Vec<_>>();
                let mut map_chunk = map_chunks
                    .get_mut(subchunk.chunk)
                    .expect("generated subchunk should retain its map chunk");
                apply_generated_tiles(
                    grid,
                    &nodes,
                    subchunk.origin,
                    &world.assets,
                    &mut map_chunk.tiles,
                );
                map_chunk.completed_subchunks += 1;
                if map_chunk.completed_subchunks == SUBGRID_COLUMNS * SUBGRID_ROWS {
                    let map_size = map_pixel_dimensions();
                    for (layer, tiles) in map_chunk.tiles.iter_mut().enumerate() {
                        commands.spawn((
                            ChildOf(subchunk.chunk),
                            TilemapChunk {
                                chunk_size: UVec2::new(GRID_X, GRID_Y),
                                tile_display_size: UVec2::splat(TILE_SIZE as u32),
                                tileset: world.tileset.clone(),
                                alpha_mode: AlphaMode2d::Blend,
                            },
                            TilemapChunkTileData(std::mem::take(tiles)),
                            Transform::from_xyz(
                                map_size.x / 2.0,
                                map_size.y / 2.0,
                                layer as f32 + 0.5,
                            ),
                        ));
                    }
                }
                commands.entity(entity).despawn();
                break;
            }
            Err(error) => {
                warn!("Cyberpunk chunk generation contradicted; retrying: {error}");
                *generator = build_generator(world.rules.clone(), grid.clone());
                break;
            }
        }
        if started.elapsed() >= GENERATION_TIME_BUDGET {
            break;
        }
    }
}

fn apply_generated_tiles(
    grid: &CartesianGrid<Cartesian3D>,
    nodes: &[ModelInstance],
    origin: UVec2,
    assets: &TileAssets,
    layers: &mut [Vec<Option<TileData>>; GRID_Z as usize],
) {
    for (node_index, model) in nodes.iter().enumerate() {
        let position = grid.pos_from_index(node_index);
        let Some(model_assets) = assets.get(model.model_index) else {
            continue;
        };
        for asset in model_assets {
            let x = i64::from(origin.x + position.x) + i64::from(asset.grid_offset.dx);
            let y = i64::from(origin.y + position.y) + i64::from(asset.grid_offset.dy);
            let z = i64::from(position.z) + i64::from(asset.grid_offset.dz);
            if x < 0
                || x >= i64::from(GRID_X)
                || y < 0
                || y >= i64::from(GRID_Y)
                || z < 0
                || z >= i64::from(GRID_Z)
            {
                continue;
            }
            layers[z as usize][y as usize * GRID_X as usize + x as usize] = Some(TileData {
                tileset_index: asset.tileset_index,
                orientation: tile_orientation(model.rotation),
                ..default()
            });
        }
    }
}

fn tile_orientation(rotation: ModelRotation) -> TileOrientation {
    match rotation {
        ModelRotation::Rot0 => TileOrientation::Default,
        ModelRotation::Rot90 => TileOrientation::Rotate90,
        ModelRotation::Rot180 => TileOrientation::Rotate180,
        ModelRotation::Rot270 => TileOrientation::Rotate270,
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
    fn streaming_keeps_a_three_block_radius_around_the_player() {
        let mut app = test_app(HashSet::new());
        let player = app.world_mut().spawn((Player, Transform::default())).id();
        let diameter = (STREAM_RADIUS * 2 + 1) as usize;

        app.update();
        assert_eq!(chunk_count(&mut app), diameter * diameter);
        assert_eq!(component_count::<TilemapChunk>(&mut app), 0);

        app.world_mut()
            .get_mut::<Transform>(player)
            .unwrap()
            .translation
            .x = map_pixel_dimensions().x;
        app.update();

        assert_eq!(chunk_count(&mut app), diameter * diameter + diameter);
    }

    #[test]
    fn streaming_batches_each_region_into_five_tilemap_layers() {
        let mut prefetched = HashSet::new();
        for y in -STREAM_RADIUS..=STREAM_RADIUS {
            for x in -STREAM_RADIUS..=STREAM_RADIUS {
                let chunk = IVec2::new(x, y);
                if chunk != IVec2::ZERO {
                    prefetched.insert(chunk);
                }
            }
        }
        let mut app = test_app(prefetched);
        let player = app.world_mut().spawn((Player, Transform::default())).id();

        let mut initial_completed = false;
        for _ in 0..160 {
            app.update();
            if streaming_settled(&mut app, 1) {
                initial_completed = true;
                break;
            }
            assert_eq!(rendered_tile_count(&mut app), 0);
        }
        assert!(initial_completed);
        assert_eq!(component_count::<TilemapChunk>(&mut app), GRID_Z as usize);
        assert_eq!(
            app.world_mut()
                .query_filtered::<Entity, (With<MapChunk>, With<Visibility>)>()
                .iter(app.world())
                .count(),
            1
        );
        assert_eq!(component_count::<Sprite>(&mut app), 0);
        let initial_tile_count = rendered_tile_count(&mut app);
        assert!(initial_tile_count > 0);

        let next_chunk = IVec2::new(STREAM_RADIUS + 1, 0);
        {
            let mut world = app.world_mut().resource_mut::<StreamingWorld>();
            for y in -STREAM_RADIUS..=STREAM_RADIUS {
                for x in -STREAM_RADIUS..=STREAM_RADIUS {
                    let chunk = IVec2::X + IVec2::new(x, y);
                    if chunk != next_chunk {
                        world.generated_chunks.insert(chunk);
                    }
                }
            }
        }
        app.world_mut()
            .get_mut::<Transform>(player)
            .unwrap()
            .translation
            .x = map_pixel_dimensions().x;

        let mut completed = false;
        for _ in 0..160 {
            app.update();
            if streaming_settled(&mut app, 2) {
                completed = true;
                break;
            }
            assert_eq!(rendered_tile_count(&mut app), initial_tile_count);
        }
        assert!(completed);
        assert_eq!(
            component_count::<TilemapChunk>(&mut app),
            (2 * GRID_Z) as usize
        );
        assert_eq!(component_count::<Sprite>(&mut app), 0);
        assert!(rendered_tile_count(&mut app) > initial_tile_count);
    }

    fn test_app(generated_chunks: HashSet<IVec2>) -> App {
        use bevy::sprite_render::{
            TilemapChunkMaterial, TilemapChunkMeshCache, TilemapChunkPlugin,
        };

        let (asset_definitions, models, sockets) = build_world();
        let rules = RulesBuilder::new_cartesian_3d(models, sockets)
            .with_rotation_axis(Direction::ZForward)
            .build()
            .unwrap();
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(AssetPlugin::default())
            .init_asset::<Image>()
            .init_asset::<Mesh>()
            .init_asset::<TilemapChunkMaterial>()
            .init_resource::<TilemapChunkMeshCache>()
            .add_plugins(TilemapChunkPlugin)
            .insert_resource(StreamingWorld {
                rules: Arc::new(rules),
                tileset: Handle::default(),
                assets: load_assets(asset_definitions),
                generated_chunks,
            })
            .add_systems(
                Update,
                (
                    stream_chunks,
                    advance_chunk_generation,
                    start_subchunk_generation,
                )
                    .chain(),
            );
        app
    }

    fn chunk_count(app: &mut App) -> usize {
        app.world_mut()
            .query_filtered::<Entity, With<MapChunk>>()
            .iter(app.world())
            .count()
    }

    fn component_count<T: Component>(app: &mut App) -> usize {
        app.world_mut()
            .query_filtered::<Entity, With<T>>()
            .iter(app.world())
            .count()
    }

    fn rendered_tile_count(app: &mut App) -> usize {
        app.world_mut()
            .query::<&TilemapChunkTileData>()
            .iter(app.world())
            .map(|tiles| tiles.iter().flatten().count())
            .sum()
    }

    fn streaming_settled(app: &mut App, expected_chunks: usize) -> bool {
        chunk_count(app) == expected_chunks
            && component_count::<ChunkBuildQueue>(app) == 0
            && component_count::<ChunkGenerator>(app) == 0
    }
}
