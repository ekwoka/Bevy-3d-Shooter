//! Player-specific behavior.

use bevy::prelude::*;

use bevy_enhanced_input::prelude::*;
use bevy_tnua::prelude::TnuaController;
use bevy_trenchbroom::prelude::*;

use avian_bullet_trajectory::BulletPhysicsConfig;
use avian3d::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Player>()
        .register_type::<WeaponSpawner>();

    app.add_observer(setup_player)
        .add_observer(handled_player_looking)
        .add_observer(setup_weapon_spawner);

    app.add_systems(Update, sync_player_camera);
}

/// The player character.
pub fn _player() -> impl Bundle {
    tracing::info!("Spawning Player");
    Player
}

fn setup_player(trigger: Trigger<OnAdd, Player>, mut commands: Commands) {
    tracing::info!("Setting Up Spawned Player");
    commands.entity(trigger.target()).insert((
        Name::new("PlayerRoot"),
        super::movement::DefaultInputContext,
        super::target::WeaponContext,
        RigidBody::Dynamic,
        Collider::capsule(0.29, 1.0),
        TnuaController::default(),
        LockedAxes::ROTATION_LOCKED,
        children![(
            Name::new("PlayerView"),
            PlayerView,
            Transform::from_xyz(0.0, 0.5, 0.0)
        )],
    ));
}

#[point_class(model({ path: "models/fnf2000.glb" }), base(Transform))]
#[derive(Default)]
#[reflect(Component)]
pub struct WeaponSpawner {
    pub weapon: WeaponType,
}

#[derive(Component, FgdType, Default, Reflect, Clone, Copy)]
#[reflect(Component)]
pub enum WeaponType {
    #[default]
    Glock,
    FNF2000,
}

impl WeaponType {
    pub fn name(&self) -> &str {
        match self {
            WeaponType::Glock => "Glock",
            WeaponType::FNF2000 => "FN F2000",
        }
    }

    pub fn model(&self) -> &str {
        match self {
            WeaponType::Glock => "models/glock.glb#Scene0",
            WeaponType::FNF2000 => "models/fnf2000.glb#Scene0",
        }
    }

    pub fn ballistics(&self) -> BulletPhysicsConfig {
        match self {
            WeaponType::Glock => BulletPhysicsConfig::caliber_9mm(),
            WeaponType::FNF2000 => BulletPhysicsConfig::caliber_556(),
        }
    }

    pub fn muzzle_velocity(&self) -> f32 {
        match self {
            WeaponType::Glock => 375.0,
            WeaponType::FNF2000 => 900.0,
        }
    }
}

fn setup_weapon_spawner(
    trigger: Trigger<OnAdd, WeaponSpawner>,
    mut commands: Commands,
    spawner: Query<&WeaponSpawner>,
    asset_server: Res<AssetServer>,
) {
    tracing::info!("Setting Up Spawned Weaponer Spawner");
    if let Ok(spawner) = spawner.get(trigger.target()) {
        commands.entity(trigger.target()).insert((
            Name::new("WeaponSpawner"),
            RigidBody::Dynamic,
            Collider::cuboid(0.08, 0.2, 0.6),
            SceneRoot(asset_server.load(spawner.weapon.model())),
        ));
    }
}

#[point_class]
#[derive(Component, Debug, Clone, Copy, PartialEq, Default, Reflect)]
#[reflect(Component)]
pub struct Player;

#[derive(Component, Debug, Clone, Copy, PartialEq, Default, Reflect)]
#[reflect(Component)]
pub struct PlayerView;

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
    camera.translation = player_root.translation + player_view.translation;
    camera.rotation = player_root.rotation * player_view.rotation;
}
