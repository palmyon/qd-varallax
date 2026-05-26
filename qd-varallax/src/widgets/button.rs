use vx_macro::VxWidgetDerive;

use crate::{
	abstractions::{abstract_layouts::VxAlignment, abstract_widgets::{
		VxWidget,
		VxWidgetId,
		VxWidgetInternal,
		VxWidgetStats
	}},
	core::{
		glyph::VxFont,
		vx_event::{
			VxEventResult,
			VxMouseButton
		}
	},
	types::{color::VxColor, geometry::{
		VxRect,
		VxRectR,
		VxVec2
	}},
	vx_widget_signals,
	widgets::{text::VxTextWidget, theme::{
		VxTheme,
		VxThemeMode
	}}
};

vx_widget_signals!(pub struct VxButtonSignals {
	clicked: VxClickedSignal >> VxVec2,
});

pub struct VxButtonStyle {
	theme: VxTheme,
	font: VxFont,
}

impl VxButtonStyle {
	#[inline]
	pub fn new(theme_mode: VxThemeMode, font: VxFont) -> Self {
		match theme_mode {
			VxThemeMode::LightMode => {
				Self {
					theme: VxTheme::light(),
					font,
				}
			},
			VxThemeMode::DarkMode => {
				Self {
					theme: VxTheme::dark(),
					font,
				}
			},
			VxThemeMode::CustomMode { .. } => {
				Self {
					theme: VxTheme::light(),
					font
				}
			}
		}
		
	}

	#[inline]
	pub fn with_font(mut self, font: VxFont) -> Self {
		self.font = font;
		self
	}

	#[inline]
	pub fn font(&self) -> &VxFont { &self.font }
}

pub struct VxButtonState {
	is_pressed: bool,
	is_hovered: bool,
}

impl VxButtonState {
	#[inline]
	pub const fn new() -> Self {
		Self { is_pressed: false, is_hovered: false }
	}

	// getter
	#[inline]
	pub fn is_pressed(&self) -> bool { self.is_pressed }
	#[inline]
	pub fn is_hovered(&self) -> bool { self.is_hovered }

	// setter
	#[inline]
	pub fn set_pressed(&mut self, pressed: bool) {
		self.is_pressed = pressed;
	}
	#[inline]
	pub fn set_hovered(&mut self, hovered: bool) {
		self.is_hovered = hovered;
	}
}

#[derive(VxWidgetDerive)]
pub struct VxButtonWidget {
	#[vx(Stat)]
	stats: VxWidgetStats,
	rect: VxRectR,
	style: VxButtonStyle,
	state: VxButtonState,
	pub signals: VxButtonSignals<Self>
}

impl VxWidget for VxButtonWidget {
	fn bounding_rect(&self) -> VxRect {
		self.rect.rect().with_transform(self.transform())
	}
	fn paint(&mut self, painter: &mut crate::painter::painter::VxPainter) {
		let draw_color = if self.state.is_pressed() {
			self.style.theme.w_primary().with_darken(0.1)
		} else if self.state.is_hovered() {
			self.style.theme.w_primary().with_lighten(0.1)
		} else {
			self.style.theme.w_primary()
		};

		painter.push_tranform(self.transform());
		painter.draw_sdf_rect(self.rect, draw_color);
		painter.pop_transform();
	}

	fn mouse_press_event(&mut self, event: &crate::core::vx_event::VxMouseEvent) -> VxEventResult {
		if let Some(button) = event.button() {
			if button != VxMouseButton::Left { return VxEventResult::Ignore }
		}
		self.state.set_pressed(true);
		self.set_dirty(true);

		let s = self.signals.pressed.clone();
		s.emit(self, &event.pos());
		VxEventResult::Accept
	}

	fn mouse_release_event(&mut self, event: &crate::core::vx_event::VxMouseEvent) -> VxEventResult {
		let s = self.signals.released.clone();
		s.emit(self, &event.pos());
		self.set_dirty(true);

		if self.state.is_hovered() && self.state.is_pressed() {
			self.state.set_pressed(false);

			let s = self.signals.clicked.clone();
			s.emit(self, &event.pos());
			VxEventResult::Accept
		} else {
			self.state.set_pressed(false);
			VxEventResult::Ignore
		}
	}

	fn mouse_enter_event(&mut self, event: &crate::core::vx_event::VxMouseEvent) -> VxEventResult {
		self.state.set_hovered(true);
		self.set_dirty(true);

		let s = self.signals.hovered.clone();
		s.emit(self, &event.pos());
		VxEventResult::Accept
	}

	fn mouse_leave_event(&mut self, event: &crate::core::vx_event::VxMouseEvent) -> VxEventResult {
		self.state.set_hovered(false);
		self.state.set_pressed(false);
		self.set_dirty(true);

		let s = self.signals.leaved.clone();
		s.emit(self, &event.pos());
		VxEventResult::Accept
	}
}

impl VxButtonWidget {
	pub fn new(
		rect: VxRectR,
		text: impl Into<String>,
		style: VxButtonStyle,
		parent: Option<VxWidgetId>
	) -> Self {
		let mut t = VxTextWidget::new(text, VxFont::new("kokumr", 20.0), VxColor::from_hex(0xFFFFFF), None);
		t.stats_mut().set_alignment(VxAlignment::LeftCenter);
		let mut stats = VxWidgetStats::new(parent);
		stats.add_child_widget(t);
		Self {
			stats, 
			rect,
			style,
			state: VxButtonState::new(),
			signals: VxButtonSignals::new(),
		}
	}

	fn set_state(&mut self, state: VxButtonState) {
		self.state = state;
	}
}