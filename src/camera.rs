use std::f32::consts;

use nalgebra::{Isometry3, Perspective3, Point3, Vector3};

pub struct Camera {
    pub position: Point3<f32>,
    fov: f32,
    pub near: f32,
    pub far: f32,
    view_matrix: nalgebra::Matrix4<f32>,
    projection_matrix: nalgebra::Matrix4<f32>,
    pub vp_matrix: nalgebra::Matrix4<f32>,
    target: Point3<f32>,
    aspect_ratio: f32,
}

impl Camera {
    pub fn new(aspect_ratio: f32) -> Camera {
        let mut ret = Camera {
            position: Point3::new(0.0, 0.0, 0.0),
            fov: consts::PI / 2.0,
            view_matrix: nalgebra::Matrix4::identity(),
            projection_matrix: nalgebra::Matrix4::identity(),
            vp_matrix: nalgebra::Matrix4::identity(),
            target: Point3::new(0.0, 0.0, 0.0),
            aspect_ratio,
            near: 0.4,
            far: 20.0,
        };

        ret.update_projection();

        ret
    }

    pub fn set_position(&mut self, pos: Point3<f32>) {
        self.position = pos;

        self.update_view();
    }

    pub fn set_fovy(&mut self, fovy: f32) {
        self.fov = fovy;

        self.update_projection();
    }

    pub fn set_target(&mut self, target: Point3<f32>) {
        self.target = target;
        self.update_view();
    }

    pub fn get_direction(&self) -> Vector3<f32> {
        let transpose = self.view_matrix; //.transpose();

        Vector3::new(transpose[(2, 0)], transpose[(2, 1)], transpose[(2, 2)])
    }

    fn update_view(&mut self) {
        let view = Isometry3::look_at_rh(&self.position, &self.target, &Vector3::y());

        self.view_matrix = view.to_homogeneous();
        self.vp_matrix = self.projection_matrix * self.view_matrix;
    }

    fn update_projection(&mut self) {
        let projection = Perspective3::new(self.aspect_ratio, self.fov, self.near, self.far);
        self.projection_matrix = projection.to_homogeneous();
        self.vp_matrix = self.projection_matrix * self.view_matrix;
    }
}
