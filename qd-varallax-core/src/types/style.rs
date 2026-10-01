use crate::types::{color::{VxColorChannel, VxColorChannelSwap}, prelude::*};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct VxSdfStyle {
	rectr: VxRectR,
	color: VxColor,
	outline_color: VxColor,
	outline_width: f32,
	blur_radius: f32,
}

impl VxSdfStyle {
	#[inline]
	pub const fn new(
		rectr: VxRectR,
		color: VxColor,
		outline_color: VxColor,
		outline_width: f32,
		blur_radius: f32,
	) -> Self {
		Self { rectr, color, outline_color, outline_width, blur_radius }
	}

	#[inline]
	pub const fn rectr(&self) -> VxRectR { self.rectr }
	#[inline]
	pub const fn color(&self) -> VxColor { self.color }
	#[inline]
	pub const fn outline_color(&self) -> VxColor { self.outline_color }
	#[inline]
	pub const fn outline_width(&self) -> f32 { self.outline_width }
	#[inline]
	pub const fn blur_radius(&self) -> f32 { self.blur_radius }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum VxThemeMode {
	Light,
	Dark,
	Custom { std_color: VxColor }
}

pub struct VxColorPalette {
	theme: VxThemeMode,

	background_primary: VxColor,
	background_secondary: VxColor,

	text_primary: VxColor,
	text_secondary: VxColor,
	text_disabled: VxColor,

	widget_primary: VxColor,
	widget_secondary: VxColor,
	widget_disabled: VxColor,

	outline: VxColor,
	accent: VxColor,
}

impl VxColorPalette {
	pub fn new(
		theme: VxThemeMode,
		background_primary: VxColor, background_secondary: VxColor,
		text_primary: VxColor, text_secondary: VxColor, text_disabled: VxColor,
		widget_primary: VxColor, widget_secondary: VxColor, widget_disabled: VxColor,
		outline: VxColor, accent: VxColor
	) -> Self {
		Self {
			theme,
			background_primary,
			background_secondary,
			text_primary,
			text_secondary,
			text_disabled,
			widget_primary,
			widget_secondary,
			widget_disabled,
			outline,
			accent
		}
	}

	#[inline]
	pub fn from_theme(theme: VxThemeMode) -> Self {
		match theme {
			VxThemeMode::Light => Self::light(),
			VxThemeMode::Dark => Self::dark(),
			VxThemeMode::Custom { std_color } => Self::custom(std_color),
		}
	}

	pub fn light() -> Self {
		Self {
			theme: VxThemeMode::Light,
			background_primary: VxColor::from_hex(0xF3F3F3),
			background_secondary: VxColor::from_hex(0xF0F0F0),
			text_primary: VxColor::from_name(VxColorName::Black),
			text_secondary: VxColor::from_hex(0x333333),
			text_disabled: VxColor::from_hex(0x666666),
			widget_primary: VxColor::from_name(VxColorName::White),
			widget_secondary: VxColor::from_hex(0x555555),
			widget_disabled: VxColor::from_hex(0x444444),
			outline: VxColor::from_hex(0x000000),
			accent: VxColor::from_hex(0x00D4FF),
		}
	}

	pub fn dark() -> Self {
		Self {
			theme: VxThemeMode::Dark,
			background_primary: VxColor::from_hex(0x1E1E1E),
			background_secondary: VxColor::from_hex(0x2D2D2D),
			text_primary: VxColor::from_name(VxColorName::White),
			text_secondary: VxColor::from_hex(0x888888),
			text_disabled: VxColor::from_hex(0x666666),
			widget_primary: VxColor::from_hex(0x2B2B2B),
			widget_secondary: VxColor::from_hex(0x2E2E2E),
			widget_disabled: VxColor::from_hex(0x101010),
			outline: VxColor::from_hex(0x3C3C3C),
			accent: VxColor::from_hex(0x00D4FF),
		}
	}

	pub fn custom(std_color: VxColor) -> Self {
		Self {
			theme: VxThemeMode::Custom { std_color },
			background_primary: std_color.with_darken(0.3),
			background_secondary: std_color.with_darken(0.2),
			text_primary: std_color.with_swap_channel(VxColorChannelSwap::SwapRB)
							.with_lighten(0.3),
			text_secondary:  std_color.with_swap_channel(VxColorChannelSwap::SwapRB)
							.with_lighten(0.5),
			text_disabled:  std_color.with_swap_channel(VxColorChannelSwap::SwapRB)
							.with_darken(0.2),
			widget_primary: std_color.with_darken(0.25),
			widget_secondary: std_color.with_darken(0.15),
			widget_disabled: std_color.with_darken(0.40),
			outline: std_color,
			accent: std_color.with_color_channel(VxColorChannel::BGRA)
		}
	}

	pub fn set_theme(&mut self, theme: VxThemeMode) {
		let palette = Self::from_theme(theme);
		self.theme = theme;
		self.background_primary = palette.background_primary();
		self.background_secondary = palette.background_secondary();
		self.text_primary = palette.text_primary();
		self.text_secondary = palette.text_secondary();
		self.text_disabled = palette.text_disabled();
		self.widget_primary = palette.widget_primary();
		self.widget_secondary = palette.widget_secondary();
		self.widget_disabled = palette.widget_disabled();
		self.outline = palette.outline();
		self.accent = palette.accent();
	}

	#[inline]
	pub fn with_theme(mut self, theme: VxThemeMode) -> Self {
		self = Self::from_theme(theme);
		self
	}
}

impl VxColorPalette {
	#[inline]
	pub const fn theme(&self) -> VxThemeMode { self.theme }
	#[inline]
	pub const fn background_primary(&self) -> VxColor { self.background_primary }
	#[inline]
	pub const fn background_secondary(&self) -> VxColor { self.background_secondary }
	#[inline]
	pub const fn text_primary(&self) -> VxColor { self.text_primary }
	#[inline]
	pub const fn text_secondary(&self) -> VxColor { self.text_secondary }
	#[inline]
	pub const fn text_disabled(&self) -> VxColor { self.text_disabled }
	#[inline]
	pub const fn widget_primary(&self) -> VxColor { self.widget_primary }
	#[inline]
	pub const fn widget_secondary(&self) -> VxColor { self.widget_secondary }
	#[inline]
	pub const fn widget_disabled(&self) -> VxColor { self.widget_disabled }
	#[inline]
	pub const fn outline(&self) -> VxColor { self.outline }
	#[inline]
	pub const fn accent(&self) -> VxColor { self.accent }
}