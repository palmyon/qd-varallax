use qd_varallax_macro::VxWidgetImpl;
use qd_varallax_core::{
	abstracts::prelude::*,
	core::prelude::*,
};

#[derive(VxWidgetImpl)]
pub struct VxTextWidget {
	#[attr(stat)]
	stats: VxWidgetStats,

	font: VxFont,
	need_update_bounding_rect: bool,
	text: String,

	outline_width: f32,
}

impl VxWidget for VxTextWidget {
	fn size_hint(&mut self, creator: &mut VxBoundingRectCreator) -> VxSizeHint {
		if self.need_update_bounding_rect {
			self.set_block_dirty(true);
			let bounding_rect = creator.create_text_bounding_rect(&self.text, self.font);
			self.stats_mut().set_bounding_rect(bounding_rect);
			self.set_block_dirty(false);
		}
		VxSizeHint::Content(self.bounding_rect().size())
	}
	fn paint(&mut self, painter: &mut qd_varallax_core::painter::painter::VxPainter, palette: &qd_varallax_core::types::style::VxColorPalette) {
		painter.draw_text(
			&self.text,
			self.font,
			palette.text_primary(),
			palette.outline(),
			self.outline_width,
			0.0,
		);
	}
}

impl VxTextWidget {
	pub fn new(text: impl Into<String>, parent: Option<VxWidgetId>) -> Self {
		Self {
			stats: VxWidgetStats::new(parent),
			font: VxFont::default(),
			need_update_bounding_rect: true,
			text: text.into(),
			outline_width: 1.5,
		}
	}
}

impl VxTextWidget {
	pub fn set_font(&mut self, font: VxFont) {
		self.font = font;
		self.set_dirty_flag(VxDirtyFlag::REBUILD_ALL);
	}
}