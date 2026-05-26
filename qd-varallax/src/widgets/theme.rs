use crate::types::color::VxColor;


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VxThemeMode {
	LightMode,
	DarkMode,
	CustomMode { std_color: VxColor }
}

pub struct VxTheme {
	// background colors
	bg_primary: VxColor,
	bg_secondary: VxColor,

	// text colors
	text_primary: VxColor,
	text_secondary: VxColor,
	text_disabled: VxColor,
	text_outline: VxColor,

	// widgets colors ('w' = widget)
	w_primary: VxColor,
	w_secondary: VxColor,
	w_outline: VxColor,
	w_accent: VxColor,
}

impl VxTheme {
	pub fn light() -> Self {
		Self {
			bg_primary: VxColor::from_hex(0xF3F3F3),
			bg_secondary: VxColor::from_hex(0xFFFFFF),
			text_primary: VxColor::from_hex(0x000000),
			text_secondary: VxColor::from_hex(0x333333),
			text_disabled: VxColor::from_hex(0x666666),
			text_outline: VxColor::from_hex(0x00D4FF),
			w_primary: VxColor::from_hex(0x0067C0),
			w_secondary: VxColor::from_hex(0x0077DF),
			w_outline: VxColor::from_hex(0x000000),
			w_accent: VxColor::from_hex(0x00D4FF),
		}
	}

	pub fn dark() -> Self {
		Self {
			bg_primary: VxColor::from_hex(0x1E1E1E),
			bg_secondary: VxColor::from_hex(0x2D2D2D),
			text_primary: VxColor::from_hex(0xFFFFFF),
			text_secondary: VxColor::from_hex(0x888888),
			text_disabled: VxColor::from_hex(0x666666),
			text_outline: VxColor::from_hex(0x00D4FF),
			w_primary: VxColor::from_hex(0x2B2B2B),
			w_secondary: VxColor::from_hex(0x2E2E2E),
			w_outline: VxColor::from_hex(0x3C3C3C),
			w_accent: VxColor::from_hex(0x00D4FF),
		}
	}

	// getter
	#[inline]
	pub fn bg_primary(&self) -> VxColor { self.bg_primary }
	#[inline]
	pub fn bg_secondary(&self) -> VxColor { self.bg_secondary }
	#[inline]
	pub fn text_primary(&self) -> VxColor { self.text_primary }
	#[inline]
	pub fn text_secondary(&self) -> VxColor { self.text_secondary }
	#[inline]
	pub fn text_disabled(&self) -> VxColor { self.text_disabled }
	#[inline]
	pub fn text_outline(&self) -> VxColor { self.text_outline }
	#[inline]
	pub fn w_primary(&self) -> VxColor { self.w_primary }
	#[inline]
	pub fn w_secondary(&self) -> VxColor { self.w_secondary }
	#[inline]
	pub fn w_outline(&self) -> VxColor { self.w_outline }
	#[inline]
	pub fn w_accent(&self) -> VxColor { self.w_accent }
}