//! Player-specific behavior.

use bevy::{
    image::{ImageLoaderSettings, ImageSampler},
    prelude::*,
};

use bevy_enhanced_input::prelude::*;

use crate::{
    asset_tracking::LoadResource,
    demo::{
        animation::PlayerAnimation,
        movement::{MovementController, ScreenWrap},
    },
};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Player>();
    app.add_input_context::<Player>();

    app.register_type::<PlayerAssets>();
    app.load_resource::<PlayerAssets>();

    println!("Adding player observer");

    app.add_observer(handled_player_input);
    app.add_observer(handled_player_looking);

    app.add_systems(
        Update,
        (apply_rotation, apply_movement.after(apply_rotation)),
    );

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
        actions!(
            Player[(
                Action::<Move>::new(),
                DeadZone::default(),
                Bindings::spawn((
                    Cardinal::wasd_keys(),
                    Axial::left_stick()
                ))
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
    movement_intent: Vec2,
    rotation_intent: Vec2,
}

fn handled_player_input(trigger: Trigger<Fired<Move>>, mut player: Single<&mut Player>) {
    println!("movement Intent: {}", trigger.value);
    player.movement_intent += trigger.value;
}

fn handled_player_looking(trigger: Trigger<Fired<Look>>, mut player: Single<&mut Player>) {
    println!("looking: {}", trigger.value);
    player.rotation_intent += trigger.value;
}

fn apply_rotation(
    query: Single<(&mut Transform, &mut Player)>,
    time: Res<Time>,
    window: Single<&Window, With<bevy::window::PrimaryWindow>>,
) {
    if !window.focused {
        return;
    }
    let sensitivity = 100.0 / window.width().min(window.height());
    let delta = time.delta_secs() * sensitivity;
    let (mut transform, mut player) = query.into_inner();
    let (mut yaw, mut pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);
    yaw += player.rotation_intent.y * delta;
    pitch += player.rotation_intent.x * delta;
    pitch = pitch.clamp(-1.57, 1.57);
    transform.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.0);
    player.rotation_intent = Vec2::ZERO;
}

fn apply_movement(query: Single<(&mut Transform, &mut Player)>, time: Res<Time>) {
    let (mut transform, mut player) = query.into_inner();
    let delta = time.delta_secs();
    let intent = player.movement_intent;
    println!("Movement intent: {:?}", intent);
    let forward = transform.forward() * intent.y;
    let right = transform.right() * intent.x;
    let to_move = (forward + right).normalize_or_zero().with_y(0.0);
    transform.translation += to_move * delta * 50.0;
    player.movement_intent = Vec2::ZERO;
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
#[action_output(Vec2)]
struct Move;

#[derive(Debug, InputAction)]
#[action_output(Vec2)]
struct Look;
