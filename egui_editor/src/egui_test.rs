use bevy::{
    prelude::*,
    render::{camera::Viewport, view::RenderLayers},
    window::PrimaryWindow,
};
use bevy_egui::{
    EguiContexts, EguiGlobalSettings, EguiPlugin, EguiPrimaryContextPass, PrimaryEguiContext, egui,
};

pub fn plugin(app: &mut App) {
    app.add_plugins(EguiPlugin::default())
        .add_systems(Startup, setup_camera_system)
        .add_systems(EguiPrimaryContextPass, ui_example_system);
}

pub fn setup_camera_system(
    mut commands: Commands,
    mut egui_global_settings: ResMut<EguiGlobalSettings>,
) {
    egui_global_settings.auto_create_primary_context = false;
    commands.spawn((
        Camera2d,
        PrimaryEguiContext,
        RenderLayers::none(),
        Camera {
            order: 1,
            ..default()
        },
    ));

    commands.spawn((Camera2d, MainView));
}

#[derive(Component, Reflect)]
pub struct MainView;

pub fn ui_example_system(
    mut contexts: EguiContexts,
    mut camera: Single<&mut Camera, With<MainView>>,
    window: Single<&mut Window, With<PrimaryWindow>>,
) -> Result {
    let top = egui::TopBottomPanel::top("my_panel")
        .resizable(true)
        .show(contexts.ctx_mut()?, |ui| {
            ui.label("Hello World! From `TopBottomPanel`, that must be before `CentralPanel`!");
            ui.allocate_space(ui.available_size())
        })
        .response
        .rect
        .height();
    let left = egui::SidePanel::left("left_panel")
        .resizable(false)
        .show(contexts.ctx_mut()?, |ui| {
            ui.label("left");
            ui.allocate_space(ui.available_size())
        })
        .response
        .rect
        .width();
    let bottom = egui::TopBottomPanel::bottom("bottom_panel")
        .show(contexts.ctx_mut()?, |ui| {
            ui.label("bottom");
            ui.allocate_space(ui.available_size())
        })
        .response
        .rect
        .height();
    let right = egui::SidePanel::right("right_panel")
        .resizable(true)
        .show(contexts.ctx_mut()?, |ui| {
            ui.label("right");
            ui.allocate_space(ui.available_size())
        })
        .response
        .rect
        .width();

    let pos = UVec2::new(
        (left * window.scale_factor()) as u32,
        (top * window.scale_factor()) as u32,
    );
    let size = UVec2::new(window.physical_width(), window.physical_height())
        - pos
        - UVec2::new(
            (right * window.scale_factor()) as u32,
            (bottom * window.scale_factor()) as u32,
        );
    camera.viewport = Some(Viewport {
        physical_position: pos,
        physical_size: size,
        ..default()
    });
    Ok(())
}
