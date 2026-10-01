use qd_varallax_core::{abstracts::prelude::*, core::prelude::*, types::prelude::*, vx_widget_signals};
use qd_varallax_macro::{VxWidgetButtonImpl, VxWidgetImpl, VxWidgetTextImpl};

use crate::text::{VxTextWidgetComponent, VxWidgetTextExtension, VxWidgetTextExtensionExt};


vx_widget_signals!(pub struct VxButtonSignal {
	clicked: VxClickedSignal >> VxVec2,
});

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VxButtonWidgetComponent {
	is_pressed: bool,
	is_hovered: bool,
	clickable: bool,
}
impl VxButtonWidgetComponent {
	#[inline]
	pub const fn new() -> Self { Self { is_pressed: false, is_hovered: false, clickable: true, } }
	#[inline]
	pub const fn is_pressed(&self) -> bool { self.is_pressed }
	#[inline]
	pub const fn is_hovered(&self) -> bool { self.is_hovered }
	#[inline]
	pub const fn is_clickable(&self) -> bool { self.clickable }
	#[inline]
	pub const fn set_pressed(&mut self, pressed: bool) { self.is_pressed = pressed; }
	#[inline]
	pub const fn set_hovered(&mut self, hovered: bool) { self.is_hovered = hovered; }
	#[inline]
	pub const fn set_clickable(&mut self, clickable: bool) { self.clickable = clickable; }
}

pub trait VxWidgetButtonExtension: VxWidget {
	fn button_component(&self) -> &VxButtonWidgetComponent;
	fn button_component_mut(&mut self) -> &mut VxButtonWidgetComponent;
}

pub trait VxWidgetButtonExtensionExt: VxWidgetButtonExtension {
	#[inline]
	fn is_pressed(&self) -> bool { self.button_component().is_pressed() }
	#[inline]
	fn is_hovered(&self) -> bool { self.button_component().is_hovered() }
	#[inline]
	fn is_clickable(&self) -> bool { self.button_component().is_clickable() }
	#[inline]
	fn set_clickable(&mut self, clickable: bool) { self.button_component_mut().set_clickable(clickable); }
}
pub(crate) trait VxWidgetButtonExtensionInternalExt: VxWidgetButtonExtension {
	#[inline]
	fn set_pressed(&mut self, pressed: bool) { self.button_component_mut().set_pressed(pressed); }
	#[inline]
	fn set_hovered(&mut self, hovered: bool) { self.button_component_mut().set_hovered(hovered); }
}
impl<T: VxWidgetButtonExtension + ?Sized> VxWidgetButtonExtensionExt for T {}
impl<T: VxWidgetButtonExtension + ?Sized> VxWidgetButtonExtensionInternalExt for T {}

#[derive(VxWidgetImpl, VxWidgetButtonImpl, VxWidgetTextImpl)]
pub struct VxButtonWidget {
	#[attr(context)]
	context: VxWidgetContext,
	#[attr_button(component)]
	button_component: VxButtonWidgetComponent,
	pub signals: VxButtonSignal<Self>,
	optimize_signal: bool,

	corner_radius: f32,
	outline_width: f32,

	#[attr_text(component)]
	text_component: VxTextWidgetComponent,
	text_bounding_rect: VxRect,
}

