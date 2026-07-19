use qd_varallax_macro::VxWidgetDerive;

use crate::{
	abstractions::abstract_widgets::{
		VxDirtyFlag, VxWidget, VxWidgetId, VxWidgetInternal, VxWidgetStats
	}, core::glyph::VxFont, types::{
		color::VxColor,
		geometry::VxRect
	}, vx_widget_signals,
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
		painter.push_tranform(self.transform());
		painter.draw_text(
			&self.text,
			self.font,
			self.color,
			VxColor::from_hex(0x00D4FF),
			1.5,
			1.5
		);
		painter.pop_transform();
	}
	fn create_bounding_rect_event(&mut self, system: &crate::core::systems::VxFontSystem) {
		if self.change_bounding_rect {
			self.bounding_rect = system.create_text_bounding_rect(self.font, &self.text);
		}
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
		self.set_dirty_flag(VxDirtyFlag::REPAINT);
	}
}