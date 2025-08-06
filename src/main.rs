// Support configuring Bevy lints within code.
#![cfg_attr(bevy_lint, feature(register_tool), register_tool(bevy))]
// Disable console on Windows for non-dev builds.
#![cfg_attr(not(feature = "dev"), windows_subsystem = "windows")]

mod asset_tracking;
mod audio;
mod demo;
#[cfg(feature = "dev")]
mod dev_tools;
mod menus;
mod screens;
mod theme;

use avian3d::prelude::*;
use bevy::{
    asset::AssetMetaCheck,
    image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor},
    prelude::*,
};
use bevy_enhanced_input::prelude::EnhancedInputPlugin;
use bevy_tnua::prelude::TnuaControllerPlugin;
use bevy_tnua_avian3d::TnuaAvian3dPlugin;
use bevy_trenchbroom::prelude::*;

fn main() -> AppExit {
    App::new().add_plugins(AppPlugin).run()
}

pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        // Add Bevy plugins.
        app.add_plugins((
            DefaultPlugins
                .set(AssetPlugin {
                    // Wasm builds will check for meta files (that don't exist) if this isn't set.
                    // This causes errors and even panics on web build on itch.
                    // See https://github.com/bevyengine/bevy_github_ci_template/issues/48.
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Window {
                        title: "Bevy 3d".to_string(),
                        fit_canvas_to_parent: true,
                        ..default()
                    }
                    .into(),
                    ..default()
                })
                .set(ImagePlugin {
                    default_sampler: default_image_sampler_descriptor(),
                }),
            EnhancedInputPlugin,
            PhysicsPlugins::default(),
            TnuaAvian3dPlugin::new(PhysicsSchedule),
            TnuaControllerPlugin::new(PhysicsSchedule),
            TrenchBroomPlugins(
                TrenchBroomConfig::new("bevy_shooter")
                    .default_solid_spawn_hooks(|| {
                        SpawnHooks::new()
                            .convex_collider()
                            .smooth_by_default_angle()
                    })
                    .texture_sampler(texture_sampler()),
            ),
        ));

        app.add_systems(Startup, write_trenchbroom_config);

        // Add other plugins.
        app.add_plugins((
            asset_tracking::plugin,
            audio::plugin,
            demo::plugin,
            #[cfg(feature = "dev")]
            dev_tools::plugin,
            menus::plugin,
            screens::plugin,
            theme::plugin,
        ));

        // Order new `AppSystems` variants by adding them here:
        app.configure_sets(
            Update,
            (
                AppSystems::TickTimers,
                AppSystems::RecordInput,
                AppSystems::Update,
            )
                .chain(),
        );

        // Set up the `Pause` state.
        app.init_state::<Pause>();
        app.configure_sets(Update, PausableSystems.run_if(in_state(Pause(false))));

        // Spawn the main camera.
        app.add_systems(Startup, spawn_camera);
    }
}

/// High-level groupings of systems for the app in the `Update` schedule.
/// When adding a new variant, make sure to order it in the `configure_sets`
/// call above.
#[derive(SystemSet, Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
enum AppSystems {
    /// Tick timers.
    TickTimers,
    /// Record player input.
    RecordInput,
    /// Do everything else (consider splitting this into further variants).
    Update,
}

/// Whether or not the game is paused.
#[derive(States, Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
#[states(scoped_entities)]
struct Pause(pub bool);

/// A system set for systems that shouldn't run while the game is paused.
#[derive(SystemSet, Copy, Clone, Eq, PartialEq, Hash, Debug)]
struct PausableSystems;

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Name::new("Camera"),
        Camera3d::default(),
        Transform::default(),
    ));
}

fn write_trenchbroom_config(server: Res<TrenchBroomServer>, type_registry: Res<AppTypeRegistry>) {
    info!("Writing TrenchBroom config");
    // Errors at this point usually mean that the player has not installed TrenchBroom.
    // The error messages give more details about the exact issue.
    if let Err(err) = server
        .config
        .write_game_config_to_default_directory(&type_registry.read())
    {
        warn!("Could not write TrenchBroom game config: {err}");
    }
    if let Err(err) = server.config.add_game_to_preferences_in_default_directory() {
        warn!("Could not add game to TrenchBroom preferences: {err}");
    }
}

fn texture_sampler() -> ImageSampler {
    let mut sampler = ImageSampler::linear();
    *sampler.get_or_init_descriptor() = default_image_sampler_descriptor();
    sampler
}

pub(crate) fn default_image_sampler_descriptor() -> ImageSamplerDescriptor {
    ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        address_mode_w: ImageAddressMode::Repeat,
        anisotropy_clamp: 16,
        ..ImageSamplerDescriptor::linear()
    }
}
