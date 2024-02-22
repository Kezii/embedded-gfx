use std::f32::consts;

use embedded_graphics_core::pixelcolor::Rgb565;
use embedded_graphics_core::pixelcolor::WebColors;
use nalgebra::Isometry3;
use nalgebra::Perspective3;
use nalgebra::Point2;
use nalgebra::Point3;
use nalgebra::Similarity3;
use nalgebra::UnitQuaternion;
use nalgebra::Vector3;

pub mod framebuffer;

#[derive(Debug)]
pub enum DrawPrimitive {
    ColoredPoint(Point2<i32>, Rgb565),
    Line(Point2<i32>, Point2<i32>, Rgb565),
}

#[derive(Debug, PartialEq)]
pub enum RenderMode {
    Points,
    Lines,
    Solid,
}

pub struct K3dMesh {
    model_matrix: Similarity3<f32>,
    pub vertices: Vec<nalgebra::Point3<f32>>,
    pub colors: Vec<Rgb565>,
    pub faces: Vec<(usize, usize, usize)>,
    pub color: Rgb565,
    pub render_mode: RenderMode,
    pub optimized_lines: Option<Vec<(usize, usize)>>,
}

impl K3dMesh {
    pub fn new(vertices: Vec<nalgebra::Point3<f32>>) -> K3dMesh {
        K3dMesh {
            model_matrix: Similarity3::new(Vector3::new(0.0, 0.0, 0.0), nalgebra::zero(), 1.0),
            vertices,
            faces: Vec::new(),
            colors: Vec::new(),
            color: Rgb565::CSS_WHITE,
            render_mode: RenderMode::Points,
            optimized_lines: None,
        }
    }

    pub fn set_color(&mut self, color: Rgb565) {
        self.color = color;
    }

    pub fn set_render_mode(&mut self, mode: RenderMode) {
        self.render_mode = mode;
    }

    pub fn optimize_lines(&mut self) {
        let mut lines = Vec::new();
        for face in &self.faces {
            for line in &[(face.0, face.1), (face.1, face.2), (face.2, face.0)] {
                let (a, b) = if line.0 < line.1 {
                    (line.0, line.1)
                } else {
                    (line.1, line.0)
                };
                if !lines.contains(&(a, b)) {
                    lines.push((a, b));
                }
            }
        }
        self.optimized_lines = Some(lines);
    }

    pub fn set_position(&mut self, x: f32, y: f32, z: f32) {
        self.model_matrix.isometry.translation.x = x;
        self.model_matrix.isometry.translation.y = y;
        self.model_matrix.isometry.translation.z = z;
    }

    pub fn set_attitude(&mut self, roll: f32, pitch: f32, yaw: f32) {
        self.model_matrix.isometry.rotation = UnitQuaternion::from_euler_angles(roll, pitch, yaw);
    }

    pub fn set_target(&mut self, target: Point3<f32>) {
        let view = Similarity3::look_at_rh(
            &self.model_matrix.isometry.translation.vector.into(),
            &target,
            &Vector3::y(),
            1.0,
        );

        self.model_matrix = view;
    }

    pub fn set_scale(&mut self, s: f32) {
        if s == 0.0 {
            return;
        }
        self.model_matrix.set_scaling(s)
    }
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
        MS: IntoIterator<Item = &'a K3dMesh>,
        F: FnMut(DrawPrimitive),
    {
        for mesh in meshes {
            if mesh.vertices.is_empty() {
                continue;
            }

            match mesh.render_mode {
                RenderMode::Points => {
                    let screen_space_points = mesh
                        .vertices
                        .iter()
                        .map(|v| self.transform_point(v, mesh.model_matrix));

                    if mesh.colors.len() == mesh.vertices.len() {
                        // vertices are colored
                        for (point, color) in screen_space_points.zip(&mesh.colors) {
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
                    if let Some(lines) = &mesh.optimized_lines {
                        for line in lines {
                            let p1 =
                                self.transform_point(&mesh.vertices[line.0], mesh.model_matrix);
                            let p2 =
                                self.transform_point(&mesh.vertices[line.1], mesh.model_matrix);
                            callback(DrawPrimitive::Line(p1, p2, mesh.color));
                        }
                    } else {
                        for face in &mesh.faces {
                            let p1 =
                                self.transform_point(&mesh.vertices[face.0], mesh.model_matrix);
                            let p2 =
                                self.transform_point(&mesh.vertices[face.1], mesh.model_matrix);
                            let p3 =
                                self.transform_point(&mesh.vertices[face.2], mesh.model_matrix);

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
