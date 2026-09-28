use qd_varallax_macro::VxWidgetImpl;

use crate::{abstracts::{abstract_layouts::VxSizeHint, abstract_widgets::*}, core::immediate::VxImmediateContext, types::{geometry::VxRect, render_commands::VxRenderMode}};


#[derive(VxWidgetImpl)]
pub struct VxImmediateAreaWidget {
	#[attr(stat)]
	stats: VxWidgetStats,
	area_rect: VxRect,
	ui_closure: Box<dyn FnMut(&mut VxImmediateContext)>,
}

impl VxWidget for VxImmediateAreaWidget {
	fn bounding_rect(&self) -> VxRect {
		self.area_rect
	}
	fn size_hint(&mut self, _: &mut crate::abstracts::abstract_layouts::VxBoundingRectCreator) -> VxSizeHint {
		VxSizeHint::Content(self.area_rect.size())
	}
	fn paint(&mut self, painter: &mut crate::painter::painter::VxPainter, palette: &crate::types::style::VxColorPalette) {
		let _ = painter;
		let _ = palette;
	}
	fn immediate_paint(&mut self, input: &crate::types::input::VxInputState, painter: &mut crate::painter::painter::VxPainter) {
		let mut ctx = VxImmediateContext::new(
			input, painter, self.area_rect.size(), self.pos()
		);
		(self.ui_closure)(&mut ctx);
	}
}

impl VxImmediateAreaWidget {
	pub fn new<F>(area: VxRect, ui: F, parent: Option<VxWidgetId>) -> Self
	where F: FnMut(&mut VxImmediateContext) + 'static
	{
		let mut s = Self {
			stats: VxWidgetStats::new(parent),
			area_rect: area,
			ui_closure: Box::new(ui),
		};
		s.set_update_mode(VxRenderMode::Immediate);
		s
	}
}