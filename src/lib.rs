use std::f32::consts;

use embedded_graphics_core::pixelcolor::Rgb565;
use mesh::K3dMesh;
use mesh::RenderMode;
use nalgebra::Isometry3;
use nalgebra::Perspective3;
use nalgebra::Point2;
use nalgebra::Point3;
use nalgebra::Similarity3;
use nalgebra::UnitQuaternion;
use nalgebra::Vector3;

pub mod framebuffer;
pub mod perfcounter;

pub mod mesh;

#[derive(Debug)]
pub enum DrawPrimitive {
    ColoredPoint(Point2<i32>, Rgb565),
    Line(Point2<i32>, Point2<i32>, Rgb565),
}

pub struct K3dCamera {
    pub position: Point3<f32>,
    fov: f32,
    view_matrix: nalgebra::Matrix4<f32>,
    vp_matrix: nalgebra::Matrix4<f32>,
    aspect_ratio: f32,
}

impl K3dCamera {
    pub fn new(aspect_ratio: f32) -> K3dCamera {
        K3dCamera {
            position: Point3::new(0.0, 0.0, 0.0),
            fov: consts::PI / 2.0,
            view_matrix: nalgebra::Matrix4::identity(),
            vp_matrix: nalgebra::Matrix4::identity(),
            aspect_ratio,
        }
    }

    pub fn set_position(&mut self, pos: Point3<f32>) {
        self.position = pos;

        self.update_vp();
    }

    pub fn set_fovy(&mut self, fovy: f32) {
        self.fov = fovy;

        self.update_vp();
    }

    pub fn set_target(&mut self, target: Point3<f32>) {
        let view = Isometry3::look_at_rh(&self.position, &target, &Vector3::y());

        self.view_matrix = view.to_homogeneous();

        self.update_vp();
    }

    pub fn set_attitude(&mut self, roll: f32, pitch: f32, yaw: f32) {
        let rotation = UnitQuaternion::from_euler_angles(roll, pitch, yaw);
        let translation = self.position;
        let isometry = Isometry3::from_parts(translation.coords.into(), rotation);
        self.view_matrix = isometry.to_homogeneous();
        self.update_vp();
    }

    pub fn update_vp(&mut self) {
        let projection = Perspective3::new(self.aspect_ratio, self.fov, 1.0, 10.0);
        let view_projection = projection.as_matrix() * self.view_matrix;
        self.vp_matrix = view_projection;
    }
}

pub struct K3dengine {
    pub camera: K3dCamera,
    width: u16,
    height: u16,
}

impl K3dengine {
    pub fn new(width: u16, height: u16) -> K3dengine {
        K3dengine {
            camera: K3dCamera::new(width as f32 / height as f32),
            width,
            height,
        }
    }

    fn transform_point(&self, point: &Point3<f32>, model_matrix: Similarity3<f32>) -> Point2<i32> {
        let point = model_matrix.transform_point(point);
        let point = self.camera.vp_matrix.transform_point(&point);
        Point2::new(
            self.width as i32 / 2 - ((self.width / 2) as f32 * point.x) as i32,
            self.height as i32 / 2 - ((self.height / 2) as f32 * point.y) as i32,
        )
    }

    pub fn render<'a, MS, F>(&self, meshes: MS, mut callback: F)
    where
        MS: IntoIterator<Item = &'a K3dMesh<'a>>,
        F: FnMut(DrawPrimitive),
    {
        for mesh in meshes {
            if mesh.geometry.vertices.is_empty() {
                continue;
            }

            match mesh.render_mode {
                RenderMode::Points => {
                    let screen_space_points = mesh
                        .geometry
                        .vertices
                        .iter()
                        .map(|v| self.transform_point(&t2p3(v), mesh.model_matrix));

                    if mesh.geometry.colors.len() == mesh.geometry.vertices.len() {
                        // vertices are colored
                        for (point, color) in screen_space_points.zip(mesh.geometry.colors) {
                            callback(DrawPrimitive::ColoredPoint(point, *color));
                        }
                        continue;
                    }

                    // global mesh color
                    for point in screen_space_points {
                        callback(DrawPrimitive::ColoredPoint(point, mesh.color));
                    }
                }
                RenderMode::Lines => {
                    if !mesh.geometry.lines.is_empty() {
                        for line in mesh.geometry.lines {
                            let p1 = self.transform_point(
                                &t2p3(&mesh.geometry.vertices[line.0]),
                                mesh.model_matrix,
                            );
                            let p2 = self.transform_point(
                                &t2p3(&mesh.geometry.vertices[line.1]),
                                mesh.model_matrix,
                            );
                            callback(DrawPrimitive::Line(p1, p2, mesh.color));
                        }
                    } else if !mesh.geometry.faces.is_empty() {
                        for face in mesh.geometry.faces {
                            let p1 = self.transform_point(
                                &t2p3(&mesh.geometry.vertices[face.0]),
                                mesh.model_matrix,
                            );
                            let p2 = self.transform_point(
                                &t2p3(&mesh.geometry.vertices[face.1]),
                                mesh.model_matrix,
                            );
                            let p3 = self.transform_point(
                                &t2p3(&mesh.geometry.vertices[face.2]),
                                mesh.model_matrix,
                            );

                            callback(DrawPrimitive::Line(p1, p2, mesh.color));
                            callback(DrawPrimitive::Line(p2, p3, mesh.color));
                            callback(DrawPrimitive::Line(p3, p1, mesh.color));
                        }
                    }
                }
                RenderMode::Solid => todo!(),
            }
        }
    }
}

fn t2p3(t: &(f32, f32, f32)) -> Point3<f32> {
    Point3::new(t.0, t.1, t.2)
}
