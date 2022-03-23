mod shape;
mod texture;

use winit::{
  event::*,
  event_loop::{ControlFlow, EventLoop},
  window::WindowBuilder,
  window::Window,
};
use wgpu::util::DeviceExt;
use shape::{
  Vertex,
  get_circle
};

struct State {
  surface: wgpu::Surface,
  device: wgpu::Device,
  queue: wgpu::Queue,
  config: wgpu::SurfaceConfiguration,
  size: winit::dpi::PhysicalSize<u32>,
  background: wgpu::Color,
  render_pipeline: wgpu::RenderPipeline,
  render_pipeline2: wgpu::RenderPipeline,
  render_pipeline_default: bool,
  vertex_buffer: wgpu::Buffer,
  index_buffer: wgpu::Buffer,
  index_num: u32,
  texture_bind_group: wgpu::BindGroup
}

// const VERTICES: &[Vertex] = &[
//   Vertex { position: [0.0, 0.5, 0.0], color: [1.0, 0.0, 0.0] },
//   Vertex { position: [-0.5, -0.5, 0.0], color: [0.0, 1.0, 0.0] },
//   Vertex { position: [0.5, -0.5, 0.0], color: [0.0, 0.0, 1.0] },
// ];

impl State {
  pub async fn new(window: &Window) -> Self {
    let size = window.inner_size();
    let instance = wgpu::Instance::new(wgpu::Backends::all());
    let surface = unsafe { instance.create_surface(window) };
    let adpater = instance.request_adapter(&wgpu::RequestAdapterOptions {
      power_preference: wgpu::PowerPreference::default(),
      compatible_surface: Some(&surface),
      force_fallback_adapter: false,
    }).await.unwrap();
    let (device, queue) = adpater.request_device(&wgpu::DeviceDescriptor {
      features: wgpu::Features::empty(),
      limits: wgpu::Limits::default(),
      label: None,
    }, None).await.unwrap();
    let config = wgpu::SurfaceConfiguration {
      usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
      format: surface.get_preferred_format(&adpater).unwrap(),
      width: size.width,
      height: size.height,
      present_mode: wgpu::PresentMode::Fifo,
    };
    let background = wgpu::Color {
      r: 1.0,
      g: 0.0,
      b: 0.0,
      a: 1.0,
    };
    let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
      label: Some("Shader"),
      source: wgpu::ShaderSource::Wgsl(include_str!("texture.wgsl").into())
    });
    let shader2 = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
      label: Some("Shader"),
      source: wgpu::ShaderSource::Wgsl(include_str!("test2.wgsl").into())
    });
    let diffuse_texture = texture::Texture::default(&device, &queue).unwrap();
    let texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      label: Some("texture_bind_group_layout"),
      entries: &[
        wgpu::BindGroupLayoutEntry {
          binding: 0,
          visibility: wgpu::ShaderStages::FRAGMENT,
          ty: wgpu::BindingType::Texture {
            multisampled: false,
            view_dimension: wgpu::TextureViewDimension::D2,
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
          },
          count: None
        },
        wgpu::BindGroupLayoutEntry {
          binding: 1,
          visibility: wgpu::ShaderStages::FRAGMENT,
          ty: wgpu::BindingType::Sampler(
            // SamplerBindingType::Comparison is only for TextureSampleType::Depth
            // SamplerBindingType::Filtering if the sample_type of the texture is:
            //     TextureSampleType::Float { filterable: true }
            // Otherwise you'll get an error.
            wgpu::SamplerBindingType::Filtering,
          ),
          count: None,
        }
      ]
    });
    let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
      label: Some("texture_bind_group"),
      layout: &texture_bind_group_layout,
      entries: &[
        wgpu::BindGroupEntry {
          binding: 0,
          resource: wgpu::BindingResource::TextureView(&diffuse_texture.view)
        },
        wgpu::BindGroupEntry {
          binding: 1,
          resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler)
        }
      ]
    });
    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("Render Pipeline Layout"),
      bind_group_layouts: &[&texture_bind_group_layout],
      push_constant_ranges: &[]
    });
    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
      label: Some("Render Pipeline"),
      layout: Some(&render_pipeline_layout),
      vertex: wgpu::VertexState {
        module: &shader,
        entry_point: "vs_main",
        buffers: &[
          Vertex::desc()
        ]
      },
      fragment: Some(wgpu::FragmentState {
        module: &shader,
        entry_point: "fs_main",
        targets: &[wgpu::ColorTargetState {
          format: config.format,
          blend: Some(wgpu::BlendState::REPLACE),
          write_mask: wgpu::ColorWrites::ALL,
        }],
      }),
      primitive: wgpu::PrimitiveState { // 图元设置，如何生成三角
        topology: wgpu::PrimitiveTopology::TriangleList, // 每三个顶点为一个三角形
        strip_index_format: None,
        front_face: wgpu::FrontFace::Ccw, // 逆时针为正面
        cull_mode: Some(wgpu::Face::Back), // 背面隐藏
        polygon_mode: wgpu::PolygonMode::Fill, // 填充着色
        unclipped_depth: false,
        conservative: false
      },
      depth_stencil: None, // 深度模板缓存
      multisample: wgpu::MultisampleState {
        count: 1,
        mask: !0,
        alpha_to_coverage_enabled: false,
      },
      multiview: None
    });
    let render_pipeline2 = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
      label: Some("Render Pipeline"),
      layout: Some(&render_pipeline_layout),
      vertex: wgpu::VertexState {
        module: &shader2,
        entry_point: "vs_main",
        buffers: &[]
      },
      fragment: Some(wgpu::FragmentState {
        module: &shader2,
        entry_point: "fs_main",
        targets: &[wgpu::ColorTargetState {
          format: config.format,
          blend: Some(wgpu::BlendState::REPLACE),
          write_mask: wgpu::ColorWrites::ALL,
        }],
      }),
      primitive: wgpu::PrimitiveState { // 图元设置，如何生成三角
        topology: wgpu::PrimitiveTopology::TriangleList, // 每三个顶点为一个三角形
        strip_index_format: None,
        front_face: wgpu::FrontFace::Ccw, // 逆时针为正面
        cull_mode: Some(wgpu::Face::Back), // 背面隐藏
        polygon_mode: wgpu::PolygonMode::Fill, // 填充着色
        unclipped_depth: false,
        conservative: false
      },
      depth_stencil: None, // 深度模板缓存
      multisample: wgpu::MultisampleState {
        count: 1,
        mask: !0,
        alpha_to_coverage_enabled: false,
      },
      multiview: None
    });
    let buffer_info = get_circle(32, 0.8, size.width as f32 / size.height as f32);
    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: Some("Vertex Buffer"),
      usage: wgpu::BufferUsages::VERTEX,
      contents: bytemuck::cast_slice(&buffer_info.vertices),
    });
    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: Some("Index Buffer"),
      usage: wgpu::BufferUsages::INDEX,
      contents: bytemuck::cast_slice(&buffer_info.indices),
    });
    surface.configure(&device, &config); // 初始化时一定要进行配置
    State {
      size,
      surface,
      device,
      queue,
      config,
      background,
      render_pipeline,
      render_pipeline2,
      render_pipeline_default: true,
      vertex_buffer,
      index_buffer,
      index_num: buffer_info.indices.len() as u32,
      texture_bind_group
    }
  }

  pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
    if new_size.width > 0 && new_size.height > 0 {
      self.size = new_size;
      self.config.width = new_size.width;
      self.config.height = new_size.height;
      self.surface.configure(&self.device, &self.config);
    }
  }

  pub fn input(&mut self, event: &WindowEvent) -> bool {
    match event {
      WindowEvent::CursorMoved {
        device_id: _,
        position: winit::dpi::PhysicalPosition { x, y },
        modifiers: _,
      } => {
        self.background = wgpu::Color {
          r: x / (self.size.width as f64),
          g: y / (self.size.height as f64),
          b: 0.0,
          a: 1.0
        }; // 根据鼠标位置改变背景颜色
        true
      },
      WindowEvent::KeyboardInput {
        input: KeyboardInput {
          state: ElementState::Pressed,
          virtual_keycode: Some(VirtualKeyCode::Space),
          ..
        },
        ..
      } => {
        self.render_pipeline_default = !self.render_pipeline_default; // 切换渲染管线状态
        true
      },
      _ => false
    }
  }

  fn update(&mut self) {
    //
  }

  fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
    let output = self.surface.get_current_texture()?;
    let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
    let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
      label: Some("Render Encoder")
    });
    {
      let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Render Pass"),
        color_attachments: &[wgpu::RenderPassColorAttachment {
          view: &view,
          resolve_target: None,
          ops: wgpu::Operations {
            load: wgpu::LoadOp::Clear(self.background),
            store: true,
          }
        }],
        depth_stencil_attachment: None
      });
      render_pass.set_pipeline(if let true = self.render_pipeline_default {
        &self.render_pipeline
      } else {
        &self.render_pipeline2
      }); // 根据状态切换渲染管线
      render_pass.set_bind_group(0, &self.texture_bind_group, &[]); // 绑定到group中
      render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
      render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16); // 指定索引缓冲
      render_pass.draw_indexed(0..self.index_num, 0, 0..1); // 指定顶点数和实例数
    }

    self.queue.submit(std::iter::once(encoder.finish()));
    output.present();

    Ok(())
  }
}

