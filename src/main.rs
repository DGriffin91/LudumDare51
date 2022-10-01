#![allow(clippy::too_many_arguments, clippy::type_complexity)]

use bevy::{
    math::vec3,
    prelude::*,
    window::{PresentMode, WindowMode, WindowResizeConstraints},
};
use bevy_mod_picking::*;

fn main() {
    let mut app = App::new();
    app.insert_resource(WindowDescriptor {
        title: "LD51".to_string(),
        width: 1280.0,
        height: 720.0,
        position: WindowPosition::Automatic,
        resize_constraints: WindowResizeConstraints {
            min_width: 256.0,
            min_height: 256.0,
            ..Default::default()
        },
        scale_factor_override: Some(1.0), //Needed for some mobile devices, but disables scaling
        present_mode: PresentMode::AutoVsync,
        resizable: true,
        decorations: true,
        cursor_locked: false,
        cursor_visible: true,
        mode: WindowMode::Windowed,
        transparent: false,
        canvas: Some("#bevy".to_string()),
        fit_canvas_to_parent: true,
    })
    .insert_resource(ClearColor(Color::BLACK))
    .add_plugins(DefaultPlugins)
    .add_plugins(DefaultPickingPlugins)
    .add_plugin(DebugCursorPickingPlugin)
    .add_plugin(DebugEventsPickingPlugin)
    .add_startup_system(setup)
    .add_system_to_stage(CoreStage::PostUpdate, print_events);

    #[cfg(target_arch = "wasm32")]
    {
        app.add_plugin(bevy_web_resizer::Plugin);
    }

    app.run();
}

pub fn print_events(mut events: EventReader<PickingEvent>) {
    for event in events.iter() {
        match event {
            PickingEvent::Selection(e) => info!("A selection event happened: {:?}", e),
            PickingEvent::Hover(e) => info!("Egads! A hover event!? {:?}", e),
            PickingEvent::Clicked(e) => info!("Gee Willikers, it's a click! {:?}", e),
        }
    }
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // plane
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane { size: 24.0 })),
            material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
            ..default()
        })
        .insert_bundle(PickableBundle::default());

    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::UVSphere {
                radius: 0.5,
                ..default()
            })),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..default()
        })
        .insert_bundle(PickableBundle::default());

    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::UVSphere {
                radius: 0.5,
                ..default()
            })),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            transform: Transform::from_xyz(1.0, 0.5, 0.0),
            ..default()
        })
        .insert_bundle(PickableBundle::default());

    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::UVSphere {
                radius: 0.5,
                ..default()
            })),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            transform: Transform::from_xyz(2.0, 0.5, 0.0),
            ..default()
        })
        .insert_bundle(PickableBundle::default());

    // light
    commands.spawn_bundle(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });

    // camera
    commands
        .spawn_bundle(Camera3dBundle {
            transform: Transform::from_xyz(0.0, 24.0, 18.0)
                .looking_at(vec3(0.0, 0.0, 3.0), Vec3::Y),
            ..default()
        })
        .insert_bundle(PickingCameraBundle::default());
}
