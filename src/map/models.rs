use bevy_procedural_tilemaps::prelude::*;

use super::assets::SpawnableAsset;

pub struct TerrainModelBuilder {
    models: ModelCollection<Cartesian3D>,
    assets: Vec<Vec<SpawnableAsset>>,
}

impl TerrainModelBuilder {
    pub fn new() -> Self {
        Self {
            models: ModelCollection::new(),
            assets: Vec::new(),
        }
    }

    pub fn create_model<T>(
        &mut self,
        template: T,
        assets: Vec<SpawnableAsset>,
    ) -> &mut Model<Cartesian3D>
    where
        T: Into<ModelTemplate<Cartesian3D>>,
    {
        let model = self.models.create(template);
        self.assets.push(assets);
        model
    }

    pub fn into_parts(self) -> (Vec<Vec<SpawnableAsset>>, ModelCollection<Cartesian3D>) {
        (self.assets, self.models)
    }
}
