use crate::{
	core::glyph::VxFont, types::{
		color::VxColor, transform::VxMatrix3x3
	}
};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Hash, PartialOrd, Ord)]
pub(crate) enum VxRenderMode {
	#[default]
	Retained = 0,
	Immediate = 1,
}

pub(crate) struct VxDrawTextData {
	pub(crate) text: String,
	pub(crate) font: VxFont,
	pub(crate) color: VxColor,
	pub(crate) matrix: VxMatrix3x3,
	pub(crate) z_value: VxVertexZValue,
	pub(crate) outline_color: VxColor,
	pub(crate) outline_width: f32,
	pub(crate) blur_radius: f32,
	pub(crate) render_mode: VxRenderMode,
}

impl VxDrawTextData {
	#[inline]
	pub(crate) fn new(
		text: &str,
		font: VxFont,
		color: VxColor,
		matrix: VxMatrix3x3,
		outline_color: VxColor,
		outline_width: f32,
		blur_radius: f32,
		render_mode: VxRenderMode,
	) -> Self {
		Self {
			text: text.into(),
			font,
			color,
			matrix,
			z_value: VxVertexZValue::Disable,
			outline_color,
			outline_width,
			blur_radius,
			render_mode
		}
	}
	#[inline]
	pub fn z_value(&self) -> i32 { self.z_value.z_value() }
	#[inline]
	pub fn is_z_enable(&self) -> bool { self.z_value != VxVertexZValue::Disable }
	#[inline]
	pub fn set_z_value(&mut self, z: i32) {
		self.z_value = VxVertexZValue::Enable { z };
	}
}

#[derive(Clone, Copy, Default, PartialEq, Eq, Hash)]
pub(crate) enum VxVertexZValue {
	#[default]
	Disable,
	Enable { z: i32 }
}
impl VxVertexZValue {
	#[inline]
	pub fn z_value(&self) -> i32 {
		match self {
			Self::Disable => 0,
			Self::Enable { z } => *z,
		}
	}
}

pub(crate) struct VxVertexContainer<T> {
	pub(crate) verts: Vec<T>,
	pub(crate) index: Vec<u32>,
	z_value: VxVertexZValue,
}

impl<T> VxVertexContainer<T> {
	#[inline]
	pub fn new(verts: Vec<T>, index: Vec<u32>) -> Self {
		Self {
			verts,
			index,
			z_value: VxVertexZValue::Disable,
		}
	}
	#[inline]
	pub fn verts(&mut self) -> Vec<T> {
		std::mem::take(&mut self.verts)
	}
	#[inline]
	pub fn index(&mut self) -> Vec<u32> {
		std::mem::take(&mut self.index)
	}
	#[inline]
	pub fn z_value(&self) -> i32 { self.z_value.z_value() }
	#[inline]
	pub fn is_z_enable(&self) -> bool { self.z_value != VxVertexZValue::Disable }
	#[inline]
	pub fn set_z_value(&mut self, z: i32) {
		self.z_value = VxVertexZValue::Enable { z };
	}
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub(crate) enum VxModuleId {
	VertexModule,
	SdfModule,
	TextureModule,
	TextModule,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct VxDrawLine {
	module: VxModuleId,
	index_start: u32,
	index_count: u32,
	z_value: i32,
	render_mode: VxRenderMode,
}
impl VxDrawLine {
	#[inline]
	pub fn new(module: VxModuleId, index_start: u32,
		count: u32, z_value: i32, render_mode: VxRenderMode
	) -> Self {
		Self { module, index_start, index_count: count, z_value, render_mode }
	}
	#[inline]
	pub fn module(&self) -> VxModuleId { self.module }
	#[inline]
	pub fn module_id(&self) -> u8 {
		match self.module {
			VxModuleId::VertexModule => 0,
			VxModuleId::SdfModule => 1,
			VxModuleId::TextureModule => 2,
			VxModuleId::TextModule => 3,
		}
	}
	#[inline]
	pub fn start(&self) -> u32 {
		self.index_start
	}
	#[inline]
	pub fn count(&self) -> u32 {
		self.index_count
	}
	#[inline]
	pub fn z_value(&self) -> i32 {
		self.z_value
	}
	#[inline]
	pub fn set_index_start(&mut self, val: u32) {
		self.index_start = val;
	}
	#[inline]
	pub fn set_index_count(&mut self, val: u32) {
		self.index_count = val;
	}
	#[inline]
	pub fn render_mode(&self) -> VxRenderMode {
		self.render_mode
	}
}

pub struct VxDrawLineContainer {
	draw_lines: Vec<VxDrawLine>,
	sorted: bool,
}

impl VxDrawLineContainer {
	#[inline]
	pub fn new() -> Self {
		Self {
			draw_lines: Vec::new(),
			sorted: false,
		}
	}
	#[inline]
	pub fn is_sorted(&self) -> bool {
		self.sorted
	}
	#[inline]
	pub fn draw_lines(&self) -> &Vec<VxDrawLine> {
		&self.draw_lines
	}
	#[inline]
	pub fn draw_lines_mut(&mut self) -> &mut Vec<VxDrawLine> {
		&mut self.draw_lines
	}
	#[inline]
	pub fn draw_lines_take(&mut self) -> Vec<VxDrawLine> {
		std::mem::take(&mut self.draw_lines)
	}
	#[inline]
	pub fn set_sorted(&mut self, sorted: bool) {
		self.sorted = sorted;
	}
	#[inline]
	pub fn push(&mut self, draw_line: VxDrawLine) {
		self.draw_lines.push(draw_line);
		self.sorted = false;
	}
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum VxDirtyCheckResult {
	None,
	OnlyImmediate,
	All,
}