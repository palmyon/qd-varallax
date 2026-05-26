use vx_macro::VxWindowDerive;

use crate::{
	abstractions::{
		abstract_widgets::VxWidget,
		abstract_windows::{
			VxWindow, VxWindowAttributes, VxWindowBuilder, VxWindowExt, VxWindowInternal, VxWindowLayer, VxWindowStats
		}
	}, core::{glyph::VxFont, vx_event::VxEvent}, types::geometry::{
		VxRect,
		VxRectR,
		VxSize,
		VxVec2
	}, vx_connect, widgets::{
		button::{
			VxButtonStyle,
			VxButtonWidget
		}, text::VxTextWidget, theme::VxThemeMode, vx_widgets::VxRectWidget
	}
};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum VxMessageBoxButton {
	#[default]
	Ok,
	OkCancel,
	YesNo,
	YesNoCancel,
	OkCustomText { text: String },
	OkCancelCustomText { ok_text: String, cancel_text: String },
	YesNoCancelCustomText { yes_text: String, no_text: String, cancel_text: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum VxMessageBoxResult {
	Ok,
	#[default]
	Cancel,
	Yes,
	No,
	CustomResult { text: String }
}

impl From<VxMessageBoxButton> for rfd::MessageButtons {
	fn from(button: VxMessageBoxButton) -> Self {
		match button {
			VxMessageBoxButton::Ok => rfd::MessageButtons::Ok,
			VxMessageBoxButton::OkCancel => rfd::MessageButtons::OkCancel,
			VxMessageBoxButton::YesNo => rfd::MessageButtons::YesNo,
			VxMessageBoxButton::YesNoCancel => rfd::MessageButtons::YesNoCancel,
			VxMessageBoxButton::OkCustomText { text } => rfd::MessageButtons::OkCustom(text),
			VxMessageBoxButton::OkCancelCustomText { ok_text, cancel_text } => {
				rfd::MessageButtons::OkCancelCustom(ok_text, cancel_text)
			},
			VxMessageBoxButton::YesNoCancelCustomText { yes_text, no_text, cancel_text } => {
				rfd::MessageButtons::YesNoCancelCustom(yes_text, no_text, cancel_text)
			}
		}
	}
}

impl From<rfd::MessageDialogResult> for VxMessageBoxResult {
	fn from(result: rfd::MessageDialogResult) -> Self {
		match result {
			rfd::MessageDialogResult::Ok => { VxMessageBoxResult::Ok }
			rfd::MessageDialogResult::Cancel => { VxMessageBoxResult::Cancel }
			rfd::MessageDialogResult::Yes => { VxMessageBoxResult::Yes }
			rfd::MessageDialogResult::No => { VxMessageBoxResult::No }
			rfd::MessageDialogResult::Custom(text) => { VxMessageBoxResult::CustomResult { text } }
		}
	}
}

fn show_dialog(title: impl Into<String>, text: impl Into<String>, button: VxMessageBoxButton, level: rfd::MessageLevel) -> VxMessageBoxResult {
	let btn: rfd::MessageButtons = button.into();
	let res = rfd::MessageDialog::new()
		.set_title(title)
		.set_description(text)
		.set_level(level)
		.set_buttons(btn)
		.show();
	res.into()
}

/// QD-Varallax VxMessageBox> info() \
/// インフォメーションアイコンのメッセージダイアログを表示 \
/// * # Argments
/// * `title`: ダイアログのタイトル
/// * `text`: ダイアログのテキスト
/// * `button`: ダイアログに表示するボタン
///
/// # Results
/// ユーザーの入力した結果を返す。カスタムの場合、カスタムテキストが返る。
///
/// # Examples
/// ```no_run
/// let result = vx_message_box::info("Information", "Any text", VxMessageBoxButton::YesNo);
/// if result == VxMessageBoxButton::Yes {
/// 	println!("Answered [Yes]");
/// } else {
/// 	println!("Answered [No]");
/// }
/// ```
pub fn info(
	title: impl Into<String>,
	text: impl Into<String>,
	button: VxMessageBoxButton,
) -> VxMessageBoxResult {
	show_dialog(title, text, button, rfd::MessageLevel::Info)
}

/// QD-Varallax VxMessageBox> warning() \
/// 警告アイコンのメッセージダイアログを表示 \
/// * # Argments
/// * `title`: ダイアログのタイトル
/// * `text`: ダイアログのテキスト
/// * `button`: ダイアログに表示するボタン
///
/// # Results
/// ユーザーの入力した結果を返す。カスタムの場合、カスタムテキストが返る。
///
/// # Examples
/// ```no_run
/// let result = vx_message_box::warning("Information", "Any text", VxMessageBoxButton::Ok);
/// if result == VxMessageBoxButton::Yes {
/// 	println!("Answered [Ok]");
/// } else {
/// 	println!("Clicked [X]");
/// }
/// ```
pub fn warning(
	title: impl Into<String>,
	text: impl Into<String>,
	button: VxMessageBoxButton,
) -> VxMessageBoxResult {
	show_dialog(title, text, button, rfd::MessageLevel::Warning)
}

/// QD-Varallax VxMessageBox> critical() \
/// インフォメーションアイコンのメッセージダイアログを表示 \
/// * # Argments
/// * `title`: ダイアログのタイトル
/// * `text`: ダイアログのテキスト
/// * `button`: ダイアログに表示するボタン
///
/// # Results
/// ユーザーの入力した結果を返す。カスタムの場合、カスタムテキストが返る。
///
/// # Examples
/// ```no_run
/// let result = vx_message_box::critical("Information", "Any text", VxMessageBoxButton::Ok);
/// if result == VxMessageBoxButton::Yes {
/// 	println!("Answered [Ok]");
/// } else {
/// 	println!("Clicked [X]");
/// }
/// ```
pub fn critical(
	title: impl Into<String>,
	text: impl Into<String>,
	button: VxMessageBoxButton,
) -> VxMessageBoxResult {
	show_dialog(title, text, button, rfd::MessageLevel::Error)
}

#[derive(VxWindowDerive)]
pub struct VxMessageBox {
	#[vx(Stat)]
	stats: Option<VxWindowStats>,
	#[vx(WindowAttr)]
	attr: VxWindowAttributes,

	text: String,
}

impl VxWindow for VxMessageBox {
	fn init_event(&mut self) {
		self.set_fixed_size(Some((300.0, 150.0).into()));
		self.set_window_layer(VxWindowLayer::AlwaysOnTopLayer);
		self.set_window_minimizable(false);

		let bg = VxRectWidget::new(
			VxRect::from_i32(0, 0, 300, 150), 0xF3F3F3.into(), None
		);
		self.add_widget(bg);

		let mut text = VxTextWidget::new(self.text.clone(), VxFont::new("kokumr", 20.0), 0x000000.into(), None);
		text.set_pos((0.0, 30.0).into());

		self.add_widget(text);

		let mut button = VxButtonWidget::new(
			VxRectR::new(
				VxRect::from_i32(0, 0, 120, 50), 10.0
			),
			"Ok",
			VxButtonStyle::new(VxThemeMode::LightMode, VxFont::new("kokumr", 20.0)),
			None
		);
		button.set_pos(VxVec2::from_i32(120, 40));

		let stat = self.stats().as_ref().unwrap();
		let id = stat.window.id();
		let proxy = stat.proxy.clone();

		vx_connect!(button.signals.clicked, move |_, _| {
			let _ = proxy.send_event(VxEvent::CloseEvent { window_id: id });
		});

		self.add_widget(button);

	}
}

impl VxMessageBox {
	pub fn info<T: VxWindow>(
		parent: &T,
		title: impl Into<String>,
		text: impl Into<String>
	) {
		let wndw = Box::new(Self {
			stats: None,
			attr: VxWindowAttributes::new(
				title,
				VxSize::from_i32(300, 250)
			),
			text: text.into(),
		});
		parent.show(wndw);
	}
}