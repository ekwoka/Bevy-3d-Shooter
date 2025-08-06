use std::f32::consts::TAU;

use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use bevy_tnua::prelude::{TnuaBuiltinWalk, TnuaController};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, apply_movement);

    app.add_input_context::<DefaultInputContext>();

    app.add_observer(apply_default_binding);
    app.add_observer(remove_default_binding);
}

fn apply_movement(
    mut controller: Single<&mut TnuaController>,
    transform: Single<&Transform, With<Camera3d>>,
    move_action: Single<&Action<Move>, Changed<Action<Move>>>,
) {
    let yaw = transform.rotation.to_euler(EulerRot::YXZ).0;
    let yaw_quat = Quat::from_axis_angle(Vec3::Y, yaw);

    controller.basis(TnuaBuiltinWalk {
        desired_velocity: yaw_quat * ***move_action,
        float_height: 2.0,
        max_slope: TAU / 8.0,
        ..default()
    });
}

#[derive(Debug, InputAction)]
#[action_output(Vec3)]
pub(super) struct Move;

#[derive(Debug, InputAction)]
#[action_output(Vec2)]
pub(super) struct Look;

#[derive(Component)]
pub(super) struct DefaultInputContext;

impl DefaultInputContext {
    fn bindings() -> impl Bundle {
        actions!(
            DefaultInputContext[(
                Action::<Move>::new(),
                DeadZone::default(),
                SmoothNudge::default(),
                Bindings::spawn((
                    Cardinal::wasd_keys(),
                    Axial::left_stick()
                )),
                Negate::y(),
                SwizzleAxis::XZY,
                Scale::splat(12.0)
            ),
            (
                Action::<Look>::new(),
                Bindings::spawn(Spawn((Binding::mouse_motion(), Negate::all(), SwizzleAxis::YXZ)))
            )]
        )
    }
}

fn apply_default_binding(trigger: Trigger<OnAdd, DefaultInputContext>, mut commands: Commands) {
    commands
        .entity(trigger.target())
        .insert(DefaultInputContext::bindings());
}

fn remove_default_binding(
    trigger: Trigger<OnRemove, DefaultInputContext>,
    mut commands: Commands,
    mut actions: Query<&mut Actions<DefaultInputContext>>,
) {
    let owner = trigger.target();
    let actions = actions.get_mut(owner).unwrap();
    actions.into_iter().for_each(|entity| {
        info!(?entity, "Removing Entity");
        commands.entity(entity).try_despawn();
    });
}
