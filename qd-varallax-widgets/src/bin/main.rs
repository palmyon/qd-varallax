#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use qd_varallax_macro::VxWindowImpl;
use qd_varallax_core::{
	abstracts::prelude::*,
	core::prelude::*,
};
use qd_varallax_widgets::{
	button::VxButtonWidget, layout::VxHBoxLayoutWidget, text::VxTextWidget
};

#[derive(VxWindowImpl)]
pub struct DemoWindow {
	#[attr(stat)]
	stats: Option<VxWindowStats>,
	#[attr(w_attr)]
	attr: VxWindowAttributes,
}

impl DemoWindow {
	pub fn new(attr: VxWindowAttributes) -> Self {
		Self { stats: None, attr }
	}
}
impl VxWindow for DemoWindow {
	fn init_event(&mut self) {
		let mut layout = VxHBoxLayoutWidget::new(None);
		layout.set_padding(10.0);
	
		let button = VxButtonWidget::new("VxHBoxLayoutWidget", None);
		layout.add_widget(button);
		let mut text = VxTextWidget::new("やっほー！", None);
		text.set_font(VxFont::from_family_str("kokumr", 13.0));
		layout.add_widget(text);
		self.add_widget(layout);
	}
}



fn main() {
	let mut app = VxApplication::new();
	let window = DemoWindow::new(Default::default());
	app.add_window(window);
	app.exec();
}