use crate::{abstractions::{abstract_widgets::VxWidget, abstract_windows::*}, types::{color::VxColor, geometry::VxRect}, widgets::{immediate_area::VxImmediateAreaWidget, vx_widgets::VxRectWidget}};
use vx_macro::VxWindowDerive;

#[derive(VxWindowDerive)]
pub struct DemoWindow {
	#[vx(Stat)]
	stat: Option<VxWindowStats>,
	#[vx(WindowAttr)]
	window_attr: VxWindowAttributes,
}

impl VxWindow for DemoWindow {
	fn init_event(&mut self) {
		for i in 0..1920 {
			let mut rect = VxRectWidget::new(
				VxRect::from_i32(0, 0, 1, 1080),
				VxColor::from_hsv(i as f32 / 1920.0, 1.0, 1.0),
				None
			);
			rect.set_pos((i, 0).into());
			self.add_widget(rect);
		}

		let mut counter: f32 = 0.0;
		let mut area = VxImmediateAreaWidget::new(
			VxRect::from_i32(0, 0, 500, 500),
			move |ctx| {
				counter += 0.01;
				ctx.button("HI", VxColor::from_hsv(counter, 1.0, 1.0));
			},
			None
		);
		area.set_pos((150, 250).into());
		self.add_widget(area);
	}
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