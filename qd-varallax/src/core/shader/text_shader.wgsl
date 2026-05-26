struct Projection {
	matrix: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> projection: Projection;

@group(1) @binding(0) var t_diffuse: binding_array<texture_2d<f32>>;
@group(1) @binding(1) var msdfSampler: sampler;

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
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
	let msd = textureSample(t_diffuse[in.texture_index], msdfSampler, in.tex_coords).rgb;
	let sd = median(msd.r, msd.g, msd.b);
	let px_range = 10.0 * (sd - 0.5);
	let opacity = clamp(px_range / fwidth(px_range) + 0.5, 0.0, 1.0);
	return vec4f(in.color.rgb, in.color.a * opacity);
}

fn median(r: f32, g: f32, b: f32) -> f32 {
    return max(min(r, g), min(max(r, g), b));
}