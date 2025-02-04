use avian2d::{math::*, prelude::*};
use bevy::window::WindowMode;
use bevy::{asset::AssetMetaCheck, prelude::*};

mod curve;
mod mouse;
use curve::{ControlPoints, CurvePlugin};
use mouse::{MousePlugin, MousePosition};

const PARTICLE_RADIUS: f32 = 4.0;
const PARTICLE_MASS: f32 = 1.0;
const PARTICLE_COLOR: Color = Color::srgb(0.2, 0.7, 0.9);

#[derive(Component)]
struct Edge {
    chain: Vec<Entity>,
}

#[derive(Component)]
struct CurrentEdge;

#[derive(Resource)]
struct ParticleAssets {
    mesh: Handle<Mesh>,
    material: Handle<ColorMaterial>,
}

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
        CurvePlugin,
        MousePlugin,
    ));

    app.insert_resource(Gravity(Vector::ZERO));

    app.insert_resource(ClearColor(Color::srgb(0.05, 0.05, 0.1)))
        .insert_resource(SubstepCount(50))
        .add_systems(Startup, setup)
        .add_systems(Update, create_edge);

    app.run();
}

const EDGE_POINTS_MINIMUM_DISTANCE: f32 = 50.0;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d);

    // Create and store the particle assets
    let particle_assets = ParticleAssets {
        mesh: meshes.add(Circle::new(PARTICLE_RADIUS)),
        material: materials.add(PARTICLE_COLOR),
    };
    commands.insert_resource(particle_assets);
}

fn create_edge(
    mut commands: Commands,
    buttons: Res<ButtonInput<MouseButton>>,
    mouse_position: Res<MousePosition>,
    mut current_edge: Query<(Entity, &mut Edge, &mut ControlPoints), With<CurrentEdge>>,
    particle_assets: Res<ParticleAssets>,
    transform: Query<&Transform>,
) {
    let cursor_world_pos = mouse_position.position;

    match (
        buttons.pressed(MouseButton::Left),
        current_edge.get_single_mut(),
    ) {
        // Left mouse pressed and there's a current edge
        (true, Ok((_, mut current_edge, mut control_points))) => {
            let end_pos = transform
                .get(*current_edge.chain.last().unwrap())
                .unwrap()
                .translation
                .xy();
            let distance = end_pos.distance(cursor_world_pos);
            if distance < EDGE_POINTS_MINIMUM_DISTANCE {
                return;
            }

            // create one more particle
            let new_particle = commands
                .spawn((
                    RigidBody::Dynamic,
                    Transform::from_xyz(cursor_world_pos.x, cursor_world_pos.y, 0.0),
                    MassPropertiesBundle::from_shape(
                        &Circle::new(PARTICLE_RADIUS as f32),
                        PARTICLE_MASS,
                    ),
                    Mesh2d(particle_assets.mesh.clone()),
                    MeshMaterial2d(particle_assets.material.clone()),
                ))
                .id();

            // update the current edge
            current_edge.chain.push(new_particle);
            control_points.points.push(cursor_world_pos);
        }

        // Left mouse pressed but no current edge
        (true, Err(_)) => {
            // Create a new start
            let control_points = ControlPoints {
                points: vec![cursor_world_pos],
            };
            let start = commands
                .spawn((
                    RigidBody::Static,
                    Transform::from_xyz(cursor_world_pos.x, cursor_world_pos.y, 0.0),
                    Mesh2d(particle_assets.mesh.clone()),
                    MeshMaterial2d(particle_assets.material.clone()),
                    control_points,
                ))
                .id();

            commands.spawn((Edge { chain: vec![start] }, CurrentEdge));
        }

        // Left mouse not pressed but there's a current edge
        (false, Ok((current_edge_entity, mut current_edge, mut control_points))) => {
            let end = commands
                .spawn((
                    RigidBody::Static,
                    Transform::from_xyz(cursor_world_pos.x, cursor_world_pos.y, 0.0),
                    Mesh2d(particle_assets.mesh.clone()),
                    MeshMaterial2d(particle_assets.material.clone()),
                ))
                .id();

            current_edge.chain.push(end);
            control_points.points.push(cursor_world_pos);
            commands.entity(current_edge_entity).remove::<CurrentEdge>();
        }

        // Left mouse not pressed and no current edge
        (false, Err(_)) => {}
    }
}
