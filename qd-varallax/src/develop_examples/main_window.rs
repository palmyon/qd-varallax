use crate::{abstractions::{abstract_layouts::*, abstract_widgets::*, abstract_windows::*}, types::{color::VxColor, geometry::VxRect}, widgets::{immediate_area::VxImmediateAreaWidget, layout::{VxHBoxLayoutWidget, VxVBoxLayoutWidget}, vx_widgets::VxRectWidget}};
use qd_varallax_macro::VxWindowImpl;

#[derive(VxWindowImpl)]
pub struct DemoWindow {
	#[attr(stat)]
	stat: Option<VxWindowStats>,
	#[attr(w_attr)]
	window_attr: VxWindowAttributes,
}

impl VxWindow for DemoWindow {
	fn init_event(&mut self) {
		let mut layout = VxVBoxLayoutWidget::new(None);
		let mut layout2 = VxHBoxLayoutWidget::new(None);
		for _ in 0..3 {
			layout2.add_widget(VxRectWidget::new(VxRect::from_i32(0, 0, 10, 10), VxColor::from_hex(0x00D4FF), None));
		}

		let mut counter: f32 = 0.0;
		let mut area = VxImmediateAreaWidget::new(
			VxRect::from_i32(0, 0, 500, 500),
			move |ctx| {
				counter += 0.01;
				ctx.button("HI", VxColor::from_hsv(counter, 1.0, 1.0), (250.0 * (counter.sin().abs() + 1.0), 50.0).into());
			},
			None
		);
		area.set_pos((150, 250).into());
		layout2.add_widget(area);
		layout.add_widget(layout2);
		layout.add_widget(VxRectWidget::new(VxRect::from_i32(0, 0, 10, 10), VxColor::from_hex(0x00D4FF), None));
		self.add_widget(layout);
	}
	#[inline]
	fn has_immediate(&self) -> bool {
		true
	}
}

impl DemoWindow {
	pub fn new(attr: VxWindowAttributes) -> Self {
		Self {
			stat: None,
			window_attr: attr,
		}
	}
}