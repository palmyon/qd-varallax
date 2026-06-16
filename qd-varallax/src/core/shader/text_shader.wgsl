struct Projection {
    matrix: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> projection: Projection;

@group(1) @binding(0) var t_diffuse: binding_array<texture_2d<f32>>;
@group(1) @binding(1) var mtsdf_sampler: sampler;

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

fn median(r: f32, g: f32, b: f32) -> f32 {
    return max(min(r, g), min(max(r, g), b));
}


@fragment
fn fs_main(out: VertexOutput) -> @location(0) vec4f {
	let mtsdf = textureSample(t_diffuse[out.texture_index], mtsdf_sampler, out.tex_coords);
	let msdf_sd = median(mtsdf.r, mtsdf.g, mtsdf.b);

	let sd = clamp(msdf_sd - mtsdf.a, -0.5, 0.5) + mtsdf.a;
	let boost = pow((sd - 0.05) * 2.5, 15.0);
	let clamp_boost = clamp(boost, 0.0, 1.0);

    return vec4f(out.color.rgb, out.color.a * clamp_boost);
}