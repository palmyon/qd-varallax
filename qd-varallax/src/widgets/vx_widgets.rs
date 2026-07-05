
use vx_macro::VxWidgetDerive;

use crate::{
	abstractions::abstract_widgets::{
		VxWidget, VxWidgetId, VxWidgetInternal, VxWidgetStats
	},
	types::{
		color::VxColor,
		geometry::VxRect,
		texture::VxTexture
	},
};


#[derive(VxWidgetDerive)]
pub struct VxRectWidget {
	#[vx(Stat)]
	stat: VxWidgetStats,
	rect: VxRect,
	color: VxColor,
}

impl VxWidget for VxRectWidget {
	fn bounding_rect(&self) -> VxRect {
		self.rect.with_transform(self.transform())
	}

	fn paint(&mut self, painter: &mut crate::painter::painter::VxPainter) {
		painter.push_tranform(self.transform());
		painter.draw_rect(self.rect, self.color);
		painter.pop_transform();
	}
}

impl VxRectWidget {
	pub fn new(rect: VxRect, color: VxColor, parent: Option<VxWidgetId>) -> Self {
		let mut new = Self {
			stat: VxWidgetStats::new(parent),
			rect,
			color,
		};
		new.set_rect(rect);
		new
	}
	pub fn set_rect(&mut self, rect: VxRect) {
		self.rect = rect;
		let mut t = self.transform();
		t.set_size(rect.size());
		self.set_transform(t);
	}
}

#[derive(VxWidgetDerive)]
pub struct VxTextureWidget {
	#[vx(Stat)]
	stat: VxWidgetStats,
	rect: VxRect,
	texture: VxTexture,
	opacity: f32,
}

impl VxWidget for VxTextureWidget {
	fn bounding_rect(&self) -> VxRect {
		self.rect.with_transform(self.transform())
	}
	fn paint(&mut self, painter: &mut crate::painter::painter::VxPainter) {
		painter.push_tranform(self.transform());
		painter.draw_texture(
			self.rect,
			VxColor::from_hex(0xFFFFFF).with_alpha(self.opacity),
			&self.texture()
		);
		painter.pop_transform();
	}

	fn register_texture_event(&mut self, gpu: &crate::core::gpu_resource::VxGpuResource, system: &mut crate::core::systems::VxTextureSystem) {
		system.register_texture(gpu, &mut self.texture);
	}
}

impl VxTextureWidget {
	pub fn new(rect: VxRect, texture: VxTexture, parent:Option<VxWidgetId>) -> Self {
		Self {
			stat: VxWidgetStats::new(parent),
			rect,
			texture,
			opacity: 1.0
		}
	}
	pub fn texture(&self) -> &VxTexture {
		&self.texture
	}

	pub fn set_texture(&mut self, tex: VxTexture) {
		self.texture = tex;
	}
	pub fn set_rect(&mut self, rect: VxRect) {
		self.rect = rect;
	}
	pub fn set_opacity(&mut self, opacity: f32) {
		self.opacity = opacity.clamp(0.0, 1.0);
	}
}