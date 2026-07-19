use qd_varallax_macro::VxWidgetDerive;

use crate::{abstractions::abstract_widgets::*, core::immediate::VxImmediateContext, types::{color::VxColor, geometry::VxRect, render_commands::VxRenderMode}};


#[derive(VxWidgetDerive)]
pub struct VxImmediateAreaWidget {
	#[vx(Stat)]
	stats: VxWidgetStats,
	area_rect: VxRect,
	ui_closure: Box<dyn FnMut(&mut VxImmediateContext)>,
}

impl VxWidget for VxImmediateAreaWidget {
	fn bounding_rect(&self) -> VxRect {
		self.area_rect.with_transform(self.transform())
	}
	fn paint(&mut self, painter: &mut crate::painter::painter::VxPainter) {
		painter.push_tranform(self.transform());
		painter.draw_rect(self.bounding_rect(), VxColor::from_hex(0xFF0000).with_alpha(0.6));
		painter.pop_transform();
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