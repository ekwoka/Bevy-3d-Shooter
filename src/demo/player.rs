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

    // Record directional input as movement controls.
}

/// The player character.
pub fn player() -> impl Bundle {
    println!("Spawning Player");
    (
        Name::new("Player"),
        Player,
        Camera3d::default(),
        Transform::default(),
        actions!(
            Player[(
                Action::<Move>::new(),
                DeadZone::default(),
                SmoothNudge::default(),
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

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
struct Player;

fn handled_player_input(trigger: Trigger<Fired<Move>>, mut _controller: Single<&Player>) {
    println!("Intent: {}", trigger.value);
}

fn handled_player_looking(
    trigger: Trigger<Fired<Look>>,
    mut player: Single<&mut Transform, With<Player>>,
) {
    println!("looking: {}", trigger.value);
    player.rotation.y += trigger.value.y / 1000.0;
    player.rotation.x += trigger.value.x / 1000.0;
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
