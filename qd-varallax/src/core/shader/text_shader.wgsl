struct Projection {
    matrix: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> projection: Projection;

@group(1) @binding(0)
var t_diffuse: binding_array<texture_2d<f32>>;
@group(1) @binding(1)
var mtsdf_sampler: sampler;

struct VertexInput {
    @location(0) pos: vec3f,
    @location(1) color: vec4f,
    @location(2) tex_coords: vec2f,
    @location(3) texture_index: i32,
	@location(4) outline_color: vec4f,
	@location(5) outline_width: f32,
	@location(6) blur_radius: f32,
}

struct VertexOutput {
    @builtin(position) clip_pos: vec4f,
    @location(0) color: vec4f,
    @location(1) tex_coords: vec2f,
    @location(2) @interpolate(flat) texture_index: i32,
	@location(3) outline_color: vec4f,
	@location(4) outline_width: f32,
	@location(5) blur_radius: f32,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_pos = projection.matrix * vec4f(in.pos, 1.0);
    out.color = in.color;
	out.tex_coords = in.tex_coords;
    out.texture_index = in.texture_index;
	out.outline_color = in.outline_color;
	out.outline_width = in.outline_width;
	out.blur_radius = in.blur_radius;
    return out;
}

fn median(r: f32, g: f32, b: f32) -> f32 {
    return max(min(r, g), min(max(r, g), b));
}

@fragment
fn fs_main(out: VertexOutput) -> @location(0) vec4f {
	let mtsdf = textureSample(t_diffuse[out.texture_index], mtsdf_sampler, out.tex_coords);
	let msdf_sd = median(mtsdf.r, mtsdf.g, mtsdf.b);

	let diff = abs(msdf_sd - mtsdf.a);
	let weight = clamp(diff * 8.0, 0.0, 1.0);
	let sd = mix(msdf_sd, mtsdf.a, weight);

	let boost = pow(clamp(sd - 0.05, 0.0, 1.0) * 2.35, 16.0);
	let body_alpha = clamp(boost, 0.0, 1.0) * out.color.a;

	let outline_power = 16.0 / max(1.0 + out.outline_width, 0.05);
	let outline_boost = pow(clamp(sd - 0.05, 0.0, 1.0) * 2.35, outline_power);
	let outline_shape = clamp(outline_boost, 0.0, 1.0) * step(1e-4, out.outline_width);

	let shape_fw = fwidth(outline_shape);
	let threshold = 0.1;
	let blur_width = max(shape_fw, out.blur_radius * 0.05) * step(1e-4, out.blur_radius);
	let total_alpha = smoothstep(threshold - blur_width, threshold + blur_width, outline_shape);

	let outline_alpha = clamp(total_alpha - body_alpha, 0.0, 1.0);

	let final_color = mix(
		vec4f(out.outline_color.rgb, out.outline_color.a * outline_alpha),
		vec4f(out.color.rgb, body_alpha),
		body_alpha,
	);

	return final_color;
}