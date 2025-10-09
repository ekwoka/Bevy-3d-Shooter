// Support configuring Bevy lints within code.
#![cfg_attr(bevy_lint, feature(register_tool), register_tool(bevy))]
// Disable console on Windows for non-dev builds.
#![cfg_attr(not(feature = "dev"), windows_subsystem = "windows")]

#[cfg(feature = "dev")]
use shooter::dev_tools;
use shooter::{asset_tracking, audio, demo, menus, screens, theme};

use avian3d::prelude::*;
use bevy::gltf::GltfPlugin;
use bevy::light::DirectionalLightShadowMap;
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
                })
                .set(GltfPlugin {
                    use_model_forward_direction: true,
                    ..default()
                }),
            bevy_ui_anchor::AnchorUiPlugin::<UICamera>::new(),
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
                    .scale(64.0)
                    .texture_sampler(texture_sampler())
                    .entity_scale_expression("{{ scale == undefined -> 12000, scale }}"),
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
            #[cfg(feature = "editor")]
            egui_editor::plugin,
        ));
        app.insert_resource(DirectionalLightShadowMap { size: 4096 });
        // Spawn the main camera.
        app.add_systems(Startup, spawn_camera);
    }
}

#[derive(Component)]
pub struct UICamera;

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Name::new("Camera"),
        Camera3d::default(),
        Transform::default(),
        UICamera,
        #[cfg(feature = "editor")]
        egui_editor::MainView,
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
