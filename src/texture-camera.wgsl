struct VertexInput {
  [[location(0)]] position: vec3<f32>;
  [[location(1)]] color: vec3<f32>;
  [[location(2)]] uv: vec2<f32>;
};

struct VertexOutput {
  [[builtin(position)]] clip_position: vec4<f32>;
  [[location(0)]] uv: vec2<f32>;
};

struct CameraUnifrom {
  view_projection: mat4x4<f32>;
};

[[group(1), binding(0)]]
var<uniform> camera: CameraUnifrom;

[[stage(vertex)]]
fn vs_main(inputData: VertexInput) -> VertexOutput {
  var outputData: VertexOutput;
  outputData.clip_position = camera.view_projection * vec4<f32>(inputData.position, 1.0);
  outputData.uv = inputData.uv;
  return outputData;
}

[[group(0), binding(0)]]
var texture_t: texture_2d<f32>;
[[group(0), binding(1)]]
var texture_s: sampler;

[[stage(fragment)]]
fn fs_main(inputData: VertexOutput) -> [[location(0)]] vec4<f32> {
  return textureSample(texture_t, texture_s, inputData.uv);
}
