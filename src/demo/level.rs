//! Spawn the main level.

use avian3d::prelude::{Collider, RigidBody};
use bevy::prelude::*;

use crate::{asset_tracking::LoadResource, screens::Screen};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<LevelAssets>();
    app.load_resource::<LevelAssets>();
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct LevelAssets;

impl FromWorld for LevelAssets {
    fn from_world(_world: &mut World) -> Self {
        Self {}
    }
}

/// A system that spawns the main level.
pub fn spawn_level(
    mut commands: Commands,
    mut mesh_assets: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(DirectionalLight::default());
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(200.0, 5.0, 200.0),
        Transform::from_xyz(0.0, -10.0, 0.0),
    ));
    let ball_mesh = mesh_assets.add(Sphere::new(1.0));
    for h in 1..16 {
        let color = Color::hsl(h as f32 / 16.0 * 360.0, 1.0, 0.5);
        let ball_material = materials.add(StandardMaterial {
            base_color: color,
            ..Default::default()
        });
        commands.spawn((
            Transform::from_translation(Vec3::new((-8.0 + h as f32) * 2.0, 0.0, -50.0)),
            Mesh3d(ball_mesh.clone()),
            MeshMaterial3d(ball_material),
            StateScoped(Screen::Gameplay),
        ));
    }
    commands.spawn(super::player::player());
}
