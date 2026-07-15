use crate::{
	abstractions::abstract_layouts::VxImmediateLayoutContext, painter::painter::VxPainter, types::{color::VxColor, geometry::{VxRect, VxRectR, VxSize, VxVec2}, input::{VxInputState, VxMouseButton}, style::VxSdfStyle, transform::VxTransform}
};


pub struct VxImmediateContext<'a> {
	input: &'a VxInputState,
	painter: &'a mut VxPainter,
	layout: VxImmediateLayoutContext,
	scene_origin_point: VxVec2,
}

impl<'a> VxImmediateContext<'a> {
	pub(crate) fn new(
		input: &'a VxInputState,
		painter: &'a mut VxPainter,
		available_size: VxSize,
		scene_origin_point: VxVec2,
	) -> Self {
		Self {
			input,
			painter,
			layout: VxImmediateLayoutContext::new(available_size),
			scene_origin_point,
		}
	}
	
	/// API
	
	pub fn button(&mut self, _text: &str, color: VxColor) -> bool {
		let size = VxSize::new(120.0, 50.0);
		let local_pos = self.layout.cursor();
		let global_pos = self.scene_origin_point + local_pos;
		let button_rect = VxRect::from_pos_size(global_pos, size);

		let mouse_pos = self.input.mouse().pos();
		let is_hovered = button_rect.contains(mouse_pos);

		let mut clicked = false;
		if is_hovered {
			if let Some(state) = self.input.mouse().buttons().get(&VxMouseButton::Left) {
				if state.is_released() {
					clicked = true;
				}
			}
		}

		self.painter.push_tranform(VxTransform::new(
			(200, 150).into(), Default::default(), VxSize::from_i32(1, 1), 0.0.into(), Default::default())
		);
		self.painter.draw_sdf_rect(
			VxSdfStyle::new(VxRectR::new(button_rect, 5.0), color,
			VxColor::from_hex(0x00FF00), 2.0, 1.0
		));
		self.painter.pop_transform();
		self.painter.set_vertex_z_value(1);

		self.layout.set_cursor(self.layout.cursor().with_translate((0.0, size.height() + 8.0).into()));

		clicked
	}
}