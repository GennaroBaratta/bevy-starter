use bevy::{ecs::system::EntityCommands, prelude::*};
use bevy_procedural_tilemaps::prelude::*;

use super::tilemap::TILEMAP;

#[derive(Clone)]
pub struct SpawnableAsset {
    sprite_name: &'static str,
    grid_offset: GridDelta,
    offset: Vec3,
    components_spawner: fn(&mut EntityCommands),
}

impl SpawnableAsset {
    pub fn new(sprite_name: &'static str) -> Self {
        Self {
            sprite_name,
            grid_offset: GridDelta::new(0, 0, 0),
            offset: Vec3::ZERO,
            components_spawner: |_| {},
        }
    }

    pub fn with_grid_offset(mut self, grid_offset: GridDelta) -> Self {
        self.grid_offset = grid_offset;
        self
    }
}

#[derive(Clone)]
pub struct TilemapHandles {
    image: Handle<Image>,
    layout: Handle<TextureAtlasLayout>,
}

impl TilemapHandles {
    fn sprite(&self, atlas_index: usize) -> Sprite {
        Sprite::from_atlas_image(
            self.image.clone(),
            TextureAtlas::from(self.layout.clone()).with_index(atlas_index),
        )
    }
}

pub fn prepare_tilemap_handles(
    asset_server: &AssetServer,
    atlas_layouts: &mut Assets<TextureAtlasLayout>,
) -> TilemapHandles {
    let image = asset_server.load("tile_layers/cyberpunk_tilemap.png");
    let mut layout = TextureAtlasLayout::new_empty(TILEMAP.atlas_size());
    for index in 0..TILEMAP.sprites.len() {
        layout.add_texture(TILEMAP.sprite_rect(index));
    }

    TilemapHandles {
        image,
        layout: atlas_layouts.add(layout),
    }
}

pub fn load_assets(
    handles: &TilemapHandles,
    definitions: Vec<Vec<SpawnableAsset>>,
) -> ModelsAssets<Sprite> {
    let mut models_assets = ModelsAssets::new();
    for (model_index, assets) in definitions.into_iter().enumerate() {
        for asset in assets {
            let atlas_index = TILEMAP
                .sprite_index(asset.sprite_name)
                .unwrap_or_else(|| panic!("Unknown atlas sprite '{}'", asset.sprite_name));
            models_assets.add(
                model_index,
                ModelAsset {
                    assets_bundle: handles.sprite(atlas_index),
                    grid_offset: asset.grid_offset,
                    world_offset: asset.offset,
                    spawn_commands: asset.components_spawner,
                },
            );
        }
    }
    models_assets
}
