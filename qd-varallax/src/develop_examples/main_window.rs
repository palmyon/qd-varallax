use crate::{
    abstractions::{
        abstract_widgets::VxWidget,
        abstract_windows::{
            VxWindow, VxWindowAttributes, VxWindowBuilder, VxWindowExt, VxWindowInternal,
            VxWindowStats,
        },
    },
    core::glyph::VxFont,
    types::{
        color::VxColor,
        geometry::{VxRect, VxRectR, VxVec2},
    },
    widgets::{
        button::{
			VxButtonStyle,
			VxButtonWidget
		},
        text::VxTextWidget,
        theme::VxThemeMode,
        vx_widgets::VxRectWidget,
    },
};
use vx_macro::VxWindowDerive;

#[derive(VxWindowDerive)]
pub struct MainWindow {
    #[vx(Stat)]
    stat: Option<VxWindowStats>,
    #[vx(WindowAttr)]
    window_attr: VxWindowAttributes,
}

impl VxWindow for MainWindow {
    fn init_event(&mut self) {
        let mut bg = VxRectWidget::new(VxRect::from_i32(0, 0, 1920, 1280), VxColor::from_hex(0x202020), None);
		bg.set_z_value(-3);
		self.add_widget(bg);

		for i in 1..1920 {
			// let mut button = VxButtonWidget::new(
			// 	VxRectR::new(
			// 		VxRect::from_i32(0, 0, 1, 100),
			// 		10.0
			// 	),
			// 	"",
			// 	VxButtonStyle::new(
			// 		VxThemeMode::CustomMode { std_color: VxColor::from_hsv(i as f32 / 1920.0, 1.0, 1.0) },
			// 		VxFont::from_family_str("kokumr", 30.0)
			// 	),
			// 	None,
			// );
			// button.set_pos(VxVec2::from_i32(i, 350));
			// self.add_widget(button);

			let mut rect = VxRectWidget::new(
				VxRect::from_i32(0, 0, 1, 1280),
				VxColor::from_hsv(i as f32 / 1920.0, 1.0, 1.0),
				None,
			);
			rect.set_pos(VxVec2::from_i32(i, 0));
			self.add_widget(rect);
		}

		let mut button = VxButtonWidget::new(
			VxRectR::new(
				VxRect::from_i32(0, 0, 200, 50),
				10.0
			),
			"",
			VxButtonStyle::new(
				VxThemeMode::DarkMode,
				VxFont::from_family_str("kokumr", 30.0)
			),
			None
		);
		button.set_pos(VxVec2::from_i32(250, 350));
		button.set_z_value(50);
		self.add_widget(button);

		let mut text = VxTextWidget::new(
			"←Clickable\n日本語も表示可能です。\nしかしながら、頂点の縁辺りに残存する何かがあって、\n改善点もあります。空白打てないし。",
			VxFont::from_family_str("kokumr", 50.0),
			VxColor::from_hex(0x000000),
			None,
		);
		text.set_pos(VxVec2::from_i32(460, 380));
		self.add_widget(text);
	}
}

impl MainWindow {
    pub fn new(attr: VxWindowAttributes) -> Self {
        Self {
            stat: None,
            window_attr: attr,
        }
    }
}
