use crate::types::{color::{VxColor, VxColorU8}, geometry::{VxSize, VxVec2, VxVec3}, render_commands::VxRenderModuleId};


pub trait VxAbstractVertex
where Self:
	bytemuck::Zeroable + bytemuck::Pod
	+ Clone + Copy
{
	const MODULE_ID: VxRenderModuleId;
	fn vertex_attributes() -> &'static [wgpu::VertexAttribute];
	fn vertex_buffer_layout() -> wgpu::VertexBufferLayout<'static> {
		wgpu::VertexBufferLayout {
			array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
			step_mode: wgpu::VertexStepMode::Vertex,
			attributes: Self::vertex_attributes(),
		}
	}

	fn pos(&self) -> VxVec3;
	fn set_pos(&mut self, pos: VxVec3);

	#[inline]
	fn x(&self) -> f32 { self.pos().x() }
	#[inline]
	fn y(&self) -> f32 { self.pos().y() }
	#[inline]
	fn z(&self) -> f32 { self.pos().z() }
	#[inline]
	fn to_vec2(&self) -> VxVec2 { self.pos().as_vec2() }
	#[inline]
	fn set_x(&mut self, x: f32) { self.pos().set_x(x); }
	#[inline]
	fn set_y(&mut self, y: f32) { self.pos().set_y(y); }
	#[inline]
	fn set_z(&mut self, z: f32) { self.pos().set_z(z); }
	#[inline]
	fn set_pos_vec2(&mut self, pos: VxVec2) {
		let pos_v2 = self.pos().with_x(pos.x()).with_y(pos.y());
		self.set_pos(VxVec3::new(pos_v2.x(), pos_v2.y(), self.pos().z()));
	}
}


#[repr(C)]
#[derive(Clone, Copy, PartialEq, Debug, bytemuck::Zeroable, bytemuck::Pod)]
pub struct VxVertex {
	pos: VxVec3,
	color: VxColorU8,
}

impl VxAbstractVertex for VxVertex {
	const MODULE_ID: VxRenderModuleId = VxRenderModuleId::VERTEX;
	fn vertex_attributes() -> &'static [wgpu::VertexAttribute] {
		&Self::VERTEX_ATTRIBUTES
	}
	#[inline]
	fn pos(&self) -> VxVec3 {
		self.pos
	}
	#[inline]
	fn set_pos(&mut self, pos: VxVec3) {
		self.pos = pos;
	}
}

impl VxVertex {
	pub const VERTEX_ATTRIBUTES: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![
		0 => Float32x3,
		1 => Unorm8x4,
	];
	#[inline]
	pub const fn new(pos: VxVec3, color: VxColor) -> Self {
		Self { pos, color: color.to_vxcoloru8() }
	}
}

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Debug, bytemuck::Zeroable, bytemuck::Pod)]
pub struct VxSdfVertex {
	pos: VxVec3,
	color: VxColorU8,
	radius: f32,
	uv: VxVec2,
	size: VxSize,
	outline_color: VxColorU8,
	outline_width: f32,
	blur_radius: f32,
}

impl VxAbstractVertex for VxSdfVertex {
	const MODULE_ID: VxRenderModuleId = VxRenderModuleId::SDF;
	fn vertex_attributes() -> &'static [wgpu::VertexAttribute] {
		&Self::VERTEX_ATTRIBUTES
	}
	#[inline]
	fn pos(&self) -> VxVec3 {
		self.pos
	}
	#[inline]
	fn set_pos(&mut self, pos: VxVec3) {
		self.pos = pos;
	}
}

impl VxSdfVertex {
	pub const VERTEX_ATTRIBUTES: [wgpu::VertexAttribute; 8] = wgpu::vertex_attr_array![
		0 => Float32x3,
		1 => Unorm8x4,
		2 => Float32,
		3 => Float32x2,
		4 => Float32x2,
		5 => Unorm8x4,
		6 => Float32,
		7 => Float32,
	];
	#[inline]
	pub fn new(
		pos: VxVec3,
		color: VxColor,
		radius: f32,
		uv: VxVec2,
		size: VxSize,
		outline_color: VxColor,
		outline_width: f32,
		blur_radius: f32,
	) -> Self {
		Self {
			pos,
			color: color.to_vxcoloru8(),
			radius, uv,size,
			outline_color: outline_color.to_vxcoloru8(),
			outline_width, blur_radius
		}
	}
}

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct VxTextureVertex {
	pos: VxVec3,
	color: VxColorU8,
	texture_coordinate: VxVec2,
	texture_index: i32,
}

impl VxAbstractVertex for VxTextureVertex {
	const MODULE_ID: VxRenderModuleId = VxRenderModuleId::TEXTURE;
	fn vertex_attributes() -> &'static [wgpu::VertexAttribute] {
		&Self::VERTEX_ATTRIBUTES
	}
	#[inline]
	fn pos(&self) -> VxVec3 {
		self.pos
	}
	#[inline]
	fn set_pos(&mut self, pos: VxVec3) {
		self.pos = pos;
	}
}

impl VxTextureVertex {
	pub const VERTEX_ATTRIBUTES: [wgpu::VertexAttribute; 4] = wgpu::vertex_attr_array![
		0 => Float32x3,
		1 => Unorm8x4,
		2 => Float32x2,
		3 => Sint32,
	];
	#[inline]
	pub fn new(pos: VxVec3, color: VxColor, texture_coordinate: VxVec2, texture_index: i32) -> Self {
		Self {
			pos, color: color.to_vxcoloru8(), texture_coordinate, texture_index
		}
	}
}

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct VxGlyphVertex {
	pos: VxVec3,
	color: VxColorU8,
	texture_coordinate: VxVec2,
	texture_index: i32,
	outline_color: VxColorU8,
	outline_width: f32,
	blur_radius: f32,
}

impl VxAbstractVertex for VxGlyphVertex {
	const MODULE_ID: VxRenderModuleId = VxRenderModuleId::GLYPH;
	fn vertex_attributes() -> &'static [wgpu::VertexAttribute] {
		&Self::VERTEX_ATTRIBUTES
	}
	#[inline]
	fn pos(&self) -> VxVec3 {
		self.pos
	}
	#[inline]
	fn set_pos(&mut self, pos: VxVec3) {
		self.pos = pos;
	}
}

impl VxGlyphVertex {
	pub const VERTEX_ATTRIBUTES: [wgpu::VertexAttribute; 7] = wgpu::vertex_attr_array![
		0 => Float32x3,
		1 => Unorm8x4,
		2 => Float32x2,
		3 => Sint32,
		4 => Unorm8x4,
		5 => Float32,
		6 => Float32,
	];
	#[inline]
	pub fn new(
		pos: VxVec3,
		color: VxColor,
		texture_coordinate: VxVec2,
		texture_index: i32,
		outline_color: VxColor,
		outline_width: f32,
		blur_radius: f32
	) -> Self {
		Self {
			pos, color: color.to_vxcoloru8(), texture_coordinate, texture_index,
			outline_color: outline_color.to_vxcoloru8(), outline_width, blur_radius
		}
	}
}