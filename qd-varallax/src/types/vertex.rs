use crate::types::{color::VxColor, geometry::{VxSize, VxVec2}};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct VxTexVertex {
	position: [f32; 3],
	color: [f32; 4],
	/// テクスチャ座標(u, v) 0.0 ~ 1.0
	tex_coords: [f32; 2],
	texture_index: i32,
}

impl VxTexVertex {
	pub const ATTRS: [wgpu::VertexAttribute; 4] = wgpu::vertex_attr_array![
		0 => Float32x3, // pos
		1 => Float32x4, // color
		2 => Float32x2, // texcoord
		3 => Sint32, // tex_index
	];
	pub const VERTEXBUFFERLAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
		array_stride: std::mem::size_of::<VxTexVertex>() as wgpu::BufferAddress,
		step_mode: wgpu::VertexStepMode::Vertex,
		attributes: &Self::ATTRS,
	};

	#[inline]
	pub fn new(position: [f32; 3], color: VxColor, tex_coords: [f32; 2], tex_index: i32) -> Self {
		Self {
			position,
			color: color.to_array_with_alpha(),
			tex_coords,
			texture_index: tex_index,
		}
	}

	#[inline]
	pub fn position(&self) -> [f32; 3] { self.position }
	#[inline]
	pub fn x(self) -> f32 { self.position[0] }
	#[inline]
	pub fn y(self) -> f32 { self.position[1] }
	#[inline]
	pub fn z(self) -> f32 { self.position[2] }

	#[inline]
	pub fn to_vec2(&self) -> VxVec2 {
		VxVec2::new(self.x(), self.y())
	}

	#[inline]
	pub fn set_x(&mut self, x: f32) {
		self.position[0] = x;
	}
	#[inline]
	pub fn set_y(&mut self, y: f32) {
		self.position[1] = y;
	}
	#[inline]
	pub fn set_z(&mut self, z: f32) {
		self.position[2] = z;
	}
	#[inline]
	pub fn set_position(&mut self, position: [f32; 3]) {
		self.position = position;
	}
	#[inline]
	pub fn set_position_vec2(&mut self, new_pos: VxVec2) {
		self.set_x(new_pos.x());
		self.set_y(new_pos.y());
	}
	#[inline]
	pub fn set_tex_coord(&mut self, u: f32, v: f32) {
		self.tex_coords[0] = u;
		self.tex_coords[1] = v;
	}
	#[inline]
	pub fn set_texture_index(&mut self, id: i32) {
		self.texture_index = id;
	}
}


#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct VxVertex {
	position: [f32; 3],
	color: [f32; 4],
}

impl VxVertex {
	pub const ATTRS: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![
		0 => Float32x3, // pos
		1 => Float32x4, // color
	];
	pub const VERTEXBUFFERLAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
		array_stride: std::mem::size_of::<VxVertex>() as wgpu::BufferAddress,
		step_mode: wgpu::VertexStepMode::Vertex,
		attributes: &Self::ATTRS,
	};

	#[inline]
	pub fn new(position: [f32; 3], color: VxColor) -> Self {
		Self {
			position,
			color: color.to_array_with_alpha(),
		}
	}

	#[inline]
	pub fn position(&self) -> [f32; 3] { self.position }
	#[inline]
	pub fn x(self) -> f32 { self.position[0] }
	#[inline]
	pub fn y(self) -> f32 { self.position[1] }
	#[inline]
	pub fn z(self) -> f32 { self.position[2] }

	#[inline]
	pub fn to_vec2(&self) -> VxVec2 {
		VxVec2::new(self.x(), self.y())
	}

	#[inline]
	pub fn set_x(&mut self, x: f32) {
		self.position[0] = x;
	}
	#[inline]
	pub fn set_y(&mut self, y: f32) {
		self.position[1] = y;
	}
	#[inline]
	pub fn set_z(&mut self, z: f32) {
		self.position[2] = z;
	}
	#[inline]
	pub fn set_position(&mut self, position: [f32; 3]) {
		self.position = position;
	}
	#[inline]
	pub fn set_position_vec2(&mut self, new_pos: VxVec2) {
		self.set_x(new_pos.x());
		self.set_y(new_pos.y());
	}
}


#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct VxSdfVertex {
	position: [f32; 3],
	color: [f32; 4],
	radius: f32,
	uv: [f32; 2],
	size: [f32 ; 2],
}

impl VxSdfVertex {
	pub const ATTRS: [wgpu::VertexAttribute; 5] = wgpu::vertex_attr_array![
		0 => Float32x3,
		1 => Float32x4,
		2 => Float32,
		3 => Float32x2,
		4 => Float32x2,
	];
	pub const VERTEXBUFFERLAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
		array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
		step_mode: wgpu::VertexStepMode::Vertex,
		attributes: &Self::ATTRS,
	};

	#[inline]
	pub fn new(position: [f32; 3], color: VxColor, radius: f32, uv: [f32; 2], rect_pix_size: VxSize) -> Self {
		Self {
			position,
			color: color.to_array_with_alpha(),
			radius,
			uv	,
			size: rect_pix_size.to_array(),
		}
	}

	#[inline]
	pub fn position(&self) -> [f32; 3] { self.position }
	#[inline]
	pub fn x(self) -> f32 { self.position[0] }
	#[inline]
	pub fn y(self) -> f32 { self.position[1] }
	#[inline]
	pub fn z(self) -> f32 { self.position[2] }

	#[inline]
	pub fn to_vec2(&self) -> VxVec2 {
		VxVec2::new(self.x(), self.y())
	}

	#[inline]
	pub fn set_x(&mut self, x: f32) {
		self.position[0] = x;
	}
	#[inline]
	pub fn set_y(&mut self, y: f32) {
		self.position[1] = y;
	}
	#[inline]
	pub fn set_z(&mut self, z: f32) {
		self.position[2] = z;
	}
	#[inline]
	pub fn set_position(&mut self, position: [f32; 3]) {
		self.position = position;
	}
	#[inline]
	pub fn set_position_vec2(&mut self, new_pos: VxVec2) {
		self.set_x(new_pos.x());
		self.set_y(new_pos.y());
	}
	#[inline]
	pub fn set_uv(&mut self, u: f32, v: f32) {
		self.uv[0] = u;
		self.uv[1] = v;
	}
	#[inline]
	pub fn set_size(&mut self, size: VxSize) {
		self.size = size.to_array();
	}
	#[inline]
	pub fn set_radius(&mut self, radius: f32) {
		self.radius = radius;
	}
}