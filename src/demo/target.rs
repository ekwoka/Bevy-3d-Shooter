use crate::theme::widget;

use super::debug::DebugLines;
use avian_bullet_trajectory::BulletTrajectory;
use avian3d::prelude::*;
use bevy::{prelude::*, render::view::NoFrustumCulling};
use bevy_enhanced_input::prelude::*;
use bevy_trenchbroom::prelude::*;
use bevy_ui_anchor::{
    AnchorPoint, AnchorUiConfig, AnchorUiNode, AnchoredUiNodes, HorizontalAnchor, VerticalAnchor,
};

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
    app.add_systems(Update, update_target_distances);
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
        AnchoredUiNodes::spawn_one((
            AnchorUiConfig {
                anchorpoint: AnchorPoint {
                    vertical: VerticalAnchor::Bottom,
                    horizontal: HorizontalAnchor::Mid,
                },
                offset: Some(Vec3::new(0.0, 0.5, 0.0)),
                ..default()
            },
            widget::label("Target"),
            Visibility::Hidden,
        )),
    ));
}

fn handle_click(
    mouse: Res<ButtonInput<MouseButton>>,
    origin: Single<&Transform, With<Camera3d>>,
    spatial_query: SpatialQuery,
    targets: Query<&Target>,
    weapon: Single<&super::player::WeaponType>,
    mut commands: Commands,
    mut lines: ResMut<DebugLines>,
) {
    if mouse.just_pressed(MouseButton::Left) {
        // Bullet starts slightly in front of camera to avoid self-collision
        let start = origin.translation + origin.forward() * 2.0;
        let direction = origin.forward();

        let initial_velocity = direction * weapon.muzzle_velocity();
        let bullet_mass = 0.0075; // 7.5 grams for 9mm

        // Use realistic physics config for 9mm
        let config = weapon.ballistics();
        let filter = SpatialQueryFilter::default();

        // Simulate the bullet trajectory
        let trajectory = spatial_query.simulate_bullet_trajectory(
            start,
            initial_velocity,
            bullet_mass,
            Some(config),
            &filter,
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
    weapons: Query<&super::player::WeaponSpawner>,
    existing_weapons: Query<Entity, With<super::player::WeaponType>>,
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
        .filter_map(|entity| weapons.get(*entity).map(|weapon| weapon.weapon).ok())
        .next()
    {
        info!(weapon = weapon.name(), "Weapon Found");
        lines.push(move |gizmos| {
            gizmos.sphere(location, 2.0, Color::linear_rgb(0.0, 1.0, 0.0));
        });
        existing_weapons.iter().for_each(|entity| {
            commands.entity(entity).try_despawn();
        });
        commands.entity(*player_view).insert(children![(
            Name::new(weapon.name().to_string()),
            SceneRoot(asset_server.load(weapon.model())),
            Transform::from_xyz(0.08, -0.12, -0.3),
            NoFrustumCulling,
            weapon
        )]);
    } else {
        lines.push(move |gizmos| {
            gizmos.sphere(location, 2.0, Color::WHITE);
        });
    }
}

fn update_target_distances(
    origin: Single<&Transform, With<Camera3d>>,
    spatial_query: SpatialQuery,
    targets: Query<&AnchoredUiNodes, With<Target>>,
    mut ui_nodes: Query<(&mut Text, &mut Visibility), With<AnchorUiNode>>,
) {
    let start = origin.translation + origin.forward() * 2.0;
    let direction = origin.forward();
    let mut target: Option<ShapeHitData> = None;
    spatial_query.shape_hits_callback(
        &Collider::sphere(2.0),
        start,
        Quat::default(),
        direction,
        &ShapeCastConfig::from_max_distance(500.0),
        &SpatialQueryFilter::default(),
        |hit| {
            if targets.contains(hit.entity) {
                target.replace(hit);
                false
            } else {
                true
            }
        },
    );
    for (_, mut visibility) in ui_nodes.iter_mut() {
        *visibility = Visibility::Hidden;
    }

    if let Some((anchored_nodes, distance)) = target.and_then(|hit| {
        targets
            .get(hit.entity)
            .ok()
            .map(|anchored_nodes| (anchored_nodes, hit.distance))
    }) {
        for ui_node in anchored_nodes.iter() {
            if let Ok((mut text, mut visibility)) = ui_nodes.get_mut(ui_node) {
                **text = format!("{distance:.1}m");
                *visibility = Visibility::Visible;
            }
        }
    }
}
