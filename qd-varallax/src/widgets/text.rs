use vx_macro::VxWidgetDerive;

use crate::{
	abstractions::abstract_widgets::{
		VxWidget, VxWidgetId, VxWidgetStats, VxWidgetInternal
	},
	core::glyph::VxFont,
	types::{
		color::VxColor,
		geometry::VxRect
	},
	vx_widget_signals,
};

vx_widget_signals!(pub struct VxTextSignals {
	text_changed: VxTextChangedSignal >> String,
});

#[derive(VxWidgetDerive)]
pub struct VxTextWidget {
	#[vx(Stat)]
	stats: VxWidgetStats,
	text: String,
	last_text: String,
	color: VxColor,
	font: VxFont,
	change_bounding_rect: bool,
	bounding_rect: VxRect,
	signals: VxTextSignals<Self>
}

impl VxWidget for VxTextWidget {
	fn bounding_rect(&self) -> VxRect {
		self.bounding_rect.with_transform(self.transform())
	}
	fn paint(&mut self, painter: &mut crate::painter::painter::VxPainter) {
		if self.change_bounding_rect {
			self.change_bounding_rect = false;
			self.bounding_rect = painter.font_system.create_text_bounding_rect(&self.text, &self.font);
		}
		painter.push_tranform(self.transform());
		painter.draw_text(self.text.clone(), self.font.clone(), self.color);
		painter.draw_rect(self.bounding_rect, VxColor::from_hex_with_alpha(0xFF000066));
		painter.pop_transform();
	}
}

impl VxTextWidget {
	pub fn new(text: impl Into<String>, font: VxFont, color: VxColor, parent: Option<VxWidgetId>) -> Self {
		let text = text.into();
		let mut s = Self {
			stats: VxWidgetStats::new(parent),
			text: "".into(),
			last_text: String::from(""),
			color,
			font,
			change_bounding_rect: false,
			bounding_rect: VxRect::default(),
			signals: VxTextSignals::new(),
		};
		s.set_text(text);
		s
	}

	pub fn set_text(&mut self, text: impl AsRef<str>) {
		let text_ref = text.as_ref();
		if self.text == text_ref { return; }

		self.last_text = std::mem::take(&mut self.text);
		self.text = text_ref.to_string();
		self.change_bounding_rect = true;
		let s = self.signals.text_changed.clone();
		s.emit(self, &self.text.clone());
		self.set_dirty(true);
	}
}