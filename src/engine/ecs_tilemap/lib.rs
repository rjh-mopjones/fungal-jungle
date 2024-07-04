#![allow(dead_code)]

//! Bevy ECS Tilemap plugin is a ECS driven tilemap rendering library. It's designed to be fast and highly customizable. Each tile is considered a unique entity and all tiles are stored in the game world.
//!
//!
//! ## Features
//! - A tile per entity.
//! - Fast rendering using a chunked approach.
//! - Layers and sparse tile maps.
//! - GPU powered animations.
//! - Isometric and Hexagonal tile maps.
//! - Initial support for Tiled file exports.
//! - Support for isometric and hexagon rendering.
//! - Built in animation support  – see [`animation` example](https://github.com/StarArawn/bevy_ecs_tilemap/blob/main/examples/animation.rs).
//! - Texture array support.

use bevy::prelude::{
    Bundle, Changed, Component, Deref, First, GlobalTransform, InheritedVisibility, Plugin, Query,
    Reflect, ReflectComponent, Transform, ViewVisibility, Visibility,
};

use bevy::prelude::Handle;

use engine::ecs_tilemap::map::{
    TilemapGridSize, TilemapSize, TilemapSpacing, TilemapTexture, TilemapTextureSize,
    TilemapTileSize, TilemapType,
};
use prelude::{TilemapId, TilemapRenderSettings};
use engine::ecs_tilemap::render::material::{MaterialTilemap, StandardTilemapMaterial};
use engine::ecs_tilemap::tiles::{
    AnimatedTile, TileColor, TileFlip, TilePos, TilePosOld, TileStorage, TileTextureIndex,
    TileVisible,
};

use bevy::render::{ExtractSchedule, RenderApp};
use crate::engine;

/// A module that allows pre-loading of atlases into array textures.
pub mod array_texture_preload;
/// A module which provides helper functions.
pub mod helpers;
/// A module which contains tilemap components.
pub mod map;
#[cfg(feature = "render")]
pub(crate) mod render;
/// A module which contains tile components.
pub mod tiles;

/// A bevy tilemap plugin. This must be included in order for everything to be rendered.
/// But is not necessary if you are running without a renderer.
pub struct TilemapPlugin;

impl Plugin for TilemapPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(engine::ecs_tilemap::render::TilemapRenderingPlugin);

        app.add_systems(First, update_changed_tile_positions);

        app.insert_resource(engine::ecs_tilemap::array_texture_preload::ArrayTextureLoader::default());
        let render_app = app.sub_app_mut(RenderApp);
        render_app.add_systems(ExtractSchedule, engine::ecs_tilemap::array_texture_preload::extract);

        app.register_type::<FrustumCulling>()
            .register_type::<TilemapId>()
            .register_type::<TilemapSize>()
            .register_type::<TilemapTexture>()
            .register_type::<TilemapTileSize>()
            .register_type::<TilemapGridSize>()
            .register_type::<TilemapSpacing>()
            .register_type::<TilemapTextureSize>()
            .register_type::<TilemapType>()
            .register_type::<TilePos>()
            .register_type::<TileTextureIndex>()
            .register_type::<TileColor>()
            .register_type::<TileVisible>()
            .register_type::<TileFlip>()
            .register_type::<TileStorage>()
            .register_type::<TilePosOld>()
            .register_type::<AnimatedTile>();
    }
}

#[derive(Component, Reflect, Debug, Clone, Copy, Deref)]
#[reflect(Component)]
pub struct FrustumCulling(pub bool);

impl Default for FrustumCulling {
    /// By default, `FrustumCulling` is `true`.
    fn default() -> Self {
        FrustumCulling(true)
    }
}

pub type TilemapBundle = MaterialTilemapBundle<StandardTilemapMaterial>;

/// The default tilemap bundle. All of the components within are required.
#[derive(Bundle, Debug, Default, Clone)]
pub struct MaterialTilemapBundle<M: MaterialTilemap> {
    pub grid_size: TilemapGridSize,
    pub map_type: TilemapType,
    pub size: TilemapSize,
    pub spacing: TilemapSpacing,
    pub storage: TileStorage,
    pub texture: TilemapTexture,
    pub tile_size: TilemapTileSize,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub render_settings: TilemapRenderSettings,
    /// User indication of whether an entity is visible
    pub visibility: Visibility,
    /// Algorithmically-computed indication of whether an entity is visible and should be extracted
    /// for rendering
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,
    /// User indication of whether tilemap should be frustum culled.
    pub frustum_culling: FrustumCulling,
    pub material: Handle<M>,
}

#[cfg(not(feature = "render"))]
/// The default tilemap bundle. All of the components within are required.
#[derive(Bundle, Debug, Default, Clone)]
pub struct StandardTilemapBundle {
    pub grid_size: TilemapGridSize,
    pub map_type: TilemapType,
    pub size: TilemapSize,
    pub spacing: TilemapSpacing,
    pub storage: TileStorage,
    pub texture: TilemapTexture,
    pub tile_size: TilemapTileSize,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub render_settings: TilemapRenderSettings,
    /// User indication of whether an entity is visible
    pub visibility: Visibility,
    /// Algorithmically-computed indication of whether an entity is visible and should be extracted
    /// for rendering
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,
    /// User indication of whether tilemap should be frustum culled.
    pub frustum_culling: FrustumCulling,
}

/// A module which exports commonly used dependencies.
pub mod prelude {
    pub use crate::engine::ecs_tilemap::array_texture_preload::*;
    pub use crate::engine::ecs_tilemap::helpers;
    pub use crate::engine::ecs_tilemap::helpers::filling::*;
    pub use crate::engine::ecs_tilemap::helpers::geometry::*;
    pub use crate::engine::ecs_tilemap::helpers::transform::*;
    pub use crate::engine::ecs_tilemap::map::*;
    pub use crate::engine::ecs_tilemap::render::material::MaterialTilemap;
    pub use crate::engine::ecs_tilemap::render::material::MaterialTilemapKey;
    pub use crate::engine::ecs_tilemap::render::material::MaterialTilemapPlugin;
    pub use crate::engine::ecs_tilemap::render::material::StandardTilemapMaterial;
    pub use crate::engine::ecs_tilemap::tiles::*;
    pub use crate::engine::ecs_tilemap::lib::MaterialTilemapBundle;
    pub use crate::engine::ecs_tilemap::lib::TilemapPlugin;
}

/// Updates old tile positions with the new values from the last frame.
fn update_changed_tile_positions(mut query: Query<(&TilePos, &mut TilePosOld), Changed<TilePos>>) {
    for (tile_pos, mut tile_pos_old) in query.iter_mut() {
        tile_pos_old.0 = *tile_pos;
    }
}
