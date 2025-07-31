//! Player-specific behavior.

use std::f32::consts::TAU;

use bevy::{
    image::{ImageLoaderSettings, ImageSampler},
    prelude::*,
};

use bevy_enhanced_input::prelude::*;
use bevy_tnua::prelude::{TnuaBuiltinWalk, TnuaController};

use crate::{
    asset_tracking::LoadResource,
    demo::{
        animation::PlayerAnimation,
        movement::{MovementController, ScreenWrap},
    },
};

use avian3d::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Player>();
    app.add_input_context::<Player>();

    app.register_type::<PlayerAssets>();
    app.load_resource::<PlayerAssets>();

    println!("Adding player observer");

    app.add_observer(handled_player_input);
    app.add_observer(handled_player_looking);

    app.add_systems(Update, apply_movement);

    // Record directional input as movement controls.
}

/// The player character.
pub fn player() -> impl Bundle {
    println!("Spawning Player");
    (
        Name::new("Player"),
        Player::default(),
        Camera3d::default(),
        Transform::default(),
        RigidBody::Dynamic,
        Collider::sphere(0.5),
        TnuaController::default(),
        actions!(
            Player[(
                Action::<Move>::new(),
                DeadZone::default(),
                Bindings::spawn((
                    Cardinal::wasd_keys(),
                    Axial::left_stick()
                )),
                Negate::y(),
                SwizzleAxis::XZY,
            ),
            (
                Action::<Look>::new(),
                Bindings::spawn(Spawn((Binding::mouse_motion(), Negate::all(), SwizzleAxis::YXZ)))
            )]
        ),
    )
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Default, Reflect)]
#[reflect(Component)]
struct Player {
    movement_intent: Vec3,
    rotation_intent: Vec2,
}

fn handled_player_input(trigger: Trigger<Fired<Move>>, mut player: Single<&mut Player>) {
    println!("movement Intent: {}", trigger.value);
    player.movement_intent += trigger.value;
}

fn handled_player_looking(
    trigger: Trigger<Fired<Look>>,
    mut transform: Single<&mut Transform, With<Player>>,
    time: Res<Time>,
    window: Single<&Window, With<bevy::window::PrimaryWindow>>,
) {
    println!("looking: {}", trigger.value);
    if !window.focused {
        return;
    }
    let sensitivity = 100.0 / window.width().min(window.height());
    let delta = time.delta_secs() * sensitivity;
    let (mut yaw, mut pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);
    yaw += trigger.value.y * delta;
    pitch += trigger.value.x * delta;
    pitch = pitch.clamp(-1.57, 1.57);
    transform.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.0);
}

fn apply_rotation(
    query: Single<(&mut Transform, &mut Player)>,
    time: Res<Time>,
    window: Single<&Window, With<bevy::window::PrimaryWindow>>,
) {
}

fn apply_movement(query: Single<(&mut TnuaController, &Transform, &mut Player)>) {
    let (mut controller, transform, mut player) = query.into_inner();
    let yaw = transform.rotation.to_euler(EulerRot::YXZ).0;
    let yaw_quat = Quat::from_axis_angle(Vec3::Y, yaw);
    controller.basis(TnuaBuiltinWalk {
        // The `desired_velocity` determines how the character will move.
        desired_velocity: yaw_quat * player.movement_intent * 8.0,
        // The `float_height` must be greater (even if by little) from the distance between the
        // character's center and the lowest point of its collider.
        float_height: 2.0,
        // Restrict the max slope so that the player cannot walk up slightly angled chairs.
        max_slope: TAU / 8.0,
        ..default()
    });
    player.movement_intent = Vec3::ZERO;
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct PlayerAssets {
    #[dependency]
    ducky: Handle<Image>,
    #[dependency]
    pub steps: Vec<Handle<AudioSource>>,
}

impl FromWorld for PlayerAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            ducky: assets.load_with_settings(
                "images/ducky.png",
                |settings: &mut ImageLoaderSettings| {
                    // Use `nearest` image sampling to preserve pixel art style.
                    settings.sampler = ImageSampler::nearest();
                },
            ),
            steps: vec![],
        }
    }
}

#[derive(Debug, InputAction)]
#[action_output(Vec3)]
struct Move;

#[derive(Debug, InputAction)]
#[action_output(Vec2)]
struct Look;
