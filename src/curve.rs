use bevy::prelude::*;

/// The curve presently being displayed. This is optional because there may not be enough control
/// points to actually generate a curve.
#[derive(Clone, Default, Component)]
pub struct Curve(Option<CubicCurve<Vec2>>);

#[derive(Clone, Component)]
pub struct ControlPoints {
    pub points: Vec<Vec2>,
}

pub struct CurvePlugin;

impl Plugin for CurvePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (draw_curve, update_curve));
    }
}

fn draw_curve(query: Query<&Curve>, mut gizmos: Gizmos) {
    for curve in query.iter() {
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
}

fn form_curve(control_points: &ControlPoints) -> Curve {
    let points = control_points.points.clone();

    if points.len() < 2 {
        Curve(None)
    } else {
        let curve = CubicCardinalSpline::new_catmull_rom(points)
            .to_curve()
            .unwrap();
        Curve(Some(curve))
    }
}

fn update_curve(mut query: Query<(&ControlPoints, &mut Curve)>) {
    for (control_points, mut curve) in query.iter_mut() {
        *curve = form_curve(control_points);
    }
}
