struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
		@location(0) texcoords: vec2<f32>,
}

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
) -> VertexOutput {
    var out: VertexOutput;
    if in_vertex_index == u32(0) {
        out.clip_position = vec4<f32>(-1.0, 1.0, 0.0, 1.0);
        out.texcoords = vec2<f32>(0.0, 0.0);
    }
		if in_vertex_index == u32(1) {
        out.clip_position = vec4<f32>(-1.0, -3.0, 0.0, 1.0);
        out.texcoords = vec2<f32>(0.0, 2.0);
		}
    if in_vertex_index == u32(2) {
        out.clip_position = vec4<f32>(3.0, 1.0, 0.0, 1.0);
        out.texcoords = vec2<f32>(2.0, 0.0);
    }
    return out;
}

@group(0) @binding(0)
var tex: texture_2d<f32>;
@group(0) @binding(1)
var tex_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(tex, tex_sampler, in.texcoords);
}

