use bevy_procedural_tilemaps::prelude::*;

pub struct TerrainSockets {
    pub dirt: DirtLayerSockets,
    pub void: Socket,
    pub cyan: PatchLayerSockets,
    pub magenta: MagentaLayerSockets,
    pub coolant: CoolantLayerSockets,
    pub props: PropsLayerSockets,
}

pub struct DirtLayerSockets {
    pub layer_up: Socket,
    pub layer_down: Socket,
    pub material: Socket,
}

pub struct PatchLayerSockets {
    pub layer_up: Socket,
    pub layer_down: Socket,
    pub material: Socket,
    pub void_and_patch: Socket,
    pub patch_and_void: Socket,
    pub fill_up: Socket,
}

pub struct MagentaLayerSockets {
    pub layer_up: Socket,
    pub layer_down: Socket,
    pub fill_down: Socket,
}

pub struct CoolantLayerSockets {
    pub layer_up: Socket,
    pub layer_down: Socket,
    pub material: Socket,
    pub void_and_coolant: Socket,
    pub coolant_and_void: Socket,
    pub ground_up: Socket,
}

pub struct PropsLayerSockets {
    pub layer_up: Socket,
    pub layer_down: Socket,
    pub props_down: Socket,
    pub cyber_tree_1_base: Socket,
    pub cyber_tree_2_base: Socket,
}

pub fn create_sockets(collection: &mut SocketCollection) -> TerrainSockets {
    let mut socket = || collection.create();
    TerrainSockets {
        dirt: DirtLayerSockets {
            layer_up: socket(),
            layer_down: socket(),
            material: socket(),
        },
        void: socket(),
        cyan: PatchLayerSockets {
            layer_up: socket(),
            layer_down: socket(),
            material: socket(),
            void_and_patch: socket(),
            patch_and_void: socket(),
            fill_up: socket(),
        },
        magenta: MagentaLayerSockets {
            layer_up: socket(),
            layer_down: socket(),
            fill_down: socket(),
        },
        coolant: CoolantLayerSockets {
            layer_up: socket(),
            layer_down: socket(),
            material: socket(),
            void_and_coolant: socket(),
            coolant_and_void: socket(),
            ground_up: socket(),
        },
        props: PropsLayerSockets {
            layer_up: socket(),
            layer_down: socket(),
            props_down: socket(),
            cyber_tree_1_base: socket(),
            cyber_tree_2_base: socket(),
        },
    }
}