fn main() {
  env_logger::init();
  let event_loop = EventLoop::new();
  let window = WindowBuilder::new().build(&event_loop).unwrap();
  let mut state = pollster::block_on(State::new(&window));

  event_loop.run(move |event, _, control_flow| match event {
    Event::WindowEvent {
      ref event,
      window_id,
    } if window_id == window.id() => if !state.input(event) {
      match event {
        WindowEvent::CloseRequested
        | WindowEvent::KeyboardInput {
          input:
            KeyboardInput {
              state: ElementState::Pressed,
              virtual_keycode: Some(VirtualKeyCode::Escape),
              ..
            },
          ..
        } => *control_flow = ControlFlow::Exit,
        WindowEvent::Resized(new_size) => {
          state.resize(*new_size);
        },
        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
          state.resize(**new_inner_size); // mut再解引用就是正常的变量了？
        },
        _ => {}
      }
    },
    Event::RedrawRequested(window_id) if window_id == window.id() => {
      state.update();
      match state.render() {
        Ok(_) => {},
        Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
        Err(wgpu::SurfaceError::OutOfMemory) => {
          *control_flow = ControlFlow::Exit;
        },
        Err(e) => eprintln!("{:?}", e)
      }
    },
    Event::MainEventsCleared => {
      window.request_redraw();
    },
    _ => {}
  });
}
