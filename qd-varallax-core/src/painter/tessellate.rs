use crate::types::{
	color::VxColor, geometry::{
		VxRect, VxSize, VxVec2, VxVec3
	}, style::VxSdfStyle, vertex::{
		VxSdfVertex, VxGlyphVertex, VxTextureVertex, VxVertex
	}
};


pub struct VxTessellator;
impl VxTessellator {
	pub fn tessellate_texture(rect: VxRect, color: VxColor, uv: VxRect, tex_index: i32) -> ([VxTextureVertex; 4], [u32; 6]) {
		let x = rect.x();
		let y = rect.y();
		let w = rect.width();
		let h = rect.height();
	
		(
			[
				VxTextureVertex::new(VxVec3::new(x, y, 0.0), color, uv.left_top(), tex_index),
				VxTextureVertex::new(VxVec3::new(x + w, y, 0.0), color, uv.right_top(), tex_index),
				
				VxTextureVertex::new(VxVec3::new(x + w, y + h, 0.0), color, uv.right_bottom(), tex_index),
				VxTextureVertex::new(VxVec3::new(x, y + h, 0.0), color, uv.left_bottom(), tex_index),
			],
			[0, 1, 2, 2, 3, 0]
		)
	}
	
	pub fn tessellate_rect(rect: VxRect, color: VxColor) -> ([VxVertex; 4], [u32; 6]) {
		let x = rect.x();
		let y = rect.y();
		let w = rect.width();
		let h = rect.height();
	
		(
			[
				VxVertex::new(VxVec3::new(x, y, 0.0), color),
				VxVertex::new(VxVec3::new(x + w, y, 0.0), color),
				
				VxVertex::new(VxVec3::new(x + w, y + h, 0.0), color),
				VxVertex::new(VxVec3::new(x, y + h, 0.0), color),
			],
			[0, 1, 2, 2, 3, 0]
		)
	}
	
	pub fn tessellate_sdf_rect(
		sdf_style: VxSdfStyle,
	) -> ([VxSdfVertex; 4], [u32; 6]) {
		let rect = sdf_style.rectr().rect();
		let color = sdf_style.color();
		let outline_color = sdf_style.outline_color();
		let outline_width = sdf_style.outline_width();
		let blur_radius = sdf_style.blur_radius();
		let margin = outline_width + blur_radius;
		let x = rect.x() - margin;
		let y = rect.y() - margin;
		let w = rect.width() + (margin * 2.0);
		let h = rect.height() + (margin * 2.0);
		let radius = sdf_style.rectr().corner_radius();
	
		let size = VxSize::new(w, h);
		
		(
			[
				VxSdfVertex::new(
					VxVec3::new(x, y, 0.0), color, radius,
					VxVec2::new(0.0, 0.0), size,
					outline_color, outline_width, blur_radius,
	
				),
				VxSdfVertex::new(
					VxVec3::new(x + w, y, 0.0), color, radius,
					VxVec2::new(1.0, 0.0), size,
					outline_color, outline_width, blur_radius,
				),
				
				VxSdfVertex::new(
					VxVec3::new(x + w, y + h, 0.0), color, radius,
					VxVec2::new(1.0, 1.0), size,
					outline_color, outline_width, blur_radius,
				),
				VxSdfVertex::new(
					VxVec3::new(x, y + h, 0.0), color, radius,
					VxVec2::new(0.0, 1.0), size,
					outline_color, outline_width, blur_radius,
				),
			],
			[0, 1, 2, 2, 3, 0]
		)
	}
	
	pub fn tessellate_text(
		rect: VxRect,
		color: VxColor,
		uv: VxRect,
		tex_index: i32,
		mut outline_color: VxColor,
		outline_width: f32,
		blur_radius: f32,
	) -> ([VxGlyphVertex; 4], [u32; 6]) {
		let x = rect.x();
		let y = rect.y();
		let w = rect.width();
		let h = rect.height();
	
		// シェーダーでは何とかできなかったのでごまかし
		if outline_width == 0.0 && blur_radius == 0.0 {
			outline_color = color;
		}
	
		(
			[
				VxGlyphVertex::new(
					VxVec3::new(x, y, 0.0), color, uv.left_top(),
					tex_index, outline_color, outline_width,
					blur_radius,
				),
				VxGlyphVertex::new(
					VxVec3::new(x + w, y, 0.0), color, uv.right_top(),
					tex_index, outline_color, outline_width,
					blur_radius,
				),
				
				VxGlyphVertex::new(
					VxVec3::new(x + w, y + h, 0.0), color, uv.right_bottom(),
					tex_index, outline_color, outline_width,
					blur_radius,
				),
				VxGlyphVertex::new(
					VxVec3::new(x, y + h, 0.0), color, uv.left_bottom(),
					tex_index, outline_color, outline_width,
					blur_radius,
				),
			],
			[0, 1, 2, 2, 3, 0]
		)
	}
}