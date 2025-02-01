use avian2d::{math::*, prelude::*};
use bevy::window::{PrimaryWindow, WindowMode};
use bevy::{asset::AssetMetaCheck, prelude::*};

fn main() {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins
            .set(AssetPlugin {
                meta_check: AssetMetaCheck::Never, // https://github.com/bevyengine/bevy/issues/10157#issuecomment-2217168402
                ..default()
            })
            .set(ImagePlugin::default_nearest())
            .set(WindowPlugin {
                primary_window: Some(Window {
                    // Borderless looks distorted when running in the web
                    #[cfg(not(target_arch = "wasm32"))]
                    mode: WindowMode::BorderlessFullscreen(MonitorSelection::Current),
                    fit_canvas_to_parent: true,
                    prevent_default_event_handling: false,
                    resizable: true,
                    ..default()
                }),
                ..default()
            }),
        PhysicsPlugins::default(),
    ));

    app.insert_resource(ClearColor(Color::srgb(0.05, 0.05, 0.1)))
        .insert_resource(SubstepCount(50))
        .insert_resource(Gravity(Vector::NEG_Y * 1000.0))
        .add_systems(Startup, setup)
        .add_systems(Update, follow_mouse)
        .run();

    app.run();
}

#[derive(Component)]
struct FollowMouse;

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    commands.spawn(Camera2d);

    let particle_count = 50;
    let particle_radius = 1.2;
    let particle_mesh = meshes.add(Circle::new(particle_radius as f32));
    let particle_material = materials.add(Color::srgb(0.2, 0.7, 0.9));
    let particle_mass = 1.0;
    let compliance = 0.0000001; // TODO: explore this. It seems to be important

    // Spawn kinematic particle that can follow the mouse
    let mut previous_particle = commands
        .spawn((
            RigidBody::Kinematic,
            FollowMouse,
            Mesh2d(particle_mesh.clone()),
            MeshMaterial2d(particle_material.clone()),
        ))
        .id();

    // Spawn the rest of the particles, connecting each one to the previous one with joints
    for i in 1..particle_count {
        let current_particle = commands
            .spawn((
                RigidBody::Dynamic,
                MassPropertiesBundle::from_shape(
                    &Circle::new(particle_radius as f32),
                    particle_mass,
                ),
                Mesh2d(particle_mesh.clone()),
                MeshMaterial2d(particle_material.clone()),
                Transform::from_xyz(0.0, -i as f32 * (particle_radius as f32 * 2.0 + 1.0), 0.0),
            ))
            .id();

        commands.spawn(
            RevoluteJoint::new(previous_particle, current_particle)
                .with_local_anchor_2(Vector::Y * (particle_radius * 2.0 + 1.0))
                .with_compliance(compliance),
        );

        previous_particle = current_particle;
    }

    // Spawn the fixed anchor and the last joint
    let anchor_particle = commands
        .spawn((
            RigidBody::Static,
            Transform::from_xyz(0.0, 0.0, 0.0),
            Mesh2d(meshes.add(Circle::new(particle_radius * 2.0))),
            MeshMaterial2d(materials.add(Color::srgb(0.2, 0.7, 0.9))),
        ))
        .id();

    commands.spawn(
        RevoluteJoint::new(previous_particle, anchor_particle)
            .with_local_anchor_2(Vector::Y * (particle_radius * 2.0 + 1.0))
            .with_compliance(compliance),
    );
}

fn follow_mouse(
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform)>,
    mut follower: Query<&mut Transform, With<FollowMouse>>,
) {
    if buttons.pressed(MouseButton::Left) {
        let window = windows.single();
        let (camera, camera_transform) = camera.single();
        let mut follower_position = follower.single_mut();

        if let Some(cursor_world_pos) = window
            .cursor_position()
            .and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor).ok())
        {
            follower_position.translation =
                cursor_world_pos.extend(follower_position.translation.z);
        }
    }
}