impl VxWidget for VxButtonWidget {
	#[inline]
	fn size_hint(&mut self, creator: &mut VxBoundingRectCreator) -> VxSizeHint {
		if self.need_update_text_bounding_rect() {
			let bounding_rect = creator.create_text_bounding_rect(&self.text(), self.font());
			self.text_bounding_rect = bounding_rect;
			self.updated_text_bounding_rect();
		}
		let size = self.text_bounding_rect.size();
		VxSizeHint::Content(VxSize::new(size.width() + 2.0, size.height() + 2.0))
	}
	fn paint(&mut self, painter: &mut qd_varallax_core::painter::painter::VxPainter, palette: &qd_varallax_core::types::style::VxColorPalette) {
		let color = {
			if !self.is_enabled() {
				palette.widget_disabled()
			} else {
				if self.is_pressed() {
					palette.widget_primary().with_darken(if palette.theme() == VxThemeMode::Dark { 0.05 } else { 0.1 })
				} else if self.is_hovered() {
					palette.widget_primary().with_lighten(0.05)
				} else {
					palette.widget_primary()
				}
			}
		};
		painter.draw_sdf_rect(VxSdfStyle::new(
			VxRectR::new(self.bounding_rect(), self.corner_radius),
			color,
			palette.outline(),
			self.outline_width,
			0.5
		));
		let (text_color, outline_color) = if !self.is_enabled() {
			(palette.text_primary().with_darken(0.6), palette.text_secondary().with_darken(0.6))
		} else {
			(palette.text_primary(), palette.text_secondary())
		};
		painter.push_transform(VxTransform::from_translation(self.bounding_rect().center() - self.text_bounding_rect.center()));
		painter.draw_text(
			self.text(),
			self.font(),
			text_color,
			outline_color,
			self.outline_width(),
			self.blur_radius()
		);
		painter.pop_transform();
	}

	fn mouse_enter_event(&mut self, event: &VxMouseEvent) -> VxEventResult {
		if !self.is_enabled() { return VxEventResult::Ignore }
		self.set_hovered(true);
		self.set_dirty_flag(VxDirtyFlag::REPAINT);
		if !self.is_block_signal() {
			self.signals.hovered.clone().emit(self, &event.pos());
		}
		VxEventResult::Accept
	}
	fn mouse_leave_event(&mut self, event: &VxMouseEvent) -> VxEventResult {
		if !self.is_enabled() { return VxEventResult::Ignore }
		self.set_hovered(false);
		self.set_pressed(false);
		self.set_dirty_flag(VxDirtyFlag::REPAINT);
		if !self.is_block_signal() {
			self.signals.leaved.clone().emit(self, &event.pos());
		}
		VxEventResult::Accept
	}
	fn mouse_press_event(&mut self, event: &VxMouseEvent) -> VxEventResult {
		if !self.is_enabled() || !self.is_clickable() { return VxEventResult::Ignore }
		if let Some(button) = event.button() {
			if button != VxMouseButton::Left { return VxEventResult::Ignore; }
		}
		self.set_pressed(true);
		if !self.is_block_signal() {
			self.signals.pressed.clone().emit(self, &event.pos());
		}
		self.set_dirty_flag(VxDirtyFlag::REPAINT);
		VxEventResult::Accept
	}
	fn mouse_release_event(&mut self, event: &VxMouseEvent) -> VxEventResult {
		if !self.is_enabled() || !self.is_clickable() { return VxEventResult::Ignore }
		if let Some(button) = event.button() {
			if button != VxMouseButton::Left { return VxEventResult::Ignore; }
		}
		if !self.is_block_signal() {
			self.signals.released.clone().emit(self, &event.pos());
		}
		self.set_dirty_flag(VxDirtyFlag::REPAINT);
		if self.is_hovered() && self.is_pressed() {
			self.set_pressed(false);
			if !self.is_block_signal() && self.is_clickable() {
				self.signals.clicked.clone().emit(self, &event.pos());
			}
			VxEventResult::Accept
		} else {
			self.set_pressed(false);
			VxEventResult::Ignore
		}
	}
	fn mouse_move_event(&mut self, event: &VxMouseEvent) -> VxEventResult {
		if !self.is_enabled() || self.optimize_signal || self.is_block_signal() {
			return VxEventResult::Ignore
		}
		self.signals.moved.clone().emit(self, &event.pos());
		VxEventResult::Ignore
	}
}

impl VxButtonWidget {
	#[inline]
	pub fn new(text: impl Into<String>, parent: Option<VxWidgetId>) -> Self {
		let context = VxWidgetContext::new(parent);
		Self {
			context,
			button_component: VxButtonWidgetComponent::new(),
			signals: VxButtonSignal::new(),
			optimize_signal: true,
			corner_radius: 3.5,
			outline_width: 0.5,
			text_component: VxTextWidgetComponent::new(
				VxFont::default(),
				text.into(),
				0.0,
				0.0
			),
			text_bounding_rect: VxRect::default(),
		}
	}
}