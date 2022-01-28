use winit::{
  event::*,
  event_loop::{ControlFlow, EventLoop},
  window::WindowBuilder,
  window::Window,
};

struct State {
  surface: wgpu::Surface,
  device: wgpu::Device,
  queue: wgpu::Queue,
  config: wgpu::SurfaceConfiguration,
  size: winit::dpi::PhysicalSize<u32>,
}

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
    State {
      size,
      surface,
      device,
      queue,
      config
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
    false
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
      let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Render Pass"),
        color_attachments: &[wgpu::RenderPassColorAttachment {
          view: &view,
          resolve_target: None,
          ops: wgpu::Operations {
            load: wgpu::LoadOp::Clear(wgpu::Color {
              r: 0.2,
              g: 0.5,
              b: 0.8,
              a: 1.0,
            }),
            store: true,
          }
        }],
        depth_stencil_attachment: None
      });
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
    }
    _ => {}
  });

  todo!();
}
