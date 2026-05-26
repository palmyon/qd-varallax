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
}

struct VertexOutput {
	@builtin(position) clip_pos: vec4f,
	@location(0) color: vec4f,
	@location(1) radius: f32,
	@location(2) uv: vec2f,
	@location(3) size: vec2f,
}

fn sd_rounded_box(p: vec2f, b: vec2f, r: f32) -> f32 {
	let q = abs(p) - b + r;
	return length(max(q, vec2f(0.0))) + min(max(q.x, q.y), 0.0) - r;
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
	var out: VertexOutput;
	out.clip_pos = projection.matrix * vec4<f32>(in.pos, 1.0);
	out.color = in.color;
	out.uv = in.uv;
	out.size = in.size;
	out.radius = in.radius;
	return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
	let p = (in.uv - 0.5) * in.size;

	let d = sd_rounded_box(p, in.size * 0.5, in.radius);

	let edge_soft = fwidth(d);
	let alpha = 1.0 - smoothstep(-edge_soft, edge_soft, d);
	return vec4f(in.color.rgb, in.color.a * alpha);
}