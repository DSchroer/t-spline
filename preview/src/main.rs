/*
 * Copyright (C) 2026 Dominick Schroer
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use anyhow::Result;
use bevy::{
    camera_controller::free_camera::{FreeCamera, FreeCameraPlugin},
    color::palettes::tailwind,
    prelude::*,
};
use t_spline::{Command, Point3, TSpline};
use t_spline_commands::tessellate::Tessellate;

fn main() -> Result<()> {
    let spline: TSpline<f64> = TSpline::new_simple();
    let points = Tessellate { resolution: 100 }.apply(&spline);

    App::new()
        .insert_resource(ClearColor(tailwind::BLUE_50.into()))
        .insert_resource(Render { points, spline })
        .add_plugins(DefaultPlugins)
        .add_plugins(FreeCameraPlugin)
        .add_systems(Startup, (setup, draw_points, draw_control))
        .add_systems(Update, draw_cage)
        .run();

    Ok(())
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
}

fn draw_control(
    render: Res<Render>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
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
            Transform::from_xyz(
                p.geometry.x as f32,
                p.geometry.y as f32,
                p.geometry.z as f32,
            ),
        ));
    }
}

fn draw_cage(render: Res<Render>, mut gizmos: Gizmos) {
    for e in &render.spline.mesh().edges {
        let from = render.spline.mesh().vertex(e.origin);
        let to = render
            .spline
            .mesh()
            .vertex(render.spline.mesh().edge(e.next).origin);
        gizmos.line(
            Vec3::new(
                from.geometry.x as f32,
                from.geometry.y as f32,
                from.geometry.z as f32,
            ),
            Vec3::new(
                to.geometry.x as f32,
                to.geometry.y as f32,
                to.geometry.z as f32,
            ),
            tailwind::GREEN_500,
        );
    }
}
