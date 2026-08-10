use bevy::{
    image::{ImageArrayLayout, ImageLoaderSettings},
    prelude::*,
};
use bevy_procedural_tilemaps::prelude::GridDelta;

use super::tilemap::TILEMAP;

#[derive(Clone)]
pub struct SpawnableAsset {
    sprite_name: &'static str,
    grid_offset: GridDelta,
}

impl SpawnableAsset {
    pub fn new(sprite_name: &'static str) -> Self {
        Self {
            sprite_name,
            grid_offset: GridDelta::new(0, 0, 0),
        }
    }

    pub fn with_grid_offset(mut self, grid_offset: GridDelta) -> Self {
        self.grid_offset = grid_offset;
        self
    }
}

#[derive(Clone, Copy)]
pub struct TileAsset {
    pub tileset_index: u16,
    pub grid_offset: GridDelta,
}

pub type TileAssets = Vec<Vec<TileAsset>>;

pub fn load_tileset(asset_server: &AssetServer) -> Handle<Image> {
    let tile_size = TILEMAP.tile_size();
    asset_server
        .load_builder()
        .with_settings(move |settings: &mut ImageLoaderSettings| {
            settings.array_layout = Some(ImageArrayLayout::GridSize {
                tile_width_pixels: tile_size.x,
                tile_height_pixels: tile_size.y,
            });
        })
        .load("tile_layers/cyberpunk_tilemap_v2.png")
}

pub fn load_assets(definitions: Vec<Vec<SpawnableAsset>>) -> TileAssets {
    definitions
        .into_iter()
        .map(|assets| {
            assets
                .into_iter()
                .map(|asset| TileAsset {
                    tileset_index: TILEMAP
                        .sprite_index(asset.sprite_name)
                        .unwrap_or_else(|| panic!("Unknown atlas sprite '{}'", asset.sprite_name))
                        as u16,
                    grid_offset: asset.grid_offset,
                })
                .collect()
        })
        .collect()
}
