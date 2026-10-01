#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use qd_varallax_macro::VxWindowImpl;
use qd_varallax_core::{
	abstracts::prelude::*, core::{gpu::VxGpuBackend, prelude::*},
};
use qd_varallax_widgets::{
	button::VxButtonWidget, layout::{VxHBoxLayoutWidget, VxVBoxLayoutWidget}, text::VxWidgetTextExtensionExt
};

#[derive(VxWindowImpl)]
pub struct DemoWindow {
	#[attr(context)]
	context: Option<VxWindowContext>,
	#[attr(options)]
	attr: VxWindowAttributes,
}

impl DemoWindow {
	pub fn new(attr: VxWindowAttributes) -> Self {
		Self { context: None, attr }
	}
}
impl VxWindow for DemoWindow {
	fn init_event(&mut self) {
		self.set_window_layer(VxWindowLayer::AlwaysOnTopLayer);
		let mut v_layout = VxHBoxLayoutWidget::new(None);
		v_layout.set_padding(10.0);

		for _ in 0..4 {
			let mut layout = VxVBoxLayoutWidget::new(None);
			for _ in 0..4 {
				let mut button = VxButtonWidget::new("", None);
				button.set_font(VxFont::from_family_str("kokumr", 13.0));
				button.signals.clicked.connect(|btn, pos| {
					btn.set_text(&format!("座標:{},{}", pos.x(), pos.y()));
				});
				layout.add_widget(button);
			}
			v_layout.add_widget(layout);
		}

		self.add_widget(v_layout);
	}
}



fn main() {
	let mut app = VxApplication::new()
		.with_render_backend(VxGpuBackend::DX12)
		.with_target_frame_rate(60.0);
	let window = DemoWindow::new(Default::default());
	app.add_window(window);
	app.exec();
}