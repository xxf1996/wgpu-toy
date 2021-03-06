use std::mem;

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
  pub position: [f32; 3],
  pub color: [f32; 3],
  pub uv: [f32; 2],
}

impl Vertex {
  pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
    wgpu::VertexBufferLayout {
      array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
      step_mode: wgpu::VertexStepMode::Vertex,
      attributes: &[
        wgpu::VertexAttribute {
          offset: 0,
          shader_location: 0,
          format: wgpu::VertexFormat::Float32x3,
        }, // position
        wgpu::VertexAttribute {
          offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
          shader_location: 1,
          format: wgpu::VertexFormat::Float32x3,
        }, // color
        wgpu::VertexAttribute {
          offset: mem::size_of::<[f32; 6]>() as wgpu::BufferAddress,
          shader_location: 2,
          format: wgpu::VertexFormat::Float32x2,
        }, // uv
      ]
    }
  }
}

/// 实例数据；
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceData {
  /// 实例自身model_matrix
  pub model_matrix: [[f32; 4]; 4],
}

impl InstanceData {
  pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
    // 将4x4的矩阵拆分为4个行向量；webGPU不支持传送4x4的attribute变量？
    wgpu::VertexBufferLayout {
      array_stride: mem::size_of::<InstanceData>() as wgpu::BufferAddress,
      step_mode: wgpu::VertexStepMode::Instance,
      attributes: &[
        wgpu::VertexAttribute {
          offset: 0,
          shader_location: 3,
          format: wgpu::VertexFormat::Float32x4,
        },
        wgpu::VertexAttribute {
          offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
          shader_location: 4,
          format: wgpu::VertexFormat::Float32x4,
        },
        wgpu::VertexAttribute {
          offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
          shader_location: 5,
          format: wgpu::VertexFormat::Float32x4,
        },
        wgpu::VertexAttribute {
          offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
          shader_location: 6,
          format: wgpu::VertexFormat::Float32x4,
        }
      ]
    }
  }
}

/// 物体实例所需信息，实际上就是构成model matrix
pub struct Instance {
  /// 物体中心位置
  pub center: cgmath::Vector3<f32>,
  /// 物体旋转四元量
  pub rotation: cgmath::Quaternion<f32>
}

impl Instance {
  /// 获取实例数据；
  pub fn get_data(&self) -> InstanceData {
    let model_matrix = cgmath::Matrix4::from_translation(self.center) * cgmath::Matrix4::from(self.rotation);
    InstanceData {
      model_matrix: model_matrix.into()
    }
  }
}

pub struct BuferInfo {
  pub vertices: Vec<Vertex>,
  pub indices: Vec<u16>,
}

/// 获取圆形的顶点数据和相应的顶点索引数据，用于构建顶点缓冲和索引缓冲；
/// 
/// `segment`为角度切割份数；`radius`为半径；`aspect`为屏幕宽高比；
pub fn get_circle(segment: u8, radius: f32, aspect: f32) -> BuferInfo {
  let mut vertices: Vec<Vertex> = vec![];
  let mut indices: Vec<u16> = vec![];
  let color: [f32; 3] = [0.3, 0.5, 0.8];
  let per_angle: f32 = std::f32::consts::PI * 2.0 / (segment as f32);
  for idx in 0..=segment-1 {
    let angle = idx as f32 * per_angle;
    let x = radius * angle.cos() / aspect; // 除以宽高比，校正比例
    let y = radius * angle.sin();
    vertices.push(Vertex {
      position: [x, y, 1.0],
      color,
      uv: [x, y]
    });
    indices.push(0);
    indices.push(idx as u16);
    indices.push(((idx + 1) % segment) as u16); // 逆时针索引
  }
  BuferInfo {
    vertices,
    indices
  }
}
