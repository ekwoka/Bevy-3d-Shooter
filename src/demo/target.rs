use super::debug::DebugLines;
use avian_bullet_trajectory::{BulletPhysicsConfig, BulletTrajectory};
use avian3d::prelude::*;
use bevy::{prelude::*, render::view::NoFrustumCulling};
use bevy_enhanced_input::prelude::*;
use bevy_trenchbroom::prelude::*;

#[point_class]
#[derive(Debug, Clone, Copy)]
#[reflect(Component)]
struct Target;

#[point_class(model({ path: "models/target.gltf", scale: 75 }), base(Transform))]
#[reflect(Component)]
struct TargetSpawner;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Target>();
    app.register_type::<TargetSpawner>();
    app.add_observer(setup_target);
    app.add_systems(Update, (handle_click, spawn_target));
    app.add_input_context::<WeaponContext>();
    app.add_observer(apply_weapon_binding);
    app.add_observer(remove_weapon_binding);
    app.add_observer(pickup_weapon);
}

fn setup_target(
    trigger: Trigger<OnAdd, Target>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let entity = trigger.target();
    tracing::info!(?entity, "Setting Up Spawned Target");
    commands.entity(entity).insert((
        RigidBody::Static,
        Collider::cuboid(1.0, 1.0, 0.6),
        SceneRoot(asset_server.load("models/target.gltf#Scene0")),
    ));
}

fn handle_click(
    mouse: Res<ButtonInput<MouseButton>>,
    origin: Single<&Transform, With<Camera3d>>,
    spatial_query: SpatialQuery,
    targets: Query<&Target>,
    mut commands: Commands,
    mut lines: ResMut<DebugLines>,
) {
    if mouse.just_pressed(MouseButton::Left) {
        info!("Left mouse button clicked - simulating bullet trajectory");

        // Bullet starts slightly in front of camera to avoid self-collision
        let start = origin.translation + origin.forward() * 2.0;
        let direction = origin.forward();

        // Bullet parameters (9mm example)
        let muzzle_velocity = 900.0; // m/s for 9mm
        let initial_velocity = direction * muzzle_velocity;
        let bullet_mass = 0.0075; // 7.5 grams for 9mm

        // Use realistic physics config for 9mm
        let config = BulletPhysicsConfig::caliber_9mm();
        let filter = SpatialQueryFilter::default();

        info!(
            "Simulating bullet from {:?} with velocity {:?} m/s",
            start,
            initial_velocity.length()
        );

        // Simulate the bullet trajectory
        let trajectory = spatial_query.simulate_bullet_trajectory(
            start,
            initial_velocity,
            bullet_mass,
            Some(config),
            &filter,
        );

        info!(
            "Trajectory complete: distance={:.2}m, time={:.3}s, hit={:?}",
            trajectory.distance,
            trajectory.time_of_flight,
            trajectory.hit_entity.is_some()
        );

        // Check if we hit a target
        if let Some(hit_entity) = trajectory.hit_entity {
            if let Ok(target) = targets.get(hit_entity) {
                info!("Hit target: {:?} at {:?}", target, trajectory.hit_point);
                info!(
                    "Impact velocity: {:.1} m/s",
                    trajectory.impact_velocity.length()
                );

                // Draw successful hit trajectory in green
                let points = trajectory.trajectory_points.clone();
                lines.push(move |gizmos: &mut Gizmos| {
                    for window in points.windows(2) {
                        gizmos.line(window[0], window[1], Color::linear_rgb(0.0, 1.0, 0.0));
                    }
                    // Draw impact point
                    gizmos.sphere(trajectory.hit_point, 0.2, Color::linear_rgb(1.0, 1.0, 0.0));
                });

                commands.entity(hit_entity).despawn();
            } else {
                // Hit something else - draw in red
                let points = trajectory.trajectory_points.clone();
                let hit_point = trajectory.hit_point;
                lines.push(move |gizmos: &mut Gizmos| {
                    for window in points.windows(2) {
                        gizmos.line(window[0], window[1], Color::linear_rgb(1.0, 0.0, 0.0));
                    }
                    gizmos.sphere(hit_point, 0.15, Color::linear_rgb(1.0, 0.5, 0.0));
                });
            }
        } else {
            // No hit - draw trajectory in white
            let points = trajectory.trajectory_points.clone();
            lines.push(move |gizmos: &mut Gizmos| {
                for window in points.windows(2) {
                    gizmos.line(window[0], window[1], Color::WHITE);
                }
            });
        }
    }

    // Right-click for simple trajectory without air resistance
    if mouse.just_pressed(MouseButton::Right) {
        info!("Right mouse button clicked - simple ballistic trajectory");

        let start = origin.translation + origin.forward() * 2.0;
        let direction = origin.forward();
        let initial_velocity = direction * 900.0; // Faster for demo
        let filter = SpatialQueryFilter::default();

        let trajectory = spatial_query.simulate_simple_trajectory(
            start,
            initial_velocity,
            None, // Use default gravity
            &filter,
        );

        info!(
            "Simple trajectory: distance={:.2}m, time={:.3}s",
            trajectory.distance, trajectory.time_of_flight
        );

        // Draw simple trajectory in cyan
        let points = trajectory.trajectory_points.clone();
        let hit_point = trajectory.hit_point;
        lines.push(move |gizmos: &mut Gizmos| {
            for window in points.windows(2) {
                gizmos.line(window[0], window[1], Color::linear_rgb(0.0, 1.0, 1.0));
            }
            if trajectory.hit_entity.is_some() {
                gizmos.sphere(hit_point, 0.2, Color::linear_rgb(1.0, 0.0, 1.0));
            }
        });

        if let Some(hit_entity) = trajectory.hit_entity
            && targets.get(hit_entity).is_ok()
        {
            commands.entity(hit_entity).despawn();
        }
    }
}

