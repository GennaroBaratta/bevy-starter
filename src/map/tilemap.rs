use bevy::math::{URect, UVec2};

pub struct TilemapDefinition {
    tile_size: u32,
    columns: u32,
    rows: u32,
    pub sprites: &'static [&'static str],
}

impl TilemapDefinition {
    pub const fn tile_size(&self) -> UVec2 {
        UVec2::splat(self.tile_size)
    }

    pub const fn atlas_size(&self) -> UVec2 {
        UVec2::new(self.columns * self.tile_size, self.rows * self.tile_size)
    }

    pub fn sprite_index(&self, name: &str) -> Option<usize> {
        self.sprites.iter().position(|sprite| *sprite == name)
    }

    pub fn sprite_rect(&self, index: usize) -> URect {
        let index = index as u32;
        let min = UVec2::new(
            index % self.columns * self.tile_size,
            index / self.columns * self.tile_size,
        );
        URect::from_corners(min, min + self.tile_size())
    }
}

pub const TILEMAP: TilemapDefinition = TilemapDefinition {
    tile_size: 32,
    columns: 8,
    rows: 8,
    sprites: &[
        "asphalt",
        "cyan_center",
        "cyan_outer_0",
        "cyan_outer_90",
        "cyan_outer_180",
        "cyan_outer_270",
        "cyan_inner_0",
        "cyan_inner_90",
        "cyan_inner_180",
        "cyan_inner_270",
        "cyan_side_0",
        "cyan_side_90",
        "cyan_side_180",
        "cyan_side_270",
        "magenta_center",
        "magenta_outer_0",
        "magenta_outer_90",
        "magenta_outer_180",
        "magenta_outer_270",
        "magenta_inner_0",
        "magenta_inner_90",
        "magenta_inner_180",
        "magenta_inner_270",
        "magenta_side_0",
        "magenta_side_90",
        "magenta_side_180",
        "magenta_side_270",
        "coolant_center",
        "coolant_outer_0",
        "coolant_outer_90",
        "coolant_outer_180",
        "coolant_outer_270",
        "coolant_inner_0",
        "coolant_inner_90",
        "coolant_inner_180",
        "coolant_inner_270",
        "coolant_side_0",
        "coolant_side_90",
        "coolant_side_180",
        "coolant_side_270",
        "cyber_tree_1_tl",
        "cyber_tree_1_tr",
        "cyber_tree_1_bl",
        "cyber_tree_1_br",
        "cyber_tree_2_tl",
        "cyber_tree_2_tr",
        "cyber_tree_2_bl",
        "cyber_tree_2_br",
        "holo_plant_1",
        "holo_plant_2",
        "holo_plant_3",
        "holo_plant_4",
        "scrap_rock_1",
        "scrap_rock_2",
        "scrap_rock_3",
        "scrap_rock_4",
        "small_neon_tree_top",
        "small_neon_tree_bottom",
        "scrap_pile_1",
        "scrap_pile_2",
        "scrap_pile_3",
        "neon_pylon",
        "power_crate",
        "antenna",
    ],
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_named_sprite_fits_inside_the_atlas() {
        assert_eq!(TILEMAP.sprites.len(), 64);
        assert!(TILEMAP.sprites.iter().all(|name| !name.is_empty()));
        assert_eq!(TILEMAP.sprite_rect(63).max, TILEMAP.atlas_size());
    }
}
