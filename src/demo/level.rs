//! Spawn the main level.

use bevy::{
    prelude::*,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};
use bevy_trenchbroom::prelude::*;

use crate::{asset_tracking::LoadResource, screens::Screen};

pub(super) fn plugin(app: &mut App) {
    app.add_observer(setup_sphere);
    app.load_resource::<LevelAssets>();
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub(crate) struct Level;

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct LevelAssets {
    #[dependency]
    pub level: Handle<Scene>,
}

impl FromWorld for LevelAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            level: assets.load("maps/test.map#Scene"),
        }
    }
}

/// A system that spawns the main level.
pub fn spawn_level(
    mut commands: Commands,
    level_assets: Res<LevelAssets>,
    mut cursor: Single<&mut CursorOptions, With<PrimaryWindow>>,
) {
    cursor.visible = false;
    cursor.grab_mode = CursorGrabMode::Locked;
    commands.spawn((
        Name::new("Level"),
        Level,
        SceneRoot(level_assets.level.clone()),
        DespawnOnExit(Screen::Gameplay),
    ));
}

#[point_class]
pub struct Ball;

fn setup_sphere(
    event: On<Add, Ball>,
    mut commands: Commands,
    mut mesh_assets: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let entity = event.entity;
    tracing::info!(?entity, "Setting Up Spawned Ball");
    commands.entity(entity).insert((
        Mesh3d(mesh_assets.add(Sphere::new(1.0))),
        MeshMaterial3d(materials.add(StandardMaterial::default())),
    ));
}