fn spawn_target(
    mut commands: Commands,
    targets: Query<&Target>,
    spawners: Query<&Transform, With<TargetSpawner>>,
) {
    if targets.is_empty() {
        for spawner in spawners.iter() {
            commands.spawn((Target, *spawner));
        }
    }
}

#[derive(Debug, InputAction)]
#[action_output(bool)]
pub(super) struct Pickup;

#[derive(Component)]
pub(super) struct WeaponContext;

impl WeaponContext {
    fn bindings() -> impl Bundle {
        actions!(
            WeaponContext[(
                Action::<Pickup>::new(),
                Press::new(1.0),
                bindings![KeyCode::KeyF]
            )]
        )
    }
}

fn apply_weapon_binding(trigger: Trigger<OnAdd, WeaponContext>, mut commands: Commands) {
    info!("Applying weapon binding");
    commands
        .entity(trigger.target())
        .insert(WeaponContext::bindings());
}

fn remove_weapon_binding(
    trigger: Trigger<OnRemove, WeaponContext>,
    mut commands: Commands,
    mut actions: Query<&mut Actions<WeaponContext>>,
) {
    let owner = trigger.target();
    let actions = actions.get_mut(owner).unwrap();
    actions.into_iter().for_each(|entity| {
        info!(?entity, "Removing Entity");
        commands.entity(entity).try_despawn();
    });
}

fn pickup_weapon(
    _trigger: Trigger<Fired<Pickup>>,
    mut commands: Commands,
    mut lines: ResMut<DebugLines>,
    spatial_query: SpatialQuery,
    asset_server: Res<AssetServer>,
    weapons: Query<Entity, With<super::player::WeaponSpawner>>,
    player: Single<&Transform, With<super::player::Player>>,
    player_view: Single<Entity, With<super::player::PlayerView>>,
) {
    info!("Picking up weapon");
    let location = player.translation;
    if let Some(weapon) = spatial_query
        .shape_intersections(
            &Collider::sphere(2.0),
            location,
            Quat::default(),
            &SpatialQueryFilter::default(),
        )
        .iter()
        .filter_map(|entity| weapons.get(*entity).ok()).next()
    {
        info!(?weapon, "Weapon Found");
        lines.push(move |gizmos| {
            gizmos.sphere(location, 2.0, Color::linear_rgb(0.0, 1.0, 0.0));
        });
        commands.entity(*player_view).insert(children![(
            Name::new("FNF2000"),
            SceneRoot(asset_server.load("models/fnf2000.glb#Scene0")),
            Transform::from_xyz(0.08, -0.12, -0.3),
            NoFrustumCulling
        )]);
    } else {
        lines.push(move |gizmos| {
            gizmos.sphere(location, 2.0, Color::WHITE);
        });
    }
}
