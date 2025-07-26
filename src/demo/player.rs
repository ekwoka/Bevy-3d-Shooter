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

    app.add_observer(handled_player_input);

    // Record directional input as movement controls.
}

/// The player character.
pub fn player(
    max_speed: f32,
    player_assets: &PlayerAssets,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
) -> impl Bundle {
    // A texture atlas is a way to split a single image into a grid of related images.
    // You can learn more in this example: https://github.com/bevyengine/bevy/blob/latest/examples/2d/texture_atlas.rs
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(32), 6, 2, Some(UVec2::splat(1)), None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);
    let player_animation = PlayerAnimation::new();

    (
        Name::new("Player"),
        Player,
        Sprite {
            image: player_assets.ducky.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: texture_atlas_layout,
                index: player_animation.get_atlas_index(),
            }),
            ..default()
        },
        Transform::from_scale(Vec2::splat(8.0).extend(1.0)),
        MovementController {
            max_speed,
            ..default()
        },
        ScreenWrap,
        player_animation,
        actions!(
            Player[(
                Action::<Move>::new(),
                DeadZone::default(),
                SmoothNudge::default(),
                Bindings::spawn((Cardinal::wasd_keys(), Axial::left_stick(),))
            )]
        ),
    )
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
struct Player;

fn handled_player_input(
    trigger: Trigger<Fired<Move>>,
    mut players: Query<&mut MovementController, With<Player>>,
) {
    let mut controller = players.get_mut(trigger.target()).unwrap();

    controller.intent += trigger.value;
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
