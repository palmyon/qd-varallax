use crate::{
	core::{
		glyph::VxFont,
	},
	painter::tessellate::{
		self
	},
	types::{
		color::VxColor,
		geometry::{
			VxRect,
			VxRectR
		},
		texture::VxTexture,
		transform::{
			VxMatrix3x3,
			VxTransform
		},
		vertex::{
			VxSdfVertex,
			VxTexVertex,
			VxVertex
		}
	}
};

pub(crate) struct VxDrawTextData {
	pub(crate) text: String,
	pub(crate) font: VxFont,
	pub(crate) color: VxColor,
	pub(crate) matrix: VxMatrix3x3,
	pub(crate) z_value: VxVertexZValue,
}

impl VxDrawTextData {
	pub(crate) fn new(text: impl Into<String>, font: VxFont, color: VxColor, matrix: VxMatrix3x3) -> Self {
		Self {
			text: text.into(), font, color, matrix, z_value: VxVertexZValue::Disable
		}
	}
	pub fn z_value(&mut self) -> VxVertexZValue { std::mem::take(&mut self.z_value) }
	pub fn is_z_enable(&self) -> bool { self.z_value != VxVertexZValue::Disable }
	pub fn set_z_value(&mut self, z: i32) {
		self.z_value = VxVertexZValue::Enable { z };
	}
}

#[derive(Clone, Copy, Default, PartialEq)]
pub(crate) enum VxVertexZValue {
	#[default]
	Disable,
	Enable { z: i32 }
}
impl VxVertexZValue {
	pub fn z_value(&self) -> i32 {
		match self {
			Self::Disable => 0,
			Self::Enable { z } => *z,
		}
	}
}

pub(crate) struct VxVertexContainer<T> {
	verts: Vec<T>,
	index: Vec<u16>,
	z_value: VxVertexZValue,
}
impl<T> VxVertexContainer<T> {
	pub fn new(verts: Vec<T>, index: Vec<u16>) -> Self {
		Self { verts, index, z_value: VxVertexZValue::Disable }
	}
	pub fn verts(&mut self) -> Vec<T> {
		std::mem::take(&mut self.verts)
	}
	pub fn index(&mut self) -> Vec<u16> {
		std::mem::take(&mut self.index)
	}
	pub fn z_value(&self) -> i32 { self.z_value.z_value() }
	pub fn is_z_enable(&self) -> bool { self.z_value != VxVertexZValue::Disable }
	pub fn set_z_value(&mut self, z: i32) {
		self.z_value = VxVertexZValue::Enable { z };
	}
}

pub struct VxPainter {
	transform_stack: Vec<VxMatrix3x3>,
	pub(crate) vertices: Vec<VxVertexContainer<VxVertex>>,
	pub(crate) sdf_verts: Vec<VxVertexContainer<VxSdfVertex>>,
	pub(crate) tex_verts: Vec<VxVertexContainer<VxTexVertex>>,
	pub(crate) text_data: Vec<VxDrawTextData>,
}

impl VxPainter {
	pub fn new() -> Self {
		Self {
			transform_stack: vec![VxMatrix3x3::identity()],
			vertices: vec![],
			sdf_verts: vec![],
			tex_verts: vec![],
			text_data: vec![],
		}
	}

	pub fn current_tranform(&self) -> &VxMatrix3x3 {
		self.transform_stack.last()
		.expect("VxPainter> transform_stack: Not found last VxMatrix3x3. Check [VxWidget> paint] event.")
	}

	pub fn push_tranform(&mut self, transform: VxTransform) {
		let new_matrix = *self.current_tranform() * VxMatrix3x3::from_transform(transform);
		self.transform_stack.push(new_matrix);
	}

	pub fn pop_transform(&mut self) {
		if self.transform_stack.len() > 1 {
			self.transform_stack.pop();
		} else {
			println!("VxPainter> pop_transform: Transform stack length underflowed. Check [VxWidget> paint] event.");
		}
	}

	fn apply_current_transform_tex(&self, verts: &mut [VxTexVertex]) {
		let matrix = self.current_tranform();
		for v in verts {
			let pos = matrix.transform_point(v.to_vec2());
			v.set_position_vec2(pos);
		}
	}

	fn apply_current_transform(&self, verts: &mut [VxVertex]) {
		let matrix = self.current_tranform();
		for v in verts {
			let pos = matrix.transform_point(v.to_vec2());
			v.set_position_vec2(pos);
		}
	}

	fn apply_current_transform_sdf(&self, verts: &mut [VxSdfVertex]) {
		let matrix = self.current_tranform();
		for v in verts {
			let pos = matrix.transform_point(v.to_vec2());
			v.set_position_vec2(pos);
		}
	}

	pub(crate) fn set_vertex_z_value(&mut self, z: i32) {
		let vert_point = self.vertices.partition_point(|c| c.is_z_enable());
		self.vertices[vert_point..].iter_mut().for_each(|c| c.set_z_value(z));

		let sdf_point = self.sdf_verts.partition_point(|c| c.is_z_enable());
		self.sdf_verts[sdf_point..].iter_mut().for_each(|c| c.set_z_value(z));

		let tex_point = self.tex_verts.partition_point(|c| c.is_z_enable());
		self.tex_verts[tex_point..].iter_mut().for_each(|c| c.set_z_value(z));

		let text_point = self.text_data.partition_point(|c| c.is_z_enable());
		self.tex_verts[text_point..].iter_mut().for_each(|c| c.set_z_value(z));
	}

	pub fn draw_rect(&mut self, rect: VxRect, brush: VxColor) {
		let (mut verts, index) = tessellate::tessellate_rect(&rect, &brush);
		self.apply_current_transform(&mut verts);
		
		self.vertices.push(VxVertexContainer::new(verts.to_vec(), index.to_vec()));
	}

	pub fn draw_sdf_rect(&mut self, rect: VxRectR, brush: VxColor) {
		let (mut verts, index) = tessellate::tessellate_sdf_rect(&rect, &brush);
		self.apply_current_transform_sdf(&mut verts);

		self.sdf_verts.push(VxVertexContainer::new(verts.to_vec(), index.to_vec()));
	}

	pub fn draw_texture(&mut self, rect: VxRect, base_color: VxColor, tex: &VxTexture) {
		let Some(id) = tex.id() else { return; };
		let (mut verts, index) = tessellate::tessellate_texture(
			rect,
			base_color,
			VxRect::from_i32(0, 0, 1, 1),
			id.index as i32
		);

		self.apply_current_transform_tex(&mut verts);

		self.tex_verts.push(
			VxVertexContainer::new(verts.to_vec(), index.to_vec())
		);
	}

	pub fn draw_text(&mut self, text: impl Into<String>, font: VxFont, color: VxColor) {
		let matrix = *self.current_tranform();
		self.text_data.push(VxDrawTextData::new(text, font, color, matrix));
	}
}