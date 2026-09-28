struct Projection {
	matrix: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> projection: Projection;

struct VertexInput {
	@location(0) pos: vec3f, // NDC座標
	@location(1) color: vec4f, // 色
	@location(2) radius: f32, // 丸める半径
	@location(3) uv: vec2f, // TexCoord
	@location(4) size: vec2f, // 四角形サイズ
	@location(5) outline_color: vec4f,
	@location(6) outline_width: f32,
	@location(7) blur_radius: f32,
}

struct VertexOutput {
	@builtin(position) clip_pos: vec4f,
	@location(0) color: vec4f,
	@location(1) radius: f32,
	@location(2) uv: vec2f,
	@location(3) size: vec2f,
	@location(4) outline_color: vec4f,
	@location(5) outline_width: f32,
	@location(6) blur_radius: f32,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
	var out: VertexOutput;
	out.clip_pos = projection.matrix * vec4<f32>(in.pos, 1.0);
	out.color = in.color;
	out.uv = in.uv;
	out.size = in.size;
	out.radius = in.radius;
	out.outline_color = in.outline_color;
	out.outline_width = in.outline_width;
	out.blur_radius = in.blur_radius;
	return out;
}

fn sd_rounded_box(p: vec2f, b: vec2f, r: f32) -> f32 {
	let q = abs(p) - b + r;
	return length(max(q, vec2f(0.0))) + min(max(q.x, q.y), 0.0) - r;
}


@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
	let p = (in.uv - 0.5) * in.size;

	let margin = in.outline_width + in.blur_radius;
	let original_size = in.size - vec2f(margin * 2.0);

	let d = sd_rounded_box(p, original_size * 0.5, in.radius);

	let edge_soft = fwidth(d);

	let blur_factor = max(in.blur_radius, edge_soft);

	let outline_divider = in.outline_width;
	let outline_alpha = 1.0 - smoothstep(
		outline_divider - blur_factor,
		outline_divider + blur_factor,
		d
	);

	let body_alpha = 1.0 - smoothstep(-edge_soft, edge_soft, d);
	
	let final_color = mix(
		vec4f(in.outline_color.rgb, in.outline_color.a * outline_alpha),
		in.color,
		body_alpha
	);

	return final_color;
}