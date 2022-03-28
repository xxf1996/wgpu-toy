use cgmath::SquareMatrix;
use wgpu::util::DeviceExt;
use winit::{
  event::*
};

/// 相机
pub struct Camera {
  pub eye: cgmath::Point3<f32>,
  pub lookat: cgmath::Point3<f32>,
  pub up: cgmath::Vector3<f32>,
  pub aspect: f32,
  pub fov: f32,
  pub near: f32,
  pub far: f32,
}

/// 相机相关uniform变量
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
  /// 视图投影矩阵
  view_projection: [[f32; 4]; 4] // 4x4矩阵
}

pub struct CameraInfo {
  pub uniform: CameraUniform,
  pub buffer: wgpu::Buffer,
  pub group: wgpu::BindGroup,
  pub layout: wgpu::BindGroupLayout
}

pub struct CameraController<'a> {
  camera: &'a mut Camera,
  info: &'a mut CameraInfo,
}

/// 用于将openGL NDC（标准化设备坐标）中的z从[-1, 1]映射到[0, 1]（Vulkan和Metal）
#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
  1.0, 0.0, 0.0, 0.0,
  0.0, 1.0, 0.0, 0.0,
  0.0, 0.0, 0.5, 0.0,
  0.0, 0.0, 0.5, 1.0,
);

impl Camera {
  fn get_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
    let view = cgmath::Matrix4::look_at_rh(self.eye, self.lookat, self.up); // 视图变换矩阵
    let projection = cgmath::perspective(cgmath::Deg(self.fov), self.aspect, self.near, self.far); // 透视投影矩阵

    OPENGL_TO_WGPU_MATRIX * projection * view
  }

  /// 将相机位置围绕`lookat`旋转
  pub fn rotate(&mut self, delta_angle: f32) {
    let rotate_matrix = cgmath::Matrix4::from_axis_angle(cgmath::Vector3::unit_y(), cgmath::Deg(delta_angle)); // 获取旋转矩阵
    let next_pos = rotate_matrix * self.eye.to_homogeneous();
    self.eye = cgmath::Point3::from_homogeneous(next_pos);
  }

  /// 将相机位置沿着视线方向进行移动
  pub fn move_line(&mut self, delta_dist: f32) {
    let move_dir = cgmath::InnerSpace::normalize(self.eye - self.lookat);
    let delta = move_dir * delta_dist;
    self.eye += delta;
  }
}

impl CameraUniform {
  fn new() -> Self {
    Self {
      view_projection: cgmath::Matrix4::identity().into()
    }
  }

  fn update_matrix(&mut self, camera: &Camera) {
    self.view_projection = camera.get_view_projection_matrix().into();
  }
}

impl CameraInfo {
  fn get_info(camera: &Camera, device: &wgpu::Device) -> Self {
    let mut uniform = CameraUniform::new();
    uniform.update_matrix(camera);
    let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: Some("Camera buffer"),
      contents: bytemuck::cast_slice(&[uniform]),
      usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST
    });
    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      label: Some("camera bind group layout"),
      entries: &[
        wgpu::BindGroupLayoutEntry {
          binding: 0,
          visibility: wgpu::ShaderStages::VERTEX,
          ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None
          },
          count: None
        } // 绑定到group的索引0位置，且只在顶点着色器可见
      ],
    });
    let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
      label: Some("camera bind group"),
      layout: &layout,
      entries: &[
        wgpu::BindGroupEntry {
          binding: 0,
          resource: buffer.as_entire_binding() // buffer数据
        }
      ]
    });
    Self {
      uniform,
      buffer,
      layout,
      group,
    }
  }

  pub fn update_info(& mut self, camera: &Camera, device: &wgpu::Device) {
    let info = CameraInfo::get_info(camera, device);
    self.buffer = info.buffer;
    self.group = info.group;
    self.layout = info.layout;
    self.uniform = info.uniform;
  }

  pub fn new(camera: &Camera, device: &wgpu::Device) -> Self {
    CameraInfo::get_info(camera, device)
  }
}

impl<'a> CameraController<'a> {
  pub fn new(camera: &'a mut Camera, info: &'a mut CameraInfo) -> Self {
    Self {
      camera,
      info,
    }
  }

  pub fn watch_event(&self, event: &WindowEvent) -> bool {
    match event {
      WindowEvent::KeyboardInput {
        input: KeyboardInput {
          state: ElementState::Pressed,
          virtual_keycode: Some(VirtualKeyCode::Left), // 左方向
          ..
        },
        ..
      } => {
        true
      },
      WindowEvent::KeyboardInput {
        input: KeyboardInput {
          state: ElementState::Pressed,
          virtual_keycode: Some(VirtualKeyCode::Right), // 右方向
          ..
        },
        ..
      } => {
        true
      },
      WindowEvent::MouseWheel { // 鼠标滚动
        delta,
        phase,
        ..
      } => {
        println!("{:#?}", delta);
        println!("{:#?}", phase);
        true
      },
      _ => false
    }
  }
}
