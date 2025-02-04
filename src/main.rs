use avian2d::{math::*, prelude::*};
use bevy::math::vec2;
use bevy::window::{PrimaryWindow, WindowMode};
use bevy::{asset::AssetMetaCheck, prelude::*};

const PARTICLE_RADIUS: f32 = 4.0;
const PARTICLE_MASS: f32 = 1.0;
const PARTICLE_COLOR: Color = Color::srgb(0.2, 0.7, 0.9);

#[derive(Resource, Default)]
struct MousePosition {
    position: Vec2,
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
    ));

    app.insert_resource(Gravity(Vector::ZERO));

    app.insert_resource(ClearColor(Color::srgb(0.05, 0.05, 0.1)))
        .insert_resource(SubstepCount(50))
        .insert_resource(MousePosition::default())
        .add_systems(Startup, setup)
        .add_systems(Update, (create_edge, track_mouse, draw_curve));

    app.run();
}

const EDGE_POINTS_MINIMUM_DISTANCE: f32 = 50.0;

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

    // Starting data for [`ControlPoints`]:
    let default_points = vec![
        vec2(-50., -20.),
        vec2(-25., 25.),
        vec2(25., 25.),
        vec2(50., -20.),
    ];

    let default_control_data = ControlPoints {
        points: default_points,
    };

    let curve = form_curve(&default_control_data);
    commands.insert_resource(curve);
}

/// The curve presently being displayed. This is optional because there may not be enough control
/// points to actually generate a curve.
#[derive(Clone, Default, Resource)]
struct Curve(Option<CubicCurve<Vec2>>);
fn draw_curve(curve: Res<Curve>, mut gizmos: Gizmos) {
    let Some(ref curve) = curve.0 else {
        return;
    };
    // Scale resolution with curve length so it doesn't degrade as the length increases.
    let resolution = 100 * curve.segments().len();
    gizmos.linestrip(
        curve.iter_positions(resolution).map(|pt| pt.extend(0.0)),
        Color::srgb(1.0, 1.0, 1.0),
    );
}

fn form_curve(control_points: &ControlPoints) -> Curve {
    let points = control_points.points.clone();

    let spline = CubicBSpline::new(points);
    Curve(spline.to_curve().ok())

    // Saving for later
    // let spline = CubicCardinalSpline::new_catmull_rom(points);
    // Curve(spline.to_curve().ok())
}

#[derive(Clone, Resource)]
struct ControlPoints {
    points: Vec<Vec2>,
}

fn create_edge(
    mut commands: Commands,
    buttons: Res<ButtonInput<MouseButton>>,
    mouse_position: Res<MousePosition>,
    mut current_edge: Query<(Entity, &mut Edge), With<CurrentEdge>>,
    particle_assets: Res<ParticleAssets>,
    transform: Query<&Transform>,
) {
    let cursor_world_pos = mouse_position.position;

    match (
        buttons.pressed(MouseButton::Left),
        current_edge.get_single_mut(),
    ) {
        // Left mouse pressed and there's a current edge
        (true, Ok((_, mut current_edge))) => {
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
        }

        // Left mouse pressed but no current edge
        (true, Err(_)) => {
            // Create a new start
            let start = commands
                .spawn((
                    RigidBody::Static,
                    Transform::from_xyz(cursor_world_pos.x, cursor_world_pos.y, 0.0),
                    Mesh2d(particle_assets.mesh.clone()),
                    MeshMaterial2d(particle_assets.material.clone()),
                ))
                .id();

            commands.spawn((Edge { chain: vec![start] }, CurrentEdge));
        }

        // Left mouse not pressed but there's a current edge
        (false, Ok((current_edge_entity, mut current_edge))) => {
            let end = commands
                .spawn((
                    RigidBody::Static,
                    Transform::from_xyz(cursor_world_pos.x, cursor_world_pos.y, 0.0),
                    Mesh2d(particle_assets.mesh.clone()),
                    MeshMaterial2d(particle_assets.material.clone()),
                ))
                .id();

            current_edge.chain.push(end);
            commands.entity(current_edge_entity).remove::<CurrentEdge>();
        }

        // Left mouse not pressed and no current edge
        (false, Err(_)) => {}
    }
}

fn track_mouse(
    mut mouse_position: ResMut<MousePosition>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform)>,
) {
    let window = windows.single();
    let (camera, camera_transform) = camera.single();

    if let Some(cursor_world_pos) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor).ok())
    {
        mouse_position.position = cursor_world_pos;
    }
}
