use crate::{
	core::glyph::VxFont, painter::tessellate::VxTessellator, types::{
		color::VxColor, geometry::VxRect, render_commands::{
			VxDrawTextData,
			VxRenderMode,
			VxVertexContainer
		}, style::VxSdfStyle, texture::VxTexture, transform::{
			VxMatrix3x3,
			VxTransform
		}, vertex::{
			VxAbstractVertex, VxSdfVertex, VxTextureVertex, VxVertex
		}
	}
};

pub struct VxPainter {
	transform_stack: Vec<VxMatrix3x3>,
	pub(crate) vertices: Vec<VxVertexContainer<VxVertex>>,
	pub(crate) sdf_verts: Vec<VxVertexContainer<VxSdfVertex>>,
	pub(crate) tex_verts: Vec<VxVertexContainer<VxTextureVertex>>,
	pub(crate) text_data: Vec<VxDrawTextData>,

	transform_count: Option<usize>,
}

impl VxPainter {
	pub(crate) fn new() -> Self {
		Self {
			transform_stack: vec![VxMatrix3x3::identity()],
			vertices: Vec::new(),
			sdf_verts: Vec::new(),
			tex_verts: Vec::new(),
			text_data: Vec::new(),
			transform_count: None,
		}
	}

	pub fn push_transform(&mut self, transform: VxTransform) {
		self.transform_stack.push(self.current_transform() * VxMatrix3x3::from_transform(transform));
	}
	pub fn pop_transform(&mut self) {
		let min_limit = self.transform_count.unwrap_or(1);
		if self.transform_stack.len() > min_limit {
			self.transform_stack.pop();
		}
	}
	pub(crate) fn record_transform(&mut self) {
		self.transform_count = Some(self.transform_stack.len());
	}
	pub(crate) fn remove_stacked_transform(&mut self) {
		if let Some(count) = self.transform_count.take() {
			self.transform_stack.truncate(count);
		}
	}

	fn current_transform(&self) -> VxMatrix3x3 {
		*self.transform_stack.last().unwrap()
	}

	fn apply_current_transform<T: VxAbstractVertex>(&self, verts: &mut [T]) {
		let matrix = self.current_transform();
		for v in verts {
			let pos = matrix.transform_point(v.to_vec2());
			v.set_pos_vec2(pos);
		}
	}

	pub(crate) fn set_vertex_z_value(&mut self, z: i32) {
		Self::process_set_vertex_z_value(&mut self.vertices, z);
		Self::process_set_vertex_z_value(&mut self.sdf_verts, z);
		Self::process_set_vertex_z_value(&mut self.tex_verts, z);

		let point = self.text_data.partition_point(|data| data.is_z_enable());
		self.text_data[point..].iter_mut().for_each(|data| data.set_z_value(z));
	}
	fn process_set_vertex_z_value<T: VxAbstractVertex>(target: &mut Vec<VxVertexContainer<T>>, z: i32) {
		let point = target.partition_point(|c| c.is_z_enable());
		target[point..].iter_mut().for_each(|c| c.set_z_value(z));
	}
}

impl VxPainter {
	pub fn draw_rect(&mut self, rect: VxRect, brush: VxColor) {
		let (mut verts, index) = VxTessellator::tessellate_rect(rect, brush);
		self.apply_current_transform(&mut verts);
		
		self.vertices.push(VxVertexContainer::new(verts.to_vec(), index.to_vec()));
	}

	pub fn draw_sdf_rect(&mut self, sdf_style: VxSdfStyle) {
		let (mut verts, index) = VxTessellator::tessellate_sdf_rect(sdf_style);
		self.apply_current_transform(&mut verts);

		self.sdf_verts.push(VxVertexContainer::new(verts.to_vec(), index.to_vec()));
	}

	pub fn draw_texture(&mut self, rect: VxRect, base_color: VxColor, tex: &VxTexture) {
		let Some(id) = tex.id() else { return; };
		let (mut verts, index) = VxTessellator::tessellate_texture(
			rect,
			base_color,
			VxRect::from_i32(0, 0, 1, 1),
			id.index as i32
		);

		self.apply_current_transform(&mut verts);

		self.tex_verts.push(
			VxVertexContainer::new(verts.to_vec(), index.to_vec())
		);
	}

	pub fn draw_text(
		&mut self,
		text: &str,
		font: VxFont,
		color: VxColor,
		outline_color: VxColor,
		outline_width: f32,
		blur_radius: f32,
	) {
		let matrix = self.current_transform();
		self.text_data.push(VxDrawTextData::new(
			text, font, color, matrix, outline_color, outline_width, blur_radius, VxRenderMode::Retained
		));
	}
}