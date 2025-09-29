use bevy::prelude::*;
mod infinite_grid;
mod ui_test;

pub use ui_test::{MainView, plugin};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, States)]
pub enum EditorMode {
    #[default]
    Edit,
    View,
}
