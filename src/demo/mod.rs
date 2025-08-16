//! Demo gameplay. All of these modules are only intended for demonstration
//! purposes and should be replaced with your own game logic.
//! Feel free to change the logic found here if you feel like tinkering around
//! to get a feeling for the template.

use bevy::prelude::*;

pub mod bullet_trajectory;
pub mod debug;
pub mod level;
mod movement;
pub mod player;
pub mod target;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        debug::plugin,
        level::plugin,
        movement::plugin,
        player::plugin,
        target::plugin,
    ));
}
