//! Player-specific behavior.

use std::f32::consts::PI;

use bevy::prelude::*;

use bevy_enhanced_input::prelude::*;
use bevy_tnua::prelude::TnuaController;
use bevy_trenchbroom::prelude::*;

use avian3d::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Player>();

    app.add_observer(setup_player);
    app.add_observer(handled_player_looking);

    app.add_systems(Update, sync_player_camera);
}

/// The player character.
pub fn _player() -> impl Bundle {
    tracing::info!("Spawning Player");
    Player
}

fn setup_player(
    trigger: Trigger<OnAdd, Player>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    tracing::info!("Setting Up Spawned Player");
    commands.entity(trigger.target()).insert((
        Name::new("PlayerRoot"),
        super::movement::DefaultInputContext,
        RigidBody::Dynamic,
        Collider::sphere(0.5),
        TnuaController::default(),
        LockedAxes::ROTATION_LOCKED,
        children![(
            Name::new("PlayerView"),
            PlayerView,
            Transform::from_xyz(0.0, 0.0, 0.0),
            children![(
                Name::new("FNF2000"),
                SceneRoot(asset_server.load("models/fnf2000.glb#Scene0")),
                Transform::from_xyz(0.15, -0.35, -1.0)
            )]
        )],
    ));
}

#[point_class]
#[derive(Component, Debug, Clone, Copy, PartialEq, Default, Reflect)]
#[reflect(Component)]
struct Player;

#[derive(Component, Debug, Clone, Copy, PartialEq, Default, Reflect)]
#[reflect(Component)]
struct PlayerView;

fn handled_player_looking(
    trigger: Trigger<Fired<super::movement::Look>>,
    mut player_view: Single<&mut Transform, With<PlayerView>>,
    time: Res<Time>,
    window: Single<&Window, With<bevy::window::PrimaryWindow>>,
) {
    if !window.focused {
        return;
    }
    let sensitivity = 100.0 / window.width().min(window.height());
    let delta = time.delta_secs() * sensitivity;
    let (mut yaw, mut pitch, _) = player_view.rotation.to_euler(EulerRot::YXZ);
    tracing::debug!(yaw = yaw, pitch = pitch, "Player is Looking Around");
    yaw += trigger.value.y * delta;
    pitch += trigger.value.x * delta;
    pitch = pitch.clamp(-1.57, 1.57);
    player_view.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.0);
}

fn sync_player_camera(
    mut camera: Single<&mut Transform, With<Camera3d>>,
    player_root: Single<&Transform, (With<Player>, Without<Camera3d>)>,

    player_view: Single<&Transform, (With<PlayerView>, Without<Camera3d>, Without<Player>)>,
) {
    camera.translation = player_root.translation;
    camera.rotation = player_view.rotation;
    camera.rotate_y(PI);
}
