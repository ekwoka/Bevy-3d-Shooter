use super::debug::DebugLines;
use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_trenchbroom::prelude::*;

#[point_class]
#[derive(Component, Debug, Clone, Copy, PartialEq, Default, Reflect)]
#[reflect(Component)]
struct Target;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Target>();
    app.add_observer(setup_target);
    app.add_systems(Update, handle_click);
}

fn setup_target(
    trigger: Trigger<OnAdd, Target>,
    mut commands: Commands,
    mut mesh_assets: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let entity = trigger.target();
    tracing::info!(?entity, "Setting Up Spawned Target");
    commands.entity(entity).insert((
        Mesh3d(mesh_assets.add(Sphere::new(1.0))),
        MeshMaterial3d(materials.add(StandardMaterial::default())),
        Collider::sphere(1.0),
        RigidBody::Static,
    ));
}

fn handle_click(
    mouse: Res<ButtonInput<MouseButton>>,
    origin: Single<&Transform, With<Camera3d>>,
    spatial_query: SpatialQuery,
    mut lines: ResMut<DebugLines>,
) {
    if mouse.just_pressed(MouseButton::Left) {
        info!("Left mouse button clicked");
        let start = origin.translation;
        let direction = origin.forward();
        let max_distance = 100.0;
        let solid = true;
        let filter = SpatialQueryFilter::default();

        info!("Starting raycast from {:?}", start);

        if let Some(hit) = spatial_query.cast_ray(
            start + direction * 2.0,
            direction,
            max_distance,
            solid,
            &filter,
        ) {
            info!("Hit: {:?}", hit);
            info!("hit point: {:?}", start + direction * (hit.distance * 2.0));
            lines.push(Box::new(move |gizmos: &mut Gizmos| {
                gizmos.ray(
                    start,
                    Vec3::from(direction) * (hit.distance + 2.0),
                    Color::linear_rgb(1.0, 0.0, 0.0),
                )
            }));
        } else {
            lines.push(Box::new(move |gizmos: &mut Gizmos| {
                gizmos.ray(start, Vec3::from(direction) * max_distance, Color::WHITE)
            }));
        }
    }
}
