use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_trenchbroom::prelude::*;

#[point_class]
#[derive(Component, Debug, Clone, Copy, PartialEq, Default, Reflect)]
#[reflect(Component)]
struct Target;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Target>();
    app.add_observer(setup_target);
    app.add_systems(Update, (handle_click, draw_debug_lines));
    app.insert_resource(DebugLines::new());
}

fn setup_target(
    trigger: Trigger<OnAdd, Target>,
    mut commands: Commands,
    mut mesh_assets: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let entity = trigger.target();
    tracing::info!(?entity, "Setting Up Spawned Target");
    commands.entity(entity).insert((
        Mesh3d(mesh_assets.add(Sphere::new(1.0))),
        MeshMaterial3d(materials.add(StandardMaterial::default())),
        Collider::sphere(1.0),
        RigidBody::Static,
    ));
}

fn handle_click(
    mouse: Res<ButtonInput<MouseButton>>,
    origin: Single<&Transform, With<Camera3d>>,
    spatial_query: SpatialQuery,
    mut lines: ResMut<DebugLines>,
) {
    if mouse.just_pressed(MouseButton::Left) {
        info!("Left mouse button clicked");
        let start = origin.translation;
        let direction = origin.forward();
        let max_distance = 100.0;
        let solid = true;
        let filter = SpatialQueryFilter::default();

        info!("Starting raycast from {:?}", start);

        if let Some(hit) = spatial_query.cast_ray(
            start + direction * 2.0,
            direction,
            max_distance,
            solid,
            &filter,
        ) {
            info!("Hit: {:?}", hit);
            info!("hit point: {:?}", start + direction * (hit.distance * 2.0));
            lines.push(
                start,
                Vec3::from(direction) * (hit.distance + 2.0),
                Color::linear_rgb(1.0, 0.0, 0.0),
            );
        } else {
            lines.push(start, Vec3::from(direction) * max_distance, Color::WHITE);
        }
    }
}

fn draw_debug_lines(time: Res<Time>, mut lines: ResMut<DebugLines>, mut gizmos: Gizmos) {
    let delta = time.delta();
    for (start, end, color, timer) in lines.0.iter_mut() {
        gizmos.ray(*start, *end, *color);
        timer.tick(delta);
    }
    lines.clean();
}

#[derive(Resource)]
struct DebugLines(Vec<(Vec3, Vec3, Color, Timer)>);

impl DebugLines {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn push(&mut self, start: Vec3, end: Vec3, color: Color) {
        self.0.push((
            start,
            end,
            color,
            Timer::from_seconds(10.0, TimerMode::Once),
        ));
    }

    pub fn clean(&mut self) {
        self.0.retain(|(_, _, _, timer)| !timer.finished());
    }
}
