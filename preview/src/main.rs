use bevy::{
    camera_controller::free_camera::{FreeCamera, FreeCameraPlugin},
    color::palettes::tailwind,
    prelude::*,
};
use t_spline::{Command, Point3, TSpline};
use t_spline::tmesh::TMesh;
use t_spline_commands::tessellate::Tessellate;

fn main() {
    let mut spline: TSpline<f64> = TSpline::new_unit_square();
    spline.apply_mut(&mut |m: &mut TMesh<f64>| m.vertices[0].geometry.z = 0.5);

    let points = Tessellate { resolution: 50 }.apply(&spline);

    App::new()
        .insert_resource(ClearColor(tailwind::BLUE_50.into()))
        .insert_resource(Render { points, spline })
        .add_plugins(DefaultPlugins)
        .add_plugins(FreeCameraPlugin)
        .add_systems(Startup, (setup, draw_points))
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(10.0, 12.0, 16.0).looking_at(Vec3::ZERO, Vec3::Y),
        DirectionalLight { ..default() },
        FreeCamera {
            sensitivity: 0.2,
            friction: 25.0,
            walk_speed: 3.0,
            run_speed: 9.0,
            ..default()
        },
    ));
}

#[derive(Resource)]
struct Render {
    points: Vec<Point3<f64>>,
    spline: TSpline<f64>,
}

fn draw_points(
    render: Res<Render>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let point_mesh = meshes.add(Sphere::new(0.005));
    let point_mat = materials.add(StandardMaterial {
        base_color: tailwind::GREEN_500.into(),
        unlit: true,
        ..default()
    });

    for p in &render.points {
        commands.spawn((
            Mesh3d(point_mesh.clone()),
            MeshMaterial3d(point_mat.clone()),
            Transform::from_xyz(p.x as f32, p.y as f32, p.z as f32),
        ));
    }

    let control_mesh = meshes.add(Sphere::new(0.05));
    let control_mat = materials.add(StandardMaterial {
        base_color: tailwind::AMBER_500.into(),
        unlit: true,
        ..default()
    });

    for p in &render.spline.mesh().vertices {
        commands.spawn((
            Mesh3d(control_mesh.clone()),
            MeshMaterial3d(control_mat.clone()),
            Transform::from_xyz(p.geometry.x as f32, p.geometry.y as f32, p.geometry.z as f32),
        ));
    }
}
