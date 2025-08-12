use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, draw_debug_lines);
    app.insert_resource(DebugLines::new());
}

fn draw_debug_lines(
    time: Res<Time>,
    mut lines: ResMut<DebugLines>,
    #[cfg(debug_assertions)] mut gizmos: Gizmos,
) {
    let delta = time.delta();
    let mut should_clean = false;
    for (_cb, timer) in lines.0.iter_mut() {
        #[cfg(debug_assertions)]
        _cb(&mut gizmos);
        timer.tick(delta);
        if timer.just_finished() {
            should_clean = true;
        }
    }
    if should_clean {
        lines.clean();
    }
}

#[derive(Resource)]
pub struct DebugLines(Vec<(Box<dyn Fn(&mut Gizmos) + Send + Sync>, Timer)>);

impl DebugLines {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn push(&mut self, func: Box<dyn Fn(&mut Gizmos) + Send + Sync>) {
        self.0
            .push((func, Timer::from_seconds(5.0, TimerMode::Once)));
    }

    pub fn clean(&mut self) {
        self.0.retain(|(_, timer)| !timer.finished());
    }
}
