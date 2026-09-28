use qd_varallax_core::{abstracts::prelude::*, types::prelude::*, vx_widget_signals};
use qd_varallax_macro::VxWidgetImpl;


vx_widget_signals!(pub struct VxButtonSignal {
	clicked: VxClickedSignal >> VxVec2,
});

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VxButtonState {
	is_pressed: bool,
	is_hovered: bool,
}
impl VxButtonState {
	#[inline]
	pub const fn new() -> Self { Self { is_pressed: false, is_hovered: false, } }
	#[inline]
	pub const fn is_pressed(&self) -> bool { self.is_pressed }
	#[inline]
	pub const fn is_hovered(&self) -> bool { self.is_hovered }
	#[inline]
	pub const fn set_pressed(&mut self, pressed: bool) { self.is_pressed = pressed; }
	#[inline]
	pub const fn set_hovered(&mut self, hovered: bool) { self.is_hovered = hovered; }
}

#[derive(VxWidgetImpl)]
pub struct VxButtonWidget {
	#[attr(stat)]
	stats: VxWidgetStats,
	state: VxButtonState,
	signals: VxButtonSignal<Self>,
	optimize_signal: bool,

	corner_radius: f32,
	outline_width: f32,
	text: String,
}

impl VxWidget for VxButtonWidget {
	#[inline]
	fn size_hint(&mut self, _: &mut VxBoundingRectCreator) -> VxSizeHint {
		VxSizeHint::ChildrenSize { padding: 2.0 }
	}
	fn paint(&mut self, painter: &mut qd_varallax_core::painter::painter::VxPainter, palette: &qd_varallax_core::types::style::VxColorPalette) {
		let color = {
			if self.state.is_pressed() {
				palette.widget_secondary()
			} else if self.state.is_hovered() {
				palette.widget_primary().with_lighten(0.05)
			} else {
				palette.widget_primary()
			}
		};
		painter.draw_sdf_rect(VxSdfStyle::new(
			VxRectR::new(self.bounding_rect(), self.corner_radius),
			color,
			palette.outline(),
			self.outline_width,
			0.5
		));
	}

	fn mouse_enter_event(&mut self, event: &VxMouseEvent) -> VxEventResult {
		self.state.set_hovered(true);
		self.set_dirty_flag(VxDirtyFlag::REPAINT);
		self.signals.hovered.clone().emit(self, &event.pos());
		VxEventResult::Accept
	}
	fn mouse_leave_event(&mut self, event: &VxMouseEvent) -> VxEventResult {
		self.state.set_hovered(false);
		self.set_dirty_flag(VxDirtyFlag::REPAINT);
		self.signals.leaved.clone().emit(self, &event.pos());
		VxEventResult::Accept
	}
	fn mouse_press_event(&mut self, event: &VxMouseEvent) -> VxEventResult {
		if let Some(button) = event.button() {
			if button != VxMouseButton::Left { return VxEventResult::Ignore; }
		}
		self.state.set_pressed(true);
		self.signals.pressed.clone().emit(self, &event.pos());
		self.set_dirty_flag(VxDirtyFlag::REPAINT);
		VxEventResult::Accept
	}
	fn mouse_release_event(&mut self, event: &VxMouseEvent) -> VxEventResult {
		self.signals.released.clone().emit(self, &event.pos());
		self.set_dirty_flag(VxDirtyFlag::REPAINT);
		if self.state.is_hovered() && self.state.is_pressed() {
			self.state.set_pressed(false);
			self.signals.clicked.clone().emit(self, &event.pos());
			VxEventResult::Accept
		} else {
			self.state.set_pressed(false);
			VxEventResult::Ignore
		}
	}
	fn mouse_move_event(&mut self, event: &VxMouseEvent) -> VxEventResult {
		if self.optimize_signal { return VxEventResult::Ignore }
		self.signals.moved.clone().emit(self, &event.pos());
		VxEventResult::Ignore
	}
}

impl VxButtonWidget {
	#[inline]
	pub fn new(text: impl Into<String>, parent: Option<VxWidgetId>) -> Self {
		let stats = VxWidgetStats::new(parent);
		Self {
			stats,
			state: VxButtonState::new(),
			signals: VxButtonSignal::new(),
			optimize_signal: true,
			corner_radius: 5.0,
			outline_width: 2.0,
			text: text.into(),
		}
	}
}