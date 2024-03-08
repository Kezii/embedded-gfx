use camera::Camera;
use embedded_graphics_core::pixelcolor::Rgb565;
use embedded_graphics_core::pixelcolor::RgbColor;
use mesh::K3dMesh;
use mesh::RenderMode;
use nalgebra::Matrix4;
use nalgebra::Point2;
use nalgebra::Point3;
use nalgebra::Vector3;

pub mod camera;
pub mod draw;
pub mod framebuffer;
pub mod mesh;
pub mod perfcounter;

#[derive(Debug)]
pub enum DrawPrimitive {
    ColoredPoint(Point2<i32>, Rgb565),
    Line(Point2<i32>, Point2<i32>, Rgb565),
    ColoredTriangle(Point2<i32>, Point2<i32>, Point2<i32>, Rgb565),
}

pub struct K3dengine {
    pub camera: Camera,
    width: u16,
    height: u16,
}

impl K3dengine {
    pub fn new(width: u16, height: u16) -> K3dengine {
        K3dengine {
            camera: Camera::new(width as f32 / height as f32),
            width,
            height,
        }
    }

    fn transform_point(&self, point: &[f32; 3], model_matrix: Matrix4<f32>) -> Option<Point3<i32>> {
        let point = nalgebra::Vector4::new(point[0], point[1], point[2], 1.0);
        let point = model_matrix * point;

        if point.w < 0.0 {
            return None;
        }
        if point.z < self.camera.near || point.z > self.camera.far {
            return None;
        }

        let point = Point3::from_homogeneous(point)?;

        Some(Point3::new(
            ((1.0 + point.x) * 0.5 * self.width as f32) as i32,
            ((1.0 - point.y) * 0.5 * self.height as f32) as i32,
            (point.z * (self.camera.far - self.camera.near) + self.camera.near) as i32,
        ))
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

            let transform_matrix = self.camera.vp_matrix * mesh.model_matrix;
            let screen_space_points = mesh
                .geometry
                .vertices
                .iter()
                .filter_map(|v| self.transform_point(v, transform_matrix));

            match mesh.render_mode {
                RenderMode::Points
                    if mesh.geometry.colors.len() == mesh.geometry.vertices.len() =>
                {
                    for (point, color) in screen_space_points.zip(mesh.geometry.colors) {
                        callback(DrawPrimitive::ColoredPoint(point.xy(), *color));
                    }
                }

                RenderMode::Points => {
                    for point in screen_space_points {
                        callback(DrawPrimitive::ColoredPoint(point.xy(), mesh.color));
                    }
                }

                RenderMode::Lines if !mesh.geometry.lines.is_empty() => {
                    for line in mesh.geometry.lines {
                        let p1 = self
                            .transform_point(&mesh.geometry.vertices[line[0]], transform_matrix);
                        let p2 = self
                            .transform_point(&mesh.geometry.vertices[line[1]], transform_matrix);

                        if let (Some(p1), Some(p2)) = (p1, p2) {
                            callback(DrawPrimitive::Line(p1.xy(), p2.xy(), mesh.color));
                        }
                    }
                }

                RenderMode::Lines if !mesh.geometry.faces.is_empty() => {
                    for face in mesh.geometry.faces {
                        let p1 = self
                            .transform_point(&mesh.geometry.vertices[face[0]], transform_matrix);
                        let p2 = self
                            .transform_point(&mesh.geometry.vertices[face[1]], transform_matrix);
                        let p3 = self
                            .transform_point(&mesh.geometry.vertices[face[2]], transform_matrix);

                        if let (Some(p1), Some(p2), Some(p3)) = (p1, p2, p3) {
                            callback(DrawPrimitive::Line(p1.xy(), p2.xy(), mesh.color));
                            callback(DrawPrimitive::Line(p2.xy(), p3.xy(), mesh.color));
                            callback(DrawPrimitive::Line(p3.xy(), p1.xy(), mesh.color));
                        }
                    }
                }

                RenderMode::Lines => {}

                RenderMode::SolidLightDir(direction) => {
                    for (face, normal) in mesh.geometry.faces.iter().zip(mesh.geometry.normals) {
                        //Backface culling
                        let normal = Vector3::new(normal[0], normal[1], normal[2]);

                        let transformed_normal = mesh.model_matrix.transform_vector(&normal);

                        if self.camera.get_direction().dot(&transformed_normal) < 0.0 {
                            continue;
                        }

                        let p1 = self
                            .transform_point(&mesh.geometry.vertices[face[0]], transform_matrix);
                        let p2 = self
                            .transform_point(&mesh.geometry.vertices[face[1]], transform_matrix);
                        let p3 = self
                            .transform_point(&mesh.geometry.vertices[face[2]], transform_matrix);

                        if let (Some(p1), Some(p2), Some(p3)) = (p1, p2, p3) {
                            let color_as_float = Vector3::new(
                                mesh.color.r() as f32 / 32.0,
                                mesh.color.g() as f32 / 64.0,
                                mesh.color.b() as f32 / 32.0,
                            );

                            let mut final_color = Vector3::new(0.0f32, 0.0, 0.0);

                            let intensity = transformed_normal.dot(&direction);

                            let intensity = intensity.max(0.0);

                            final_color += color_as_float * intensity + color_as_float * 0.4;

                            let final_color = Vector3::new(
                                final_color.x.min(1.0).max(0.0),
                                final_color.y.min(1.0).max(0.0),
                                final_color.z.min(1.0).max(0.0),
                            );

                            let color = Rgb565::new(
                                (final_color.x * 31.0) as u8,
                                (final_color.y * 63.0) as u8,
                                (final_color.z * 31.0) as u8,
                            );
                            callback(DrawPrimitive::ColoredTriangle(
                                p1.xy(),
                                p2.xy(),
                                p3.xy(),
                                color,
                            ));
                        }
                    }
                }

                RenderMode::Solid => {
                    if mesh.geometry.normals.is_empty() {
                        for face in mesh.geometry.faces.iter() {
                            let p1 = self.transform_point(
                                &mesh.geometry.vertices[face[0]],
                                transform_matrix,
                            );
                            let p2 = self.transform_point(
                                &mesh.geometry.vertices[face[1]],
                                transform_matrix,
                            );
                            let p3 = self.transform_point(
                                &mesh.geometry.vertices[face[2]],
                                transform_matrix,
                            );

                            if let (Some(p1), Some(p2), Some(p3)) = (p1, p2, p3) {
                                callback(DrawPrimitive::ColoredTriangle(
                                    p1.xy(),
                                    p2.xy(),
                                    p3.xy(),
                                    mesh.color,
                                ));
                            }
                        }
                    } else {
                        for (face, normal) in mesh.geometry.faces.iter().zip(mesh.geometry.normals)
                        {
                            //Backface culling
                            let normal = Vector3::new(normal[0], normal[1], normal[2]);

                            let transformed_normal = mesh.model_matrix.transform_vector(&normal);

                            if self.camera.get_direction().dot(&transformed_normal) < 0.0 {
                                continue;
                            }

                            let p1 = self.transform_point(
                                &mesh.geometry.vertices[face[0]],
                                transform_matrix,
                            );
                            let p2 = self.transform_point(
                                &mesh.geometry.vertices[face[1]],
                                transform_matrix,
                            );
                            let p3 = self.transform_point(
                                &mesh.geometry.vertices[face[2]],
                                transform_matrix,
                            );

                            if let (Some(p1), Some(p2), Some(p3)) = (p1, p2, p3) {
                                callback(DrawPrimitive::ColoredTriangle(
                                    p1.xy(),
                                    p2.xy(),
                                    p3.xy(),
                                    mesh.color,
                                ));
                            }
                        }
                    }
                }
            }
        }
    }
}
