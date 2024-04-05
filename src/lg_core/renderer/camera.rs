use nalgebra_glm as glm;

use crate::{lg_core::{event::{LgEvent, MouseButton, MouseEvent, VKeyCode}, input::LgInput}, utils::tools::to_radians};

#[derive(Default, Debug, Clone, Copy)]
pub struct Camera {
    projection_matrix: glm::Mat4,
    view_matrix: glm::Mat4,
    position: glm::Vec3,
    focal_point: glm::Vec3,

    initial_mouse_position: glm::Vec2,

    pub distance: f32,
    pitch: f32,
    yaw: f32,

    viewport_width: f32,
    viewport_height: f32,
    aspect_ratio: f32,
    fov: f32,
    near_clip: f32,
    far_clip: f32,
}
impl Camera {
    // Public
    pub fn new(fov: f32, viewport_width: f32, viewport_height: f32, near_clip: f32, far_clip: f32) -> Self {
        let mut result = Self {
            projection_matrix: glm::perspective(
                viewport_width / viewport_height,
                fov,
                near_clip,
                far_clip
            ),
            view_matrix: glm::Mat4::identity(),
            position: glm::vec3(0.0, 0.0, 0.0),
            focal_point: glm::vec3(0.0, 0.0, 0.0),
            initial_mouse_position: glm::vec2(0.0, 0.0),                                    
            distance: 10.0,
            pitch: 0.0,
            yaw: 0.0,
            viewport_width,
            viewport_height,
            aspect_ratio: viewport_width / viewport_height,
            fov,
            near_clip,
            far_clip
        };
        
        result.update_view();
        result
    }
    
    pub fn set_viewport_size(&mut self, width: f32, height: f32) {
        self.viewport_width = width;
        self.viewport_height = height;
        
        self.update_projection();
    }
    
    pub const fn get_position(&self) -> &glm::Vec3 {
        &self.position
    }
    pub const fn get_pitch(&self) -> f32 {
        self.pitch
    }
    pub const fn get_yaw(&self) -> f32 {
        self.yaw
    }
    pub fn get_projection_matrix(&self) -> glm::Mat4 {
        let correction = glm::mat4(
            1.0,  0.0, 0.0, 0.0,
            0.0, -1.0, 0.0, 0.0,
            0.0,  0.0, 1.0, 0.0,
            0.0,  0.0, 0.0, 1.0,
        );

        self.projection_matrix * correction
    }
    pub fn get_view_matrix(&self) -> glm::Mat4 {
        self.view_matrix.clone()
    }
    pub fn get_view_projection(&self) -> glm::Mat4 {
        self.projection_matrix * self.view_matrix
    }
    
    pub fn get_up_direction(&self) -> glm::Vec3 {
        glm::quat_rotate_vec3(&self.get_orientation(), &glm::vec3(0.0, 1.0, 0.0))
    }
    pub fn get_right_direction(&self) -> glm::Vec3 {
        glm::quat_rotate_vec3(&self.get_orientation(), &glm::vec3(1.0, 0.0, 0.0))
    }
    pub fn get_forward_direction(&self) -> glm::Vec3 {
        glm::quat_rotate_vec3(&self.get_orientation(), &glm::vec3(0.0, 0.0, 1.0))
    }
    
    pub fn get_orientation(&self) -> glm::Quat {
        let pich_quat = glm::quat_angle_axis(
            self.pitch, 
            &glm::vec3(1.0, 0.0, 0.0)
        );
        let yaw_quat = glm::quat_angle_axis(
            self.yaw,
            &glm::vec3(0.0, 1.0, 0.0)
        );

        yaw_quat * pich_quat
    }
    
    pub fn on_update(&mut self, input: &LgInput) {
        optick::event!();
        if input.is_key_pressed(VKeyCode::LAlt)
        {
            let mouse = input.get_mouse_position();
            let delta = (mouse - self.initial_mouse_position) * 0.003;
            self.initial_mouse_position = *mouse;  

            if input.is_mouse_button_pressed(MouseButton::Right) {
                self.mouse_pan(&delta);
            }
            else if input.is_mouse_button_pressed(MouseButton::Left) {
                self.mouse_rotate(&delta);
            }
           
            self.update_view();
        }
    }
    
    // Private
    fn update_projection(&mut self) {
        self.aspect_ratio = self.viewport_width / self.viewport_height;
        self.projection_matrix = glm::perspective(
            self.aspect_ratio, 
            to_radians(self.fov as f64) as f32, 
            self.near_clip,
            self.far_clip
        );
    }
    fn update_view(&mut self) {
        self.position = self.calculate_position();
        
        let orientation = self.get_orientation();
        self.view_matrix = glm::translate(
            &glm::Mat4::identity(), 
            &-self.position
        )
        * glm::quat_to_mat4(&orientation);
        
        self.view_matrix = glm::inverse(&self.view_matrix);
    }

    fn mouse_pan(&mut self, delta: &glm::Vec2) {
        let (x_speed, y_speed) = self.pan_speed();
        
        self.focal_point += -self.get_right_direction() * delta.x * x_speed * self.distance;
        self.focal_point += self.get_up_direction() * delta.y * y_speed * self.distance;
    }
    fn mouse_rotate(&mut self, delta: &glm::Vec2) {
        let yaw_sign = if self.get_up_direction().y < 0.0 {
            -1.0
        } else { 1.0 };
        
        self.yaw += yaw_sign * delta.x * self.rotation_speed();
        self.pitch += delta.y * self.rotation_speed();
    }
    fn mouse_zoom(&mut self, delta: f32) {
        self.distance -= delta * self.zoom_speed();
        
        if self.distance < 1.0 {
            self.focal_point += self.get_forward_direction();
            self.distance = 1.0;
        }
    }
    
    pub fn on_event(&mut self, event: &LgEvent) {
        match event {
            LgEvent::MouseEvent(MouseEvent::ScrollEvent(scroll_event)) => {
                let delta = scroll_event.delta.1 * 0.1;
                self.mouse_zoom(delta);
            },
            _ => ()
        }
    }
    
    fn calculate_position(&self) -> glm::Vec3 {
        self.focal_point - self.get_forward_direction() * self.distance
    }
    
    fn pan_speed(&self) -> (f32, f32) {
        let x = (self.viewport_width / 1000.0).min(2.4);
        let x_factor = 0.0366 * (x * x) - 0.1778 * x + 0.3021;
        
        let y = (self.viewport_height / 1000.0).min(2.4);
        let y_factor = 0.0366 * (y * y) - 0.1778 * y + 0.3021;
            
        (x_factor, y_factor) 
    }
    const fn rotation_speed(&self) -> f32 {
        0.8
    }
    fn zoom_speed(&self) -> f32 {
        let distance = (self.distance * 0.2).max(0.0);
        
        let speed = (distance * distance).min(100.0);
        
        speed
    }
}