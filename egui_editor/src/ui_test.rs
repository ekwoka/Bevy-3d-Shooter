use bevy::{
    camera::{Viewport, visibility::RenderLayers},
    dev_tools::states::log_transitions,
    math::Affine2,
    prelude::*,
};

use crate::{
    EditorMode,
    infinite_grid::{InfiniteGrid, InfiniteGridPlugin, InfiniteGridSettings},
};

#[cfg(feature = "avian")]
use avian3d::schedule::{Physics, PhysicsTime};

pub fn plugin(app: &mut App) {
    app.add_plugins(InfiniteGridPlugin)
        .init_state::<EditorMode>()
        .add_systems(Update, log_transitions::<EditorMode>)
        .add_systems(OnEnter(EditorMode::Edit), (setup_camera_system, build_ui))
        .add_systems(Update, update_viewport.run_if(in_state(EditorMode::Edit)))
        .add_observer(hover_menu_item)
        .add_observer(unhover_menu_item);
    #[cfg(feature = "avian")]
    {
        app.add_systems(OnEnter(EditorMode::Edit), pause_physics);
        app.add_systems(OnExit(EditorMode::Edit), resume_physics);
    }
}

#[cfg(feature = "avian")]
fn pause_physics(mut time: ResMut<Time<Physics>>) {
    info!("Physics Paused");
    time.pause();
}

#[cfg(feature = "avian")]
fn resume_physics(mut time: ResMut<Time<Physics>>) {
    info!("Physics Resumed");
    time.unpause();
}

#[derive(Component)]
pub struct MainView;

#[derive(Component)]
pub struct UiView;

#[derive(Component)]
pub struct ViewPort;

#[derive(Component)]
pub struct MenuBar;

#[derive(Component)]
pub struct MenuButton;

pub fn hover_menu_item(
    trigger: On<Pointer<Over>>,
    mut menu_items: Query<&mut BackgroundColor, With<MenuButton>>,
) {
    if let Ok(mut color) = menu_items.get_mut(trigger.entity) {
        *color = BackgroundColor::from(Color::linear_rgba(0.2, 0.2, 1.0, 0.50));
    }
}

pub fn unhover_menu_item(
    trigger: On<Pointer<Out>>,
    mut menu_items: Query<&mut BackgroundColor, With<MenuButton>>,
) {
    if let Ok(mut color) = menu_items.get_mut(trigger.entity) {
        *color = BackgroundColor::DEFAULT;
    }
}

pub fn build_ui(mut commands: Commands, type_registry: Res<AppTypeRegistry>) {
    let registry = type_registry.read();
    let component_info = registry
        .iter_with_data::<ReflectComponent>()
        .map(|(registration, _)| {
            registration
                .type_info()
                .ty()
                .type_path_table()
                .path()
                .to_string()
        })
        .filter(|path| path.contains("demo"))
        .collect::<Vec<_>>();
    commands.spawn((
        Node {
            padding: UiRect::all(Val::Px(1.)),
            flex_grow: 0.0,
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            // Prevent children from expanding the height of this node.
            min_height: Val::Px(0.),
            ..default()
        },
        RenderLayers::layer(1),
        children![
            (
                Node {
                    padding: UiRect::all(Val::Px(4.)),
                    flex_grow: 0.0,
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(8.0),
                    width: Val::Percent(100.),
                    border: UiRect::bottom(Val::Px(1.0)),
                    ..default()
                },
                MenuBar,
                BorderColor::all(Color::linear_rgb(0.7, 0.7, 0.7)),
                RenderLayers::layer(1),
                children![
                    menu_button("File"),
                    menu_button("Edit"),
                    menu_button("View"),
                    menu_button("Help")
                ]
            ),
            (
                Node {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    flex_grow: 1.0,
                    flex_shrink: 1.0,
                    width: Val::Percent(100.0),
                    height: Val::Percent(50.0),
                    ..default()
                },
                children![
                    (
                        Node {
                            padding: UiRect::all(Val::Px(1.)),
                            flex_grow: 0.0,
                            width: Val::Percent(50.0),
                            ..default()
                        },
                        RenderLayers::layer(1),
                        children![(
                            Text::new(component_info.join("\n")),
                            RenderLayers::layer(1),
                            TextFont {
                                font_size: 12.0,
                                ..default()
                            }
                        )]
                    ),
                    (
                        Node {
                            flex_grow: 1.0,
                            flex_shrink: 1.0,
                            width: Val::Percent(50.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        RenderLayers::layer(1),
                        ViewPort
                    )
                ]
            ),
            (
                Node {
                    padding: UiRect::all(Val::Px(1.)),
                    flex_grow: 0.0,
                    width: Val::Percent(100.0),
                    ..default()
                },
                RenderLayers::layer(1),
                children![(Text::new("Hi, Mom!"), RenderLayers::layer(1))]
            )
        ],
    ));
}

fn menu_button(test: impl Into<String>) -> impl Bundle {
    (
        Node {
            padding: UiRect::all(Val::Px(4.)),
            ..default()
        },
        MenuButton,
        BackgroundColor::DEFAULT,
        BorderRadius::all(Val::Px(2.0)),
        children![(
            Text::new(test),
            TextFont {
                font_size: 12.0,
                ..default()
            },
            TextColor::from(Color::linear_rgb(0.7, 0.7, 0.7))
        )],
    )
}

pub fn setup_camera_system(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        UiView,
        RenderLayers::layer(1),
        Camera {
            order: 1,
            ..default()
        },
    ));

    commands.spawn((
        InfiniteGrid,
        InfiniteGridSettings {
            x_axis_color: Color::WHITE,
            z_axis_color: Color::WHITE,
            major_line_color: Color::WHITE,
            minor_line_color: Color::WHITE,
            ..default()
        },
    ));
}

pub fn update_viewport(
    view_target: Single<(&ComputedNode, &UiGlobalTransform), With<ViewPort>>,
    mut camera: Single<&mut Camera, With<MainView>>,
) {
    let (viewport, transform) = *view_target;
    let size = viewport.size();
    if size.x == 0.0 || size.y == 0.0 {
        return;
    }
    let pos = Affine2::from(transform).translation - size * Vec2::new(0.5, 0.5);
    camera.viewport = Some(Viewport {
        physical_position: pos.as_uvec2(),
        physical_size: UVec2::new(size.x as u32, size.y as u32),
        ..default()
    });
}
