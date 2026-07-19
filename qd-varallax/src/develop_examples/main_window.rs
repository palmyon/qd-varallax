use crate::{abstractions::{abstract_widgets::VxWidget, abstract_windows::*}, core::glyph::VxFont, types::{color::VxColor, geometry::{VxRect, VxRectR}}, widgets::{button::{VxButtonStyle, VxButtonWidget}, immediate_area::VxImmediateAreaWidget, theme::VxThemeMode, vx_widgets::VxRectWidget}};
use qd_varallax_macro::VxWindowDerive;

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
			for j in 0..12 {
				let mut rect = VxRectWidget::new(
					VxRect::from_i32(0, 0, 1, 100),
					VxColor::from_hsv(i as f32 / 1920.0, 1.0, 1.0),
					None
				);
				rect.set_pos((i, j * 100).into());
				rect.set_z_value(-5);
				self.add_widget(rect);
			}
		}

		let button = VxButtonWidget::new(
			VxRectR::new(VxRect::from_i32(0, 0, 200, 50), 3.0),
			"こんにちは",
			VxButtonStyle::new(VxThemeMode::DarkMode, VxFont::from_family_str("kokumr", 30.0)),
			None
		);
		button.signals.clicked.connect(|button, pos| {
			button.set_pos(*pos);
		});
		self.add_widget(button);

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