//! Player-specific behavior.

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
pub fn player() -> impl Bundle {
    tracing::info!("Spawning Player");
    Player
}

fn setup_player(trigger: Trigger<OnAdd, Player>, mut commands: Commands) {
    tracing::info!("Setting Up Spawned Player");
    commands.entity(trigger.target()).insert((
        Name::new("Player"),
        super::movement::DefaultInputContext,
        RigidBody::Dynamic,
        Collider::sphere(0.5),
        TnuaController::default(),
    ));
}

#[point_class]
#[derive(Component, Debug, Clone, Copy, PartialEq, Default, Reflect)]
#[reflect(Component)]
struct Player;

fn handled_player_looking(
    trigger: Trigger<Fired<super::movement::Look>>,
    mut camera: Single<&mut Transform, With<Camera3d>>,
    time: Res<Time>,
    window: Single<&Window, With<bevy::window::PrimaryWindow>>,
) {
    if !window.focused {
        return;
    }
    let sensitivity = 100.0 / window.width().min(window.height());
    let delta = time.delta_secs() * sensitivity;
    let (mut yaw, mut pitch, _) = camera.rotation.to_euler(EulerRot::YXZ);
    tracing::debug!(yaw = yaw, pitch = pitch, "Player is Looking Around");
    yaw += trigger.value.y * delta;
    pitch += trigger.value.x * delta;
    pitch = pitch.clamp(-1.57, 1.57);
    camera.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.0);
}

fn sync_player_camera(
    mut camera: Single<&mut Transform, With<Camera3d>>,
    player: Single<&Transform, (With<Player>, Without<Camera3d>)>,
) {
    camera.translation = player.translation;
}
