use qd_varallax_core::{
	abstracts::prelude::*, core::prelude::*, types::{prelude::*, render_commands::VxRenderMode},
};
use qd_varallax_macro::VxWidgetImpl;


#[derive(VxWidgetImpl)]
pub struct VxImmediateAreaWidget {
	#[attr(context)]
	context: VxWidgetContext,
	ui: Box<dyn FnMut(&mut VxImmediateContext)>,
}

impl VxWidget for VxImmediateAreaWidget {
	fn size_hint(&mut self, _: &mut VxBoundingRectCreator) -> VxSizeHint {
		VxSizeHint::Content(VxSize::from_i32(1, 1))
	}
	fn paint(&mut self, painter: &mut qd_varallax_core::painter::painter::VxPainter, palette: &VxColorPalette) {
		let _ = painter;
		let _ = palette;
	}
	fn immediate_paint(&mut self, input: &VxInputState, painter: &mut qd_varallax_core::painter::painter::VxPainter) {
		let mut ctx = VxImmediateContext::new(input, painter, self.bounding_rect().size(), VxVec2::default());
		(self.ui)(&mut ctx)
	}
}

impl VxImmediateAreaWidget {
	pub fn new(ui: impl FnMut(&mut VxImmediateContext) + 'static, parent: Option<VxWidgetId>) -> Self {
		let mut context = VxWidgetContext::new(parent);
		context.set_update_mode(VxRenderMode::Immediate);
		Self {
			context,
			ui: Box::new(ui)
		}
	}
}