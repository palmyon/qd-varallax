use crate::{abstractions::{abstract_widgets::VxWidget, abstract_windows::*}, core::glyph::VxFont, types::geometry::{VxRect, VxRectR}, widgets::{button::{VxButtonStyle, VxButtonWidget}, theme::VxThemeMode}};
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
		let mut button = VxButtonWidget::new(
			VxRectR::new(VxRect::from_i32(0, 0, 300, 50), 10.0),
			"おい、ウィジェットを置け",
			VxButtonStyle::new(VxThemeMode::DarkMode, VxFont::from_family_str("kokumr", 32.0)),
			None,
		);
		button.set_pos((300.0, 250.0).into());
		self.add_widget(button);
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

