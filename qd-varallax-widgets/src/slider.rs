use qd_varallax_core::{
	abstracts::prelude::*,
	core::prelude::*,
	types::prelude::*,
};
use qd_varallax_macro::{VxWidgetButtonImpl, VxWidgetImpl, VxWidgetTextImpl, VxWidgetValueImpl};

use crate::{
	abstracts::value_components::{
		VxValueWidgetComponent,
		VxWidgetValueExtension,
		VxWidgetValueExtensionExt
	},
	button::{
		VxButtonWidgetComponent,
		VxWidgetButtonExtension,
		VxWidgetButtonExtensionExt,
		VxWidgetButtonExtensionInternalExt,
	},
	text::{
		VxTextWidgetComponent,
		VxWidgetTextExtension,
		VxWidgetTextExtensionExt,
	}
};

#[derive(VxWidgetImpl, VxWidgetButtonImpl, VxWidgetTextImpl, VxWidgetValueImpl)]
pub struct VxSliderWidget {
	#[attr(context)]
	context: VxWidgetContext,
	#[attr_button(component)]
	button_component: VxButtonWidgetComponent,
	#[attr_text(component)]
	text_component: VxTextWidgetComponent,
	#[attr_value(component)]
	value_component: VxValueWidgetComponent,

	slider_rect: VxRect,
	text_bounding_rect: VxRect,
}

impl VxWidget for VxSliderWidget {
	fn size_hint(&mut self, creator: &mut VxBoundingRectCreator) -> VxSizeHint {
		if self.need_update_text_bounding_rect() {
			let bounding_rect = creator.create_text_bounding_rect(self.text(), self.font());
			self.text_bounding_rect = bounding_rect;
			self.updated_text_bounding_rect();
		}
		let mut size = self.text_bounding_rect.size();
		size = size.with_width(size.width() + Self::SLIDER_MIN_WIDTH).with_height(size.height().max(Self::SLIDER_HEIGHT));
		VxSizeHint::Content(size)
	}
	fn paint(&mut self, painter: &mut qd_varallax_core::painter::painter::VxPainter, palette: &VxColorPalette) {
		let bounding_rect = self.bounding_rect();
		let text_bounding_rect = self.text_bounding_rect;
		let slider_rect = VxRect::new(
			0.0,
			0.0,
			bounding_rect.width() - text_bounding_rect.width() - (Self::PADDING_BETWEEN_TEXT_SLIDER * 2.0),
			Self::SLIDER_HEIGHT,
		);
		let slider_mask = VxRect::new(
			0.0,
			0.0,
			slider_rect.width() * self.ratio() as f32,
			Self::SLIDER_HEIGHT,
		);
		self.slider_rect = slider_rect.with_pos(VxVec2::new(
			text_bounding_rect.width() + Self::PADDING_BETWEEN_TEXT_SLIDER,
			bounding_rect.left_center().y() - slider_rect.left_center().y()
		));

		painter.push_transform(VxTransform::from_translation(bounding_rect.left_center() - slider_rect.left_center()));
		painter.push_transform(VxTransform::from_translation(text_bounding_rect.left_center()));
		painter.draw_text(
			self.text(),
			self.font(),
			palette.text_primary(),
			palette.text_secondary(),
			self.text_outline_width(),
			self.text_blur_radius(),
		);
		painter.pop_transform();

		painter.push_transform(VxTransform::from_translation(VxVec2::new(text_bounding_rect.width() + Self::PADDING_BETWEEN_TEXT_SLIDER, 0.0)));
		painter.draw_sdf_rect(VxSdfStyle::new(
			VxRectR::new(slider_rect, 2.0),
			palette.outline().with_lighten(0.1),
			palette.widget_secondary(),
			1.0,
			0.0
		));
		painter.draw_sdf_rect(VxSdfStyle::new(
			VxRectR::new(slider_mask, 2.0),
			palette.accent(),
			palette.outline(),
			0.0,
			0.0
		));

		let toggle_rectr = VxRectR::new(
			VxRect::new(0.0, 0.0, Self::TOGGLE_SIZE, Self::TOGGLE_SIZE),
			Self::TOGGLE_SIZE / 2.0
		);
		painter.push_transform(VxTransform::from_translation(slider_mask.right_center() - toggle_rectr.rect().center()));
		painter.draw_sdf_rect(VxSdfStyle::new(
			toggle_rectr,
			palette.accent(),
			palette.widget_primary(),
			Self::TOGGLE_SIZE / 2.0,
			1.0
		));
	}

	fn mouse_press_event(&mut self, event: &VxMouseEvent) -> VxEventResult {
		let is_contains = self.move_slider(event);
		self.set_pressed(is_contains);
		VxEventResult::Accept
	}
	fn mouse_move_event(&mut self, event: &VxMouseEvent) -> VxEventResult {
		if self.is_pressed() {
			let is_contains = self.move_slider(event);
			if !is_contains {
				self.set_pressed(false);
			}
		}
		VxEventResult::Accept
	}
	fn mouse_release_event(&mut self, _: &VxMouseEvent) -> VxEventResult {
		self.set_pressed(false);
		VxEventResult::Accept
	}
}

impl VxSliderWidget {
	const SLIDER_MIN_WIDTH: f32 = 20.0;
	const SLIDER_HEIGHT: f32 = 8.0;
	const TOGGLE_SIZE: f32 = Self::SLIDER_HEIGHT + 2.0;
	const PADDING_BETWEEN_TEXT_SLIDER: f32 = 6.0;
	pub fn new(label: impl Into<String>, parent: Option<VxWidgetId>) -> Self {
		Self {
			context: VxWidgetContext::new(parent),
			button_component: VxButtonWidgetComponent::new(),
			text_component: VxTextWidgetComponent::new(
				VxFont::default(),
				label.into(),
				0.0,
				0.0,
			),
			value_component: VxValueWidgetComponent::new(0.0, 100.0),
			slider_rect: VxRect::default(),
			text_bounding_rect: VxRect::default(),
		}
	}
}

impl VxSliderWidget {
	fn move_slider(&mut self, event: &VxMouseEvent) -> bool {
		let slider_rect = self.slider_rect;
		let pos = event.pos() - self.pos();
		let text_width = self.text_bounding_rect.width();
		let is_contains = slider_rect.contains(pos);
		if is_contains {
			let ratio = ((pos.x() - text_width - Self::PADDING_BETWEEN_TEXT_SLIDER) / slider_rect.width()) as f64;
			self.set_value(self.range() * ratio);
		}
		is_contains
	}
}