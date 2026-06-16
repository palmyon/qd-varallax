use crate::{
    abstractions::{
        abstract_widgets::VxWidget,
        abstract_windows::{
            VxWindow,
			VxWindowAttributes,
			VxWindowBuilder,
			VxWindowExt,
			VxWindowInternal,
            VxWindowStats,
        },
    },
    core::glyph::VxFont,
    types::{
        color::VxColor,
        geometry::{
			VxRect,
			VxRectR,
			VxVec2
		},
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

		let mut button = VxButtonWidget::new(
			VxRectR::new(VxRect::from_i32(0, 0, 250, 50), 10.0),
			"MyNameIs\nSoumeinatenmonnnin.",
			VxButtonStyle::new(VxThemeMode::DarkMode, VxFont::from_family_str("kokumr", 128.0)),
			None
		);
		button.set_pos(VxVec2::from_i32(500, 350));
		self.add_widget(button);

		let mut button = VxButtonWidget::new(
			VxRectR::new(VxRect::from_i32(0, 0, 250, 50), 10.0),
			"はいこんにちは。聡明な者、皆涙す。",
			VxButtonStyle::new(VxThemeMode::DarkMode, VxFont::from_family_str("kokumr", 32.0)),
			None
		);
		button.set_pos(VxVec2::from_i32(500, 550));
		self.add_widget(button);
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
