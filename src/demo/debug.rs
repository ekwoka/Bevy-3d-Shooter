use std::time::Duration;

use bevy::prelude::*;
use iyes_perf_ui::{PerfUiPlugin, prelude::*};

pub(super) fn plugin(app: &mut App) {
    #[cfg(debug_assertions)]
    app.add_systems(Update, draw_debug_lines)
        .add_plugins((
            bevy::diagnostic::FrameTimeDiagnosticsPlugin::default(),
            bevy::diagnostic::EntityCountDiagnosticsPlugin,
            PerfUiPlugin,
        ))
        .add_systems(Startup, add_perf_overlay);

    app.insert_resource(DebugLines::new());
}

fn add_perf_overlay(mut commands: Commands) {
    commands.spawn((
        PerfUiRoot::default(),
        // when we have lots of entries, we have to group them
        // into tuples, because of Bevy Rust syntax limitations
        (
            PerfUiEntryFPS::default(),
            PerfUiEntryFPSWorst::default(),
            PerfUiEntryFrameTime::default(),
            PerfUiEntryFrameTimeWorst::default(),
            PerfUiEntryFrameCount::default(),
            PerfUiEntryEntityCount::default(),
        ),
        (
            PerfUiEntryFixedTimeStep::default(),
            PerfUiEntryFixedOverstep::default(),
            PerfUiEntryRunningTime::default(),
            PerfUiEntryClock::default(),
        ),
    ));
}

#[cfg_attr(not(debug_assertions), allow(dead_code))]
fn draw_debug_lines(time: Res<Time>, mut lines: ResMut<DebugLines>, mut gizmos: Gizmos) {
    let delta = time.delta();
    let should_clean = lines.run_all(&mut gizmos, delta);
    if should_clean {
        lines.clean();
    }
}

#[derive(Resource)]
#[cfg_attr(not(debug_assertions), allow(dead_code))]
pub struct DebugLines(Vec<(Box<dyn Fn(&mut Gizmos) + Send + Sync>, Timer)>);

#[cfg_attr(not(debug_assertions), allow(dead_code))]
impl DebugLines {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn run_all(&mut self, gizmos: &mut Gizmos, delta: Duration) -> bool {
        self.0
            .iter_mut()
            .map(move |(cb, timer)| {
                cb(gizmos);
                timer.tick(delta);
                timer
            })
            .filter(|timer| timer.just_finished())
            .count()
            != 0
    }

    pub fn push<T: Fn(&mut Gizmos) + Send + Sync + 'static>(&mut self, _func: T) {
        #[cfg(debug_assertions)]
        self.0
            .push((Box::new(_func), Timer::from_seconds(5.0, TimerMode::Once)));
    }

    pub fn clean(&mut self) {
        self.0.retain(|(_, timer)| !timer.finished());
    }
}
