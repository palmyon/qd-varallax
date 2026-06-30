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
pub struct DemoWindow {
    #[vx(Stat)]
    stat: Option<VxWindowStats>,
    #[vx(WindowAttr)]
    window_attr: VxWindowAttributes,
}

impl VxWindow for DemoWindow {
    fn init_event(&mut self) {
        let mut bg = VxRectWidget::new(VxRect::from_i32(0, 0, 1920, 1280), VxColor::from_hex(0x202020), None);
		bg.set_z_value(-3);
		self.add_widget(bg);

		let mut button = VxButtonWidget::new(
			VxRectR::new(VxRect::from_i32(0, 0, 250, 50), 10.0),
			"",
			VxButtonStyle::new(VxThemeMode::DarkMode, VxFont::from_family_str("kokumr", 32.0)),
			None
		);
		button.set_pos(VxVec2::from_i32(500, 350));
		let button = self.add_widget(button);

		let mut text = VxTextWidget::new(
			"聡明な皆さん",
			VxFont::from_family_str("kokumr", 32.0),
			VxColor::from_hex(0x000000),
			None,
		);
		text.set_pos(VxVec2::from_i32(750, 340));
		self.add_widget(text);

		let btn = self.get_widget(button.unwrap()).unwrap();
		btn.signals.clicked.connect(|_, _| {
			println!("ボタンをマウスで押し込め！！！！ふん！！！！！")
		});
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
