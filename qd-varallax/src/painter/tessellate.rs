use crate::types::{color::VxColor, geometry::{VxRect, VxRectR}, vertex::{VxSdfVertex, VxTexVertex, VxVertex}};


pub fn tessellate_texture(rect: VxRect, color: VxColor, uv: VxRect, tex_index: i32) -> ([VxTexVertex; 4], [u16; 6]) {
	let x = rect.x();
	let y = rect.y();
	let w = rect.width();
	let h = rect.height();

	(
		[
			VxTexVertex::new([x, y, 0.0], color, uv.left_top().to_array(), tex_index),
			VxTexVertex::new([x + w, y, 0.0], color, uv.right_top().to_array(), tex_index),
			
			VxTexVertex::new([x + w, y + h, 0.0], color, uv.right_bottom().to_array(), tex_index),
			VxTexVertex::new([x, y + h, 0.0], color, uv.left_bottom().to_array(), tex_index),
		],
		[0, 1, 2, 2, 3, 0]
	)
}

pub fn tessellate_rect(rect: &VxRect, color: &VxColor) -> ([VxVertex; 4], [u16; 6]) {
	let x = rect.x();
	let y = rect.y();
	let w = rect.width();
	let h = rect.height();

	(
		[
			VxVertex::new([x, y, 0.0], *color),
			VxVertex::new([x + w, y, 0.0], *color),
			
			VxVertex::new([x + w, y + h, 0.0], *color),
			VxVertex::new([x, y + h, 0.0], *color),
		],
		[0, 1, 2, 2, 3, 0]
	)
}

pub fn tessellate_sdf_rect(rectr: &VxRectR, color: &VxColor) -> ([VxSdfVertex; 4], [u16; 6]) {
	let rect = rectr.rect();
	let x = rect.x();
	let y = rect.y();
	let w = rect.width();
	let h = rect.height();
	let radius = rectr.corner_radius();

	(
		[
			VxSdfVertex::new([x, y, 0.0], *color, radius, [0.0, 0.0], rect.size()),
			VxSdfVertex::new([x + w, y, 0.0], *color, radius, [1.0, 0.0], rect.size()),
			
			VxSdfVertex::new([x + w, y + h, 0.0], *color, radius, [1.0, 1.0], rect.size()),
			VxSdfVertex::new([x, y + h, 0.0], *color, radius, [0.0, 1.0], rect.size()),
		],
		[0, 1, 2, 2, 3, 0]
	)
}