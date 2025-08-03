//! Spawn the main level.

use avian3d::prelude::{Collider, RigidBody};
use bevy::prelude::*;
use bevy_trenchbroom::prelude::*;

use crate::{asset_tracking::LoadResource, screens::Screen};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Level>();
    app.register_type::<Ball>();
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
    // mut mesh_assets: ResMut<Assets<Mesh>>,
    // mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Name::new("Level"),
        Level,
        SceneRoot(level_assets.level.clone()),
        StateScoped(Screen::Gameplay),
    ));
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(200.0, 5.0, 200.0),
        Transform::from_xyz(0.0, -10.0, 0.0),
    ));
    // let ball_mesh = mesh_assets.add(Sphere::new(1.0));
    // for h in 1..16 {
    //     let color = Color::hsl(h as f32 / 16.0 * 360.0, 1.0, 0.5);
    //     let ball_material = materials.add(StandardMaterial {
    //         base_color: color,
    //         ..Default::default()
    //     });
    //     commands.spawn((
    //         Transform::from_translation(Vec3::new((-8.0 + h as f32) * 2.0, 0.0, -50.0)),
    //         Mesh3d(ball_mesh.clone()),
    //         MeshMaterial3d(ball_material),
    //         StateScoped(Screen::Gameplay),
    //     ));
    // }
    // commands.spawn(super::player::player());
}

#[point_class]
pub struct Ball;

fn setup_sphere(
    trigger: Trigger<OnAdd, Ball>,
    mut commands: Commands,
    mut mesh_assets: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    tracing::info!("Setting Up Spawned Ball");
    commands.entity(trigger.target()).insert((
        Mesh3d(mesh_assets.add(Sphere::new(1.0))),
        MeshMaterial3d(materials.add(StandardMaterial::default())),
        StateScoped(Screen::Gameplay),
    ));
}
