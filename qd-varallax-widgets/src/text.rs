use qd_varallax_macro::{VxWidgetImpl, VxWidgetTextImpl};
use qd_varallax_core::{
	abstracts::prelude::*,
	core::prelude::*,
};

#[derive(Clone, Debug)]
pub struct VxTextWidgetComponent {
	font: VxFont,
	text: String,
	outline_width: f32,
	blur_radius: f32,
	need_update_bounding_rect: bool,
}
impl VxTextWidgetComponent {
	#[inline]
	pub fn new(font: VxFont, text: String, outline_width: f32, blur_radius: f32) -> Self {
		Self { font, text, outline_width, blur_radius, need_update_bounding_rect: true, }
	}
	#[inline]
	pub fn font(&self) -> VxFont { self.font }
	#[inline]
	pub fn text(&self) -> &str { &self.text }
	#[inline]
	pub fn outline_width(&self) -> f32 { self.outline_width }
	#[inline]
	pub fn blur_radius(&self) -> f32 { self.blur_radius }
	#[inline]
	pub fn need_update_bounding_rect(&self) -> bool { self.need_update_bounding_rect }

	#[inline]
	pub fn set_font(&mut self, font: VxFont) {
		self.font = font;
		self.need_update_bounding_rect = true;
	}
	#[inline]
	pub fn set_text(&mut self, text: &str) {
		self.text = text.into();
		self.need_update_bounding_rect = true;
	}
	#[inline]
	pub fn set_outline_width(&mut self, outline_width: f32) {
		self.outline_width = outline_width;
		self.need_update_bounding_rect = true;
	}
	#[inline]
	pub fn set_blur_radius(&mut self, blur_radius: f32) {
		self.blur_radius = blur_radius;
		self.need_update_bounding_rect = true;
	}
	#[inline]
	pub fn set_need_update_bounding_rect(&mut self, need: bool) {
		self.need_update_bounding_rect = need;
	}
}

pub trait VxWidgetTextExtension: VxWidget {
	fn text_component(&self) -> &VxTextWidgetComponent;
	fn text_component_mut(&mut self) -> &mut VxTextWidgetComponent;
}

pub trait VxWidgetTextExtensionExt: VxWidgetTextExtension {
	#[inline]
	fn font(&self) -> VxFont { self.text_component().font() }
	#[inline]
	fn text(&self) -> &str { &self.text_component().text() }
	#[inline]
	fn outline_width(&self) -> f32 { self.text_component().outline_width() }
	#[inline]
	fn blur_radius(&self) -> f32 { self.text_component().blur_radius() }
	#[inline]
	fn need_update_text_bounding_rect(&self) -> bool { self.text_component().need_update_bounding_rect() }

	#[inline]
	fn set_font(&mut self, font: VxFont) {
		self.text_component_mut().set_font(font);
		self.set_dirty_flag(VxDirtyFlag::REBUILD_ALL);
	}
	#[inline]
	fn set_text(&mut self, text: &str) {
		self.text_component_mut().set_text(text);
		self.set_dirty_flag(VxDirtyFlag::REBUILD_ALL);
	}
	#[inline]
	fn set_outline_width(&mut self, outline_width: f32) {
		self.text_component_mut().set_outline_width(outline_width);
		self.set_dirty_flag(VxDirtyFlag::REPAINT);
	}
	#[inline]
	fn set_blur_radius(&mut self, blur_radius: f32) {
		self.text_component_mut().set_blur_radius(blur_radius);
		self.set_dirty_flag(VxDirtyFlag::REPAINT);
	}
	#[inline]
	fn updated_text_bounding_rect(&mut self) {
		self.text_component_mut().set_need_update_bounding_rect(false);
	}
}
impl<T: VxWidgetTextExtension + ?Sized> VxWidgetTextExtensionExt for T {}

#[derive(VxWidgetImpl, VxWidgetTextImpl)]
pub struct VxTextWidget {
	#[attr(context)]
	context: VxWidgetContext,
	#[attr_text(component)]
	text_component: VxTextWidgetComponent,
}

impl VxWidget for VxTextWidget {
	fn size_hint(&mut self, creator: &mut VxBoundingRectCreator) -> VxSizeHint {
		if self.need_update_text_bounding_rect() {
			self.set_block_dirty(true);
			let bounding_rect = creator.create_text_bounding_rect(&self.text_component.text(), self.text_component.font());
			self.context_mut().set_bounding_rect(bounding_rect);
			self.updated_text_bounding_rect();
			self.set_block_dirty(false);
		}
		VxSizeHint::Content(self.bounding_rect().size())
	}
	fn paint(&mut self, painter: &mut qd_varallax_core::painter::painter::VxPainter, palette: &qd_varallax_core::types::style::VxColorPalette) {
		painter.draw_text(
			&self.text_component.text(),
			self.text_component.font(),
			palette.text_primary(),
			palette.outline(),
			self.text_component.outline_width(),
			self.text_component.blur_radius(),
		);
	}
}

impl VxTextWidget {
	pub fn new(text: impl Into<String>, parent: Option<VxWidgetId>) -> Self {
		Self {
			context: VxWidgetContext::new(parent),
			text_component: VxTextWidgetComponent::new(
				VxFont::default(),
				text.into(),
				0.0,
				0.0
			),
		}
	}
}