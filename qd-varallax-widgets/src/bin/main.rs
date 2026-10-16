#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use qd_varallax_macro::VxWindowImpl;
use qd_varallax_core::{
	abstracts::{abstract_widgets::VxHbvhHierarchy, prelude::*}, core::{gpu::VxGpuBackend, prelude::*},
};
use qd_varallax_widgets::{
	abstracts::value_components::VxWidgetValueExtensionExt,
	button::VxButtonWidget, layout::{VxHBoxLayoutWidget, VxVBoxLayoutWidget}, slider::VxSliderWidget, text::VxWidgetTextExtensionExt
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
		let mut v_layout = VxVBoxLayoutWidget::new(None);
		v_layout.set_padding(10.0);

		let mut slider = VxSliderWidget::new("SoundLevel", None);
		// slider.set_font(VxFont::from_family_str("kokumr", 13.0));
		slider.set_value(50.0);
		v_layout.add_widget(slider);

		for _ in 0..1 {
			let mut layout = VxHBoxLayoutWidget::new(None);
			layout.set_hbvh_hierarchy(VxHbvhHierarchy::Parent);
			for _ in 0..5 {
				let mut button = VxButtonWidget::new("", None);
				button.set_font(VxFont::from_family_str("kokumr", 13.0));
				button.signals.clicked.connect(|btn, pos| {
					btn.set_text(&format!("座標:{},{}", pos.x(), pos.y()));
				});

				layout.add_widget(button);
			}
			let layout2 = VxVBoxLayoutWidget::new(None);
			layout.add_widget(layout2);
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