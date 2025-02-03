use avian2d::{math::*, prelude::*};
use bevy::window::{PrimaryWindow, WindowMode};
use bevy::{asset::AssetMetaCheck, prelude::*};

const PARTICLE_COUNT: i32 = 50;
const PARTICLE_RADIUS: f32 = 1.2;
const PARTICLE_MASS: f32 = 1.0;
const JOINT_COMPLIANCE: f32 = 0.0000001; // TODO: explore this. It seems to be important
const PARTICLE_COLOR: Color = Color::srgb(0.2, 0.7, 0.9);

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
        .add_systems(Update, create_edge);

    app.insert_resource(EdgeJointSpawnTimer(Timer::from_seconds(
        EDGE_JOINT_SPAWN_INTERVAL,
        TimerMode::Repeating,
    )));

    app.run();
}

#[derive(Resource)]
struct EdgeJointSpawnTimer(Timer);

const EDGE_JOINT_SPAWN_INTERVAL: f32 = 1.0 / 30.0;

#[derive(Component)]
struct FollowMouse;

#[derive(Component)]
struct Edge {
    start: Entity,
    chain: Vec<Entity>,
    end: Entity,
}

#[derive(Component)]
struct CurrentEdge;

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn create_edge(
    mut commands: Commands,
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform)>,
    mut current_edge: Query<(Entity, &mut Edge), With<CurrentEdge>>,
    // meshes: Res<Assets<Mesh>>,
    // materials: Res<Assets<ColorMaterial>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut edge_joint_spawn_timer: ResMut<EdgeJointSpawnTimer>,
    time: Res<Time>,
) {
    if !edge_joint_spawn_timer.0.tick(time.delta()).just_finished() {
        return;
    }
    // let particle_mesh = meshes.get("particle").unwrap();
    // let particle_material = materials.get("particle").unwrap();

    let particle_mesh = meshes.add(Circle::new(PARTICLE_RADIUS as f32));
    let particle_material = materials.add(PARTICLE_COLOR);

    if buttons.pressed(MouseButton::Left) {
        let window = windows.single();
        let (camera, camera_transform) = camera.single();

        if let Some(cursor_world_pos) = window
            .cursor_position()
            .and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor).ok())
        {
            // Check if there's a current edge
            if let Ok((_, mut current_edge)) = current_edge.get_single_mut() {
                // create one more particle
                let new_particle = commands
                    .spawn((
                        RigidBody::Dynamic,
                        Transform::from_xyz(cursor_world_pos.x, cursor_world_pos.y, 0.0),
                        MassPropertiesBundle::from_shape(
                            &Circle::new(PARTICLE_RADIUS as f32),
                            PARTICLE_MASS,
                        ),
                        Mesh2d(particle_mesh.clone()),
                        MeshMaterial2d(particle_material.clone()),
                    ))
                    .id();

                // create a joint between the new particle and the last particle in the chain
                commands.spawn(
                    RevoluteJoint::new(current_edge.end, new_particle)
                        .with_compliance(JOINT_COMPLIANCE),
                );

                // update the current edge
                current_edge.chain.push(new_particle);

                current_edge.end = new_particle;
            } else {
                // Create a new start
                let start = commands
                    .spawn((
                        RigidBody::Static,
                        Transform::from_xyz(cursor_world_pos.x, cursor_world_pos.y, 0.0),
                        Mesh2d(particle_mesh.clone()),
                        MeshMaterial2d(particle_material.clone()),
                    ))
                    .id();

                commands.spawn((
                    Edge {
                        start,
                        chain: vec![],
                        end: start,
                    },
                    CurrentEdge,
                ));
            }
        }
    } else {
        if let Ok((current_edge_entity, mut current_edge)) = current_edge.get_single_mut() {
            current_edge.end = *current_edge.chain.last().unwrap_or(&current_edge.start);

            commands.entity(current_edge_entity).remove::<CurrentEdge>();
        }
        // let current_edge = current_edge.single_mut();
        // commands.entity(current_edge.end).despawn_recursive();
        // commands.entity(current_edge.start).despawn_recursive();
        // current_edge.chain.iter().for_each(|entity| {
        //     commands.entity(*entity).despawn_recursive();
        // });
    }
}

fn old_setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    commands.spawn(Camera2d);

    let particle_mesh = meshes.add(Circle::new(PARTICLE_RADIUS as f32));
    let particle_material = materials.add(PARTICLE_COLOR);

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
    for i in 1..PARTICLE_COUNT {
        let current_particle = commands
            .spawn((
                RigidBody::Dynamic,
                MassPropertiesBundle::from_shape(
                    &Circle::new(PARTICLE_RADIUS as f32),
                    PARTICLE_MASS,
                ),
                Mesh2d(particle_mesh.clone()),
                MeshMaterial2d(particle_material.clone()),
                Transform::from_xyz(0.0, -i as f32 * (PARTICLE_RADIUS as f32 * 2.0 + 1.0), 0.0),
            ))
            .id();

        commands.spawn(
            RevoluteJoint::new(previous_particle, current_particle)
                .with_local_anchor_2(Vector::Y * (PARTICLE_RADIUS * 2.0 + 1.0))
                .with_compliance(JOINT_COMPLIANCE),
        );

        previous_particle = current_particle;
    }

    // Spawn the fixed anchor and the last joint
    let anchor_particle = commands
        .spawn((
            RigidBody::Static,
            Transform::from_xyz(0.0, 0.0, 0.0),
            Mesh2d(meshes.add(Circle::new(PARTICLE_RADIUS * 2.0))),
            MeshMaterial2d(materials.add(PARTICLE_COLOR)),
        ))
        .id();

    commands.spawn(
        RevoluteJoint::new(previous_particle, anchor_particle)
            .with_local_anchor_2(Vector::Y * (PARTICLE_RADIUS * 2.0 + 1.0))
            .with_compliance(JOINT_COMPLIANCE),
    );
}
