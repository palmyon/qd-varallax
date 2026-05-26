use vx_macro::VxWindowDerive;
use crate::{
	abstractions::{
		abstract_widgets::VxWidget,
		abstract_windows::{
			VxWindow,
			VxWindowAttributes,
			VxWindowBuilder,
			VxWindowExt,
			VxWindowInternal,
			VxWindowStats
		}
	}, core::{
		glyph::VxFont,
		vx_event::{
			VxEventResult,
			VxKey
		}
	}, types::{
		color::VxColor, geometry::{
			VxRect,
			VxRectR,
			VxVec2
		}, texture::VxTexture
	}, widgets::{
		button::{
			VxButtonStyle,
			VxButtonWidget
		}, text::VxTextWidget, theme::VxThemeMode, vx_message_box::{
			self,
			VxMessageBox,
			VxMessageBoxButton,
			VxMessageBoxResult
		}, vx_widgets::{VxRectWidget, VxTextureWidget}
	}
};


#[derive(VxWindowDerive)]
pub struct MainWindow {
	#[vx(Stat)]
	stat: Option<VxWindowStats>,
	#[vx(WindowAttr)]
	window_attr: VxWindowAttributes,
}

impl VxWindow for MainWindow {
	fn init_event(&mut self) {
		let bg = VxRectWidget::new(VxRect::from_i32(0, 0, 1920, 1080), VxColor::from_hex(0x1E1E1E), None);
		self.add_widget(bg);

		let mut texture_w = VxTextureWidget::new(
			VxRect::from_i32(0, 0, 500, 500),
			VxTexture::from_file(std::path::Path::new("qd-varallax/src/develop_examples/stars.png")).unwrap(),
			None,
		);
		texture_w.set_pos((150.0, 300.0).into());
		self.add_widget(texture_w);
		
		// let mut button = VxButtonWidget::new(
		// 	VxRectR::new(VxRect::from_i32(0, 0, 100, 30), 10.0),
		// 	"選択してください",
		// 	VxButtonStyle::new(VxThemeMode::DarkMode, VxFont::new("kokumr", 40.0)),
		// 	None
		// );
		// button.set_z_value(1);
		// button.set_pos(VxVec2::new(200.0, 130.0));
		// self.add_widget(button);

		// let mut text = VxTextWidget::new("あいうえおかきくけこ", VxFont::new("kokumr", 32.0), VxColor::from_hex(0xFFFFFF), None);
		// text.set_pos(VxVec2::from_i32(50, 120));
		// self.add_widget(text);
	}

	fn key_press_event(&mut self, event: &crate::core::vx_event::VxKeyEvent) -> crate::core::vx_event::VxEventResult {
		match event.key() {
			VxKey::F11 => {
				if self.is_fullscreen() {
					self.show_normal();
				} else {
					self.show_fullscreen();
				}
				return VxEventResult::Accept;
			},
			VxKey::Escape => {
				let res = vx_message_box::info(
					"確認",
					"終了しますか?",
					VxMessageBoxButton::Ok,
				);
				if res == VxMessageBoxResult::Ok {
					self.close();
					return VxEventResult::Accept;
				}
			},
			VxKey::Enter => {
				VxMessageBox::info(self, "情報", "はいこんにちは");
			}
			_ => {}
		}
		VxEventResult::Ignore
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