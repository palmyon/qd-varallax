struct Projection {
	matrix: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> projection: Projection;

@group(1) @binding(0) var t_diffuse: binding_array<texture_2d<f32>>;
@group(1) @binding(1) var s_diffuse: sampler;

struct VertexInput {
    @location(0) pos: vec3<f32>,
    @location(1) color: vec4<f32>,
	@location(2) tex_coords: vec2<f32>,
	@location(3) texture_index: i32,
}

struct VertexOutput {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) color: vec4<f32>,
	@location(1) tex_coords: vec2<f32>,
	@location(2) @interpolate(flat) texture_index: i32,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_pos = projection.matrix * vec4<f32>(in.pos, 1.0);
    out.color = in.color;
	out.tex_coords = in.tex_coords;
	out.texture_index = in.texture_index;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
	let index  = u32(in.texture_index);
	let tex_color = textureSample(t_diffuse[index], s_diffuse, in.tex_coords);
	return tex_color * in.color;
}