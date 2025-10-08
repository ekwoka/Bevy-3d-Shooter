use std::f32::consts::TAU;

use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use bevy_tnua::prelude::{TnuaBuiltinJump, TnuaBuiltinWalk, TnuaController};

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
    jump_action: Single<&Action<Jump>, Changed<Action<Jump>>>,
    sprint_action: Single<&Action<Sprint>, Changed<Action<Sprint>>>,
) {
    let yaw = transform.rotation.to_euler(EulerRot::YXZ).0;
    let yaw_quat = Quat::from_axis_angle(Vec3::Y, yaw);
    if ***jump_action {
        info!("Jumping: {:?}", ***jump_action);
    }

    if ***sprint_action {
        info!("Sprinting: {:?}", ***sprint_action);
    }

    controller.basis(TnuaBuiltinWalk {
        desired_velocity: yaw_quat * ***move_action * if ***sprint_action { 24.0 } else { 12.0 },
        float_height: 1.4,
        max_slope: TAU / 5.0,
        spring_strength: 2000.0,
        ..default()
    });

    if ***jump_action {
        controller.action(TnuaBuiltinJump {
            height: 4.0,
            ..default()
        });
    }
}

#[derive(Debug, InputAction)]
#[action_output(Vec3)]
pub(super) struct Move;

#[derive(Debug, InputAction)]
#[action_output(Vec2)]
pub(super) struct Look;

#[derive(Debug, InputAction)]
#[action_output(bool)]
pub(super) struct Jump;

#[derive(Debug, InputAction)]
#[action_output(bool)]
pub(super) struct Sprint;

#[derive(Component)]
pub(super) struct DefaultInputContext;

impl DefaultInputContext {
    fn bindings() -> impl Bundle {
        actions!(
            DefaultInputContext[
                (
                    Action::<Move>::new(),
                    DeadZone::default(),
                    SmoothNudge::default(),
                    Bindings::spawn((
                        Cardinal::wasd_keys(),
                        Axial::left_stick()
                    )),
                    Negate::y(),
                    SwizzleAxis::XZY
                ),
                (
                    Action::<Look>::new(),
                    Bindings::spawn(Spawn((Binding::mouse_motion(), Negate::all(), SwizzleAxis::YXZ)))
                ),
                (
                    Action::<Jump>::new(),
                    bindings![KeyCode::Space]
                ),
                (
                    Action::<Sprint>::new(),
                    bindings![KeyCode::ShiftLeft]
                )
            ]
        )
    }
}

fn apply_default_binding(trigger: On<Add, DefaultInputContext>, mut commands: Commands) {
    commands
        .entity(trigger.entity)
        .insert(DefaultInputContext::bindings());
}

fn remove_default_binding(
    trigger: On<Remove, DefaultInputContext>,
    mut commands: Commands,
    mut actions: Query<&mut Actions<DefaultInputContext>>,
) {
    let owner = trigger.entity;
    let actions = actions.get_mut(owner).unwrap();
    actions.into_iter().for_each(|entity| {
        info!(?entity, "Removing Entity");
        commands.entity(entity).try_despawn();
    });
}
