use std::{ops::{Add, Mul}};

/// ## QD-Varallax> types> Color> VxColorName
/// An enum for primary colors.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum VxColorName {
	/// #FFFFFF
	#[default]
	White,
	/// #000000
	Black,
	/// #FF0000
	Red,
	/// #00FF00
	Green,
	/// #0000FF
	Blue,
	/// #00FFFF
	Cyan,
	/// #FF00FF
	Magenta,
	/// #FFFF00
	Yellow,
	/// #00000000
	Transparent,
}

/// ## QD-Varallax> types> Color> VxColorChannel
/// An enum representing the layout of color channels.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum VxColorChannel {
	/// R, G, B, A
	#[default]
	RGBA,
	/// A, R, G, B
	ARGB,
	/// B, G, R, A
	BGRA,
	/// A, B, G, R
	ABGR,
	/// G, R, B, A
	GRBA,
	/// R, B, G, A
	RBGA,
}

/// ## QD-Varallax> types> Color> VxColorChannelSwap
/// An enum representing color channel swap operations.
#[derive(Clone, Copy, Debug, PartialEq, Default, Eq, Hash)]
pub enum VxColorChannelSwap {
	/// No swapping is performed. The default option.
	#[default]
	NoSwap,
	/// Swaps the R and G channels.
	SwapRG,
	/// Swaps the G and B channels.
	SwapGB,
	/// Swaps the B and A channels.
	SwapBA,
	/// Swaps the R and A channels.
	SwapRA,
	/// Swaps the R and B channels.
	SwapRB,
	/// Swaps the G and A channels.
	SwapGA,
}

/// ## QD-Varallax> types> Color> VxColor
/// A color represented by `RGBA` components, each stored as a normalized [`f32`] in the range `[0.0, 1.0]`
/// # Usage
/// ```
/// use qd_varallax::types::color::{VxColor, VxColorName};
///
/// let white = VxColor::rgb(1.0, 1.0, 1.0);
/// // Implmented `Display` trait to format as a hex string.
/// println!("color: {}", white); // "color: #ffffffff"
///
/// let cyan = VxColor::from_name(VxColorName::Cyan);
/// println!("color: {}", cyan); // "color: #00ffffff"
///
/// let red = VxColor::from_hex(0xFF0000);
/// println!("color: {}", red); // "color: #ff0000ff"
///
/// let green: VxColor = (0.0, 1.0, 0.0).into(); // Supports conversion from `tuple(f32, f32, f32)`
/// println!("color: {}", green); // "color: #00ff00ff"
///
/// // Implmented `Default` trait as (0.0, 0.0, 0.0, 1.0).
/// let black = VxColor::default();
/// println!("color: {}", black); // "color: #000000FF"
/// ```
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VxColor {
	r: f32,
	g: f32,
	b: f32,
	a: f32,
}

//from f32------------------------------------------------------------
impl From<[f32; 3]> for VxColor {
	#[inline]
	fn from([r, g, b]: [f32; 3]) -> Self {
		Self::rgb(r, g, b)
	}
}
impl From<[f32; 4]> for VxColor {
	#[inline]
	fn from([r, g, b, a]: [f32; 4]) -> Self {
		Self::rgba(r, g, b, a)
	}
}
impl From<(f32, f32, f32)> for VxColor {
	#[inline]
	fn from((r, g, b): (f32, f32, f32)) -> Self {
		Self::rgb(r, g, b)
	}
}
impl From<(f32, f32, f32, f32)> for VxColor {
	#[inline]
	fn from((r, g, b, a): (f32, f32, f32, f32)) -> Self {
		Self::rgba(r, g, b, a)
	}
}
//from u8------------------------------------------------------------
impl From<[u8; 3]> for VxColor {
	#[inline]
	fn from([r, g, b]: [u8; 3]) -> Self {
		Self::rgb_u8(r, g, b)
	}
}
impl From<[u8; 4]> for VxColor {
	#[inline]
	fn from([r, g, b, a]: [u8; 4]) -> Self {
		Self::rgba_u8(r, g, b, a)
	}
}
impl From<(u8, u8, u8)> for VxColor {
	#[inline]
	fn from((r, g, b): (u8, u8, u8)) -> Self {
		Self::rgb_u8(r, g, b)
	}
}
impl From<(u8, u8, u8, u8)> for VxColor {
	#[inline]
	fn from((r, g, b, a): (u8, u8, u8, u8)) -> Self {
		Self::rgba_u8(r, g, b, a)
	}
}
//others------------------------------------------------------------
impl From<u32> for VxColor {
	#[inline]
	fn from(value: u32) -> Self {
		Self::from_hex(value)
	}
}
impl From<VxColorName> for VxColor {
	#[inline]
	fn from(name: VxColorName) -> Self {
		Self::from_name(name)
	}
}
impl Default for VxColor {
	#[inline]
	fn default() -> Self {
		Self::rgb(0.0, 0.0, 0.0)
	}
}
//override----------------------------------------------------------
impl Add for VxColor {
	type Output = Self;
	#[inline]
	fn add(self, rhs: Self) -> Self::Output {
		Self::rgba(
			self.red() + rhs.red(),
			self.green() + rhs.green(),
			self.blue() + rhs.blue(),
			self.alpha() + rhs.alpha(),
		)
	}
}
impl Mul for VxColor {
	type Output = Self;
	#[inline]
	fn mul(self, rhs: Self) -> Self::Output {
		Self::rgba(
			self.red() * rhs.red(),
			self.green() * rhs.green(),
			self.blue() * rhs.blue(),
			self.alpha() * rhs.alpha(),
		)
	}
}
impl Mul<f32> for VxColor {
	type Output = Self;
	#[inline]
	fn mul(self, rhs: f32) -> Self::Output {
		Self::rgba(
			self.red() * rhs,
			self.green() * rhs,
			self.blue() * rhs,
			self.alpha() * rhs
		)
	}
}
impl std::fmt::Display for VxColor {
	#[inline]
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "#{:02x}{:02x}{:02x}{:02x}", self.red_u8(), self.green_u8(), self.blue_u8(), self.alpha_u8())
	}
}


impl VxColor {
	/// ## VxColor> constructor> rgb()
	/// Creates a new color from [`f32`] components.
	#[inline]
	pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
		Self {r: r, g: g, b: b, a: 1.0}
	}
	#[inline]
	/// ## VxColor> constructors> rgba()
	/// Creates a new `RGBA` color from [`f32`] components.
	pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
		Self { r: r, g: g, b: b, a: a}
	}
	#[inline]
	/// ## VxColor> constructors> rgb_u8()
	/// Creates a new color from [`u8`] components in the range `[0, 255]`.
	pub const fn rgb_u8(r: u8, g: u8, b: u8) -> Self {
		Self {
			r: r as f32 / 255.0,
			g: g as f32 / 255.0,
			b: b as f32 / 255.0,
			a: 1.0,
		}
	}
	#[inline]
	/// ## VxColor> constructors> rgba_u8()
	/// Creates a new `RGBA` color from [`u8`] components in the range `[0, 255]`.
	pub const fn rgba_u8(r: u8, g: u8, b: u8, a: u8) -> Self {
		Self {
			r: r as f32 / 255.0,
			g: g as f32 / 255.0,
			b: b as f32 / 255.0,
			a: a as f32 / 255.0,
		}
	}
	/// ## VxColor> constructors> from_name()
	/// Creates a new color from given [`VxColorName`].
	pub fn from_name(name: VxColorName) -> Self {
		let (r, g, b, a) = name_to_color(name);
		Self::rgba_u8(r, g, b, a)
	}
	/// ## VxColor> constructors> from_html()
	/// Creates a new color from `HTML` hex string. \
	/// Supports the following formats (The `'#'` prefix is optional):
	/// * `RGB` (3 digits) - `#FFF`
	/// * `RRGGBB` (6 digits) - `#00FFFF`
	/// * `RRGGBBAA` (8 digits) - `#FF00FFFF`
	pub fn from_html(hex: impl AsRef<str>) -> Result<Self, &'static str> {
		let res = html_to_color(hex.as_ref());
		res.map(|(r, g, b, a)| Self::rgba_u8(r, g, b, a))
	}
	/// ## VxColor> constructors> from_hex()
	/// Creates a new color from [`u32`] hex literal.
	/// #### Note: The `Alpha` channel is set to `1.0`.
	/// To include an alpha channel from a hex literal, use [`VxColor::from_hex_with_alpha`].
	#[inline]
	pub fn from_hex(hex: u32) -> Self {
		let (r, g, b, a) = hex_to_color(hex);
		Self::rgba_u8(r, g, b, a)
	}
	/// ## VxColor> constructors> from_hex_with_alpha()
	/// Creates a new `RGBA` color from [`u32`] hex literal.
	#[inline]
	pub fn from_hex_with_alpha(hex: u32) -> Self {
		let (r, g, b, a) = hex_to_color_with_alpha(hex);
		Self::rgba_u8(r, g, b, a)
	}
	/// ## VxColor> constructors> from_hsv()
	/// Creates a new color from `HSV` components.
	/// #### Note: `Hue`, `Saturation`, and `Value` must all be in the range `[0.0, 1.0]` (not `0-360`).
	pub fn from_hsv(h: f32, s: f32, v: f32) -> Self {
		let h = h % 1.0;
		let i = (h * 6.0).floor() as i32;
		let f = h * 6.0 - i as f32;
		let p = v * (1.0 - s);
		let q = v * (1.0 - f * s);
		let t = v * (1.0 - ( 1.0 - f) * s);
		match i % 6 {
			0 => Self::rgb(v, t, p),
			1 => Self::rgb(q, v, p),
			2 => Self::rgb(p, v, t),
			3 => Self::rgb(p, q, v),
			4 => Self::rgb(t, p, v),
			_ => Self::rgb(v, p, q),
		}
	}
	/// ## VxColor> constructors> from_vxcoloru8()
	/// Creates a new [`VxColor`] from [`VxColorU8`].
	#[inline]
	pub fn from_vxcoloru8(color: VxColorU8) -> Self {
		let (r, g, b, a) = color.to_tuple();
		Self::rgba_u8(r, g, b, a)
	}

	/// ## VxColor> getters> red()
	/// Returns the `red` component.
	#[inline]
	pub fn red(&self) -> f32 { self.r }
	/// ## VxColor> getters> green()
	/// Returns the `green` component.
	#[inline]
	pub fn green(&self) -> f32 { self.g }
	/// ## VxColor> getters> blue()
	/// Returns the `blue` component.
	#[inline]
	pub fn blue(&self) -> f32 { self.b }
	/// ## VxColor> getters> alpha()
	/// Returns the `alpha` component.
	#[inline]
	pub fn alpha(&self) -> f32 { self.a }
	/// ## VxColor> getters> to_tuple()
	/// Returns the `RGBA` components as a tuple.
	#[inline]
	pub fn to_tuple(&self) -> (f32, f32, f32, f32) { (self.red(), self.green(), self.blue(), self.alpha()) }
	/// ## VxColor> conversions> to_array()
	/// Returns the `RGB` components as an array.
	#[inline]
	pub fn to_array(&self) -> [f32; 3] { [self.red(), self.green(), self.blue()] }
	/// ## VxColor> conversions> to_array_with_alpha()
	/// Returns the `RGBA` components as an array.
	#[inline]
	pub fn to_array_with_alpha(&self) -> [f32; 4] { [self.red(), self.green(), self.blue(), self.alpha()] }

	/// ## VxColor> getters> red_u8()
	/// Returns the `red` component as [`u8`].
	#[inline]
	pub fn red_u8(&self) -> u8 { (self.r * 255.0) as u8 }
	/// ## VxColor> getters> green_u8()
	/// Returns the `green` component as [`u8`].
	#[inline]
	pub fn green_u8(&self) -> u8 { (self.g * 255.0) as u8 }
	/// ## VxColor> getters> blue_u8()
	/// Returns the `blue` component as [`u8`].
	#[inline]
	pub fn blue_u8(&self) -> u8 { (self.b * 255.0) as u8 }
	/// ## VxColor> getters> alpha_u8()
	/// Returns the `alpha` component as [`u8`].
	#[inline]
	pub fn alpha_u8(&self) -> u8 { (self.a * 255.0) as u8 }
	/// ## VxColor> getters> to_tuple_u8()
	/// Returns the `RGBA` components as a [`u8`] tuple.
	#[inline]
	pub fn to_tuple_u8(&self) -> (u8, u8, u8, u8) { (self.red_u8(), self.green_u8(), self.blue_u8(), self.alpha_u8()) }
	/// ## VxColor> conversions> to_array_u8()
	/// Returns the `RGB` components as an [`u8`] array.
	#[inline]
	pub fn to_array_u8(&self) -> [u8; 3] { [self.red_u8(), self.green_u8(), self.blue_u8()] }
	/// ## VxColor> conversions> to_array_with_alpha_u8()
	/// Returns the `RGBA` components as an [`u8`] array.
	#[inline]
	pub fn to_array_with_alpha_u8(&self) -> [u8; 4] { [self.red_u8(), self.green_u8(), self.blue_u8(), self.alpha_u8()] }
	/// ## VxColor> conversions> to_vxcoloru8()
	/// Returns the `RGBA` components as [`VxColorU8`].
	#[inline]
	pub fn to_vxcoloru8(&self) -> VxColorU8 {
		let (r, g, b, a) = self.to_tuple_u8();
		VxColorU8::rgba(r, g, b, a)
	}

	/// ## VxColor> conversions> to_hex()
	/// Returns the color as a [`u32`] in `0xRRGGBBAA` hex format.
	#[inline]
	pub fn to_hex(&self) -> u32 {
		let r = (self.r.clamp(0.0, 1.0) * 255.0).round() as u32;
		let g = (self.g.clamp(0.0, 1.0) * 255.0).round() as u32;
		let b = (self.b.clamp(0.0, 1.0) * 255.0).round() as u32;
		let a = (self.a.clamp(0.0, 1.0) * 255.0).round() as u32;

		(r << 24) | (g << 16) | (b << 8) | a
	}

	/// ## VxColor> conversions> to_hsv_tuple()
	/// Returns the color as an `HSV` tuple.
	/// #### Note: All components are normalized to the range `[0.0, 1.0]`.
	pub fn to_hsv_tuple(&self) -> (f32, f32, f32) {
		let max = self.r.max(self.g).max(self.b);
		let min = self.r.min(self.g).min(self.b);
		let delta = max - min;

		let val = max;
		let sat = if max == 0.0 { 0.0 } else { delta / max };
		let hue = {
			if delta == 0.0 {
				0.0
			} else {
				let h = match max {
					x if x == self.r => (self.g - self.b) / delta,
					x if x == self.g => (self.b - self.r) / delta + 2.0,
					_ => (self.r - self.g) / delta + 4.0,
				};
				(h * 60.0 + 360.0) % 360.0
			}
		};

		(hue / 360.0, sat, val)
	}

	/// ## VxColor> setters> set_red()
	/// Sets the `red` component.
	#[inline]
	pub fn set_red(&mut self, r: f32) { self.r = r; }
	/// ## VxColor> setters> set_green()
	/// Sets the `green` component.
	#[inline]
	pub fn set_green(&mut self, g: f32) { self.g = g; }
	/// ## VxColor> setters> set_blue()
	/// Sets the `blue` components.
	#[inline]
	pub fn set_blue(&mut self, b: f32) { self.b = b; }
	/// ## VxColor> setters> set_alpha()
	/// Sets the `alpha` component.
	#[inline]
	pub fn set_alpha(&mut self, a: f32) { self.a = a; }
	/// ## VxColor> setters> set_rgba()
	/// Sets the `RGBA` components.
	#[inline]
	pub fn set_rgba(&mut self, r: f32, g: f32, b: f32, a: f32) {
		self.set_red(r);
		self.set_green(g);
		self.set_blue(b);
		self.set_alpha(a);
	}
	/// ## VxColor> setters> set_rgba_u8()
	/// Sets the `RGBA` components from [`u8`].
	#[inline]
	pub fn set_rgba_u8(&mut self, r: u8, g: u8, b: u8, a: u8) {
		self.set_rgba(
			r as f32 / 255.0,
			g as f32 / 255.0,
			b as f32 / 255.0,
			a as f32 / 255.0,
		);
	}
	/// ## VxColor> chains> with_red()
	/// Returns a new color with the `red` component set to the given value.
	#[inline]
	pub fn with_red(mut self, r: f32) -> Self {
		self.r = r;
		self
	}
	/// ## VxColor> chains> with_green()
	/// Returns a new color with the `green` component set to the given value.
	#[inline]
	pub fn with_green(mut self, g: f32) -> Self {
		self.g = g;
		self
	}
	/// ## VxColor> chains> with_blue()
	/// Returns a new color with the `blue` component set to the given value.
	#[inline]
	pub fn with_blue(mut self, b: f32) -> Self {
		self.b = b;
		self
	}
	/// ## VxColor> chains> with_alpha()
	/// Returns a new color with the `alpha` component set to the given value.
	#[inline]
	pub fn with_alpha(mut self, a: f32) -> Self {
		self.a = a;
		self
	}

	/// ## VxColor> methods> lerp()
	/// Returns the linearly interpolates between two colors. \
	/// The `t` parameter should be in the range `[0.0, 1.0]`.
	#[inline]
	pub fn lerp(&self, rhs: &Self, t: f32) -> Self {
		Self::rgba(
			self.red() + (rhs.red() - self.red()) * t,
			self.green() + (rhs.green() - self.green()) * t,
			self.blue() + (rhs.blue() - self.blue()) * t,
			self.alpha() + (rhs.alpha() - self.alpha()) * t
		)
	}

	/// ## VxColor> methods> lerp_between()
	/// Returns the linearly interpolates between two colors. \
	/// The `t` parameters should be in the range `[0.0, 1.0]`.
	#[inline]
	pub fn lerp_between(a: &Self, b: &Self, t: f32) -> Self {
		a.lerp(b, t)
	}

	/// ## VxColor> setters> swap_channel()
	/// Swaps color channels based on the specified [`VxColorChannelSwap`] variant.
	pub fn swap_channel(&mut self, swap_mode: VxColorChannelSwap) {
		let (r, g, b, a) = swap_color(swap_mode, self.to_hex());
		self.set_rgba_u8(r, g, b, a);
	}
	/// ## VxColor> setters> set_color_channel()
	/// Sets the channel ordering of the color.
	pub fn set_color_channel(&mut self, channel_mode: VxColorChannel) {
		let (r, g, b, a) = change_channel(channel_mode, self.to_hex());
		self.set_rgba_u8(r, g, b, a);
	}

	/// ## VxColor> setters> lighten()
	/// Lightens the color by the specified amount.
	pub fn lighten(&mut self, amount: f32) {
		let (h, s, v) = self.to_hsv_tuple();
		let new_v = (v + amount).clamp(0.0, 1.0);
		let mut new_color = Self::from_hsv(h, s, new_v);
		new_color.set_alpha(self.alpha());
		*self = new_color;
	}

	/// ## VxColor> setters> darken()
	/// Darkens the color by the specified amount.
	pub fn darken(&mut self, amount: f32) {
		let (h, s, v) = self.to_hsv_tuple();
		let new_v = (v - amount).clamp(0.0, 1.0);
		let mut new_color = Self::from_hsv(h, s, new_v);
		new_color.set_alpha(self.alpha());
		*self = new_color;
	}

	/// ## VxColor> setters> with_lighen()
	/// Returns the new color lightened by the given amount.
	pub fn with_lighten(mut self, amount: f32) -> Self {
		self.lighten(amount);
		self
	}

	/// ## VxColor> setters> with_darken()
	/// Returns the new color darkened by the given amount.
	pub fn with_darken(mut self, amount: f32) -> Self {
		self.darken(amount);
		self
	}
}


/// ## QD-Varallax> types> Color> VxColorU8
/// A color represented by `RGBA` components, each stored as a normalized [`u8`] in the range `[0, 255]`
/// # Usage
/// ```
/// use qd_varallax::types::color::{VxColorU8, VxColor, VxColorName};
///
/// let red = VxColorU8::rgb(255, 0, 0);
/// // Implmented `Display` trait to format as a hex string.
/// println!("color: {}", red); // "color: #FF0000FF"
///
/// let green = VxColorU8::rgba_f(0.0, 1.0, 0.0, 1.0);
/// println!("color: {}", green); // "color: #00FF00FF"
///
/// let transparent = VxColorU8::from_vxcolor(
/// 	VxColor::from_name(
/// 		VxColorName::Transparent
/// 	)
/// );
/// println!("color: {}", transparent) // "color: #00000000"
///
/// let blue: VxColorU8 = (0, 0, 255).into(); // Supports conversion from `tuple(u8, u8, u8)`
/// println!("color: {}", blue); // "color: #0000FFFF"
///
/// // Implmented `Default` trait as (0, 0, 0, 255).
/// let black = VxColorU8::default();
/// println!("color: {}", black); // "color: #000000FF"
/// ```
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Ord, PartialOrd, bytemuck::Pod, bytemuck::Zeroable)]
pub struct VxColorU8 {
	r: u8,
	g: u8,
	b: u8,
	a: u8,
}

// fromu8 -----------------------------------
impl From<(u8, u8, u8)> for VxColorU8 {
	#[inline]
	fn from((r, g, b): (u8, u8, u8)) -> Self {
		Self::rgb(r, g, b)
	}
}
impl From<(u8, u8, u8, u8)> for VxColorU8 {
	#[inline]
	fn from((r, g, b, a): (u8, u8, u8, u8)) -> Self {
		Self::rgba(r, g, b, a)
	}
}
impl From<[u8; 3]> for VxColorU8 {
	#[inline]
	fn from([r, g, b]: [u8; 3]) -> Self {
		Self::rgb(r, g, b)
	}
}
impl From<[u8; 4]> for VxColorU8 {
	#[inline]
	fn from([r, g, b, a]: [u8; 4]) -> Self {
		Self::rgba(r, g, b, a)
	}
}
// fromf32 -----------------------------------
impl From<(f32, f32, f32)> for VxColorU8 {
	#[inline]
	fn from((r, g, b): (f32, f32, f32)) -> Self {
		Self::rgb_f(r, g, b)
	}
}
impl From<(f32, f32, f32, f32)> for VxColorU8 {
	#[inline]
	fn from((r, g, b, a): (f32, f32, f32, f32)) -> Self {
		Self::rgba_f(r, g, b, a)
	}
}
impl From<[f32; 3]> for VxColorU8 {
	#[inline]
	fn from([r, g, b]: [f32; 3]) -> Self {
		Self::rgb_f(r, g, b)
	}
}
impl From<[f32; 4]> for VxColorU8 {
	#[inline]
	fn from([r, g, b, a]: [f32; 4]) -> Self {
		Self::rgba_f(r, g, b, a)
	}
}
// others --------------------------------------
impl From<u32> for VxColorU8 {
	#[inline]
	fn from(value: u32) -> Self {
		Self::from_hex(value)
	}
}
impl From<VxColorName> for VxColorU8 {
	#[inline]
	fn from(value: VxColorName) -> Self {
		Self::from_name(value)
	}
}
impl Default for VxColorU8 {
	#[inline]
	fn default() -> Self {
		Self::rgb(0, 0, 0)
	}
}
impl std::fmt::Display for VxColorU8 {
	#[inline]
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "#{:02x}{:02x}{:02x}{:02x}", self.r, self.g, self.b, self.a)
	}
}

impl std::ops::Add for VxColorU8 {
	type Output = Self;
	#[inline]
	fn add(self, rhs: Self) -> Self::Output {
		Self::rgba(
			self.red() + rhs.red(),
			self.green() + rhs.green(),
			self.blue() + rhs.blue(),
			self.alpha() + rhs.alpha()
		)
	}
}
impl std::ops::Mul for VxColorU8 {
	type Output = Self;
	#[inline]
	fn mul(self, rhs: Self) -> Self::Output {
		Self::rgba(
			self.red() * rhs.red(),
			self.green() * rhs.green(),
			self.blue() * rhs.blue(),
			self.alpha() * rhs.alpha()
		)
	}
}
impl std::ops::Mul<f32> for VxColorU8 {
	type Output = Self;
	#[inline]
	fn mul(self, rhs: f32) -> Self::Output {
		Self::rgba_f(
			self.red_f() * rhs,
			self.green_f() * rhs,
			self.blue_f() * rhs,
			self.alpha_f() * rhs
		)
	}
}


impl VxColorU8 {
	/// ## VxColorU8> constructors> rgb()
	/// Creates a new color from [`u8`] components.
	#[inline]
	pub fn rgb(r: u8, g: u8, b: u8) -> Self {
		Self { r, g, b, a: 255 }
	}
	/// ## VxColorU8> constructors> rgba()
	/// Creates a new `RGBA` color from [`u8`] components.
	#[inline]
	pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
		Self { r, g, b, a }
	}
	/// ## VxColorU8> constructors> rgb_f()
	/// Creates a new color from [`f32`] components in the range `[0.0, 1.0]`.
	#[inline]
	pub fn rgb_f(r: f32, g: f32, b: f32) -> Self {
		Self {
			r: (r * 255.0) as u8,
			g: (g * 255.0) as u8,
			b: (b * 255.0) as u8,
			a: 255
		}
	}
	/// ## VxColorU8> constructors> rgba_f()
	/// Creates a new `RGBA` color from [`f32`] components in the range `[0.0, 1.0]`.
	#[inline]
	pub fn rgba_f(r: f32, g: f32, b: f32, a: f32) -> Self {
		Self {
			r: (r * 255.0) as u8,
			g: (g * 255.0) as u8,
			b: (b * 255.0) as u8,
			a: (a * 255.0) as u8,
		}
	}
	/// ## VxColorU8> constructors> from_name()
	/// Creates a new color from given [`VxColorName`].
	pub fn from_name(name: VxColorName) -> Self {
		let (r, g, b, a) = name_to_color(name);
		Self::rgba(r, g, b, a)
	}
	/// ## VxColorU8> constructors> from_html()
	/// Creates a new color from `HTML` hex string. \
	/// Supports the following formats (The `'#'` prefix is optional):
	/// * `RGB` (3 digits) - `#FFF`
	/// * `RRGGBB` (6 digits) - `#00FFFF`
	/// * `RRGGBBAA` (8 digits) - `#FF00FFFF`
	pub fn from_html(hex: &str) -> Result<Self, &'static str> {
		let res = html_to_color(hex);
		match res {
			Ok((r, g, b, a)) => {
				Ok(Self::rgba(r, g, b, a))
			}
			Err(reason) => return Err(reason)
		}
	}
	/// ## VxColorU8> constructors> from_hex()
	/// Creates a new color from [`u32`] hex literal.
	/// #### Note: The `Alpha` channel is set to `255`.
	/// To include an alpha channel from a hex literal, use [`VxColorU8::from_hex_with_alpha`].
	#[inline]
	pub fn from_hex(hex: u32) -> Self {
		let (r, g, b, a) = hex_to_color(hex);
		Self::rgba(r, g, b, a)
	}
	/// ## VxColorU8> constructors> from_hex_with_alpha()
	/// Creates a new `RGBA` color from [`u32`] hex literal.
	#[inline]
	pub fn from_hex_with_alpha(hex: u32) -> Self {
		let (r, g, b, a) = hex_to_color_with_alpha(hex);
		Self::rgba(r, g, b, a)
	}
	/// ## VxColorU8> constructors> from_hsv()
	/// Creates a new color from `HSV` components.
	/// #### Note: `Hue`, `Saturation`, and `Value` must all be in the range `[0.0, 1.0]` (not `0-360`).
	/// Coversion is handled internally by [`VxColor::from_hsv`].
	pub fn from_hsv(h: f32, s: f32, v: f32) -> Self {
		let color = VxColor::from_hsv(h, s, v);
		Self::from_vxcolor(color)
	}
	/// ## VxColorU8> constructors> from_vxcolor()
	/// Creates a new [`VxColorU8`] from [`VxColor`].
	#[inline]
	pub fn from_vxcolor(color: VxColor) -> Self {
		let (r, g, b, a) = color.to_tuple_u8();
		Self::rgba(r, g, b, a)
	}

	/// ## VxColorU8> getters> red()
	/// Returns the `red` component.
	#[inline]
	pub fn red(&self) -> u8 { self.r }
	/// ## VxColorU8> getters> green()
	/// Returns the `green` component.
	#[inline]
	pub fn green(&self) -> u8 { self.g }
	/// ## VxColorU8> getters> blue()
	/// Returns the `blue` component.
	#[inline]
	pub fn blue(&self) -> u8 { self.b }
	/// ## VxColor> getters> alpha()
	/// Returns the `alpha` component.
	#[inline]
	pub fn alpha(&self) -> u8 { self.a }
	/// ## VxColorU8> getters> to_tuple()
	/// Returns the `RGBA` components as a tuple.
	#[inline]
	pub fn to_tuple(&self) -> (u8, u8, u8, u8) { (self.red(), self.green(), self.blue(), self.alpha()) }
	/// ## VxColorU8> conversions> to_array()
	/// Returns the `RGB` components as an array.
	#[inline]
	pub fn to_array(&self) -> [u8; 3] { [self.red(), self.green(), self.blue()] }
	/// ## VxColorU8> conversions> to_array_with_alpha()
	/// Returns the `RGBA` components as an array.
	#[inline]
	pub fn to_array_with_alpha(&self) -> [u8; 4] { [self.red(), self.green(), self.blue(), self.alpha()] }

	/// ## VxColorU8> getters> red_f()
	/// Returns the `red` component as [`f32`].
	#[inline]
	pub fn red_f(&self) -> f32 { self.r as f32 / 255.0 }
	/// ## VxColorU8> getters> green_f()
	/// Returns the `green` component as [`f32`].
	#[inline]
	pub fn green_f(&self) -> f32 { self.g as f32 / 255.0 }
	/// ## VxColorU8> getters> blue_f()
	/// Returns the `blue` component as [`f32`].
	#[inline]
	pub fn blue_f(&self) -> f32 { self.b as f32 / 255.0 }
	/// ## VxColorU8> getters> alpha_f()
	/// Returns the `alpha` component as [`f32`].
	#[inline]
	pub fn alpha_f(&self) -> f32 { self.a as f32 / 255.0 }
	/// ## VxColorU8> getters> to_tuple_f()
	/// Returns the `RGBA` components as a [`f32`] tuple.
	#[inline]
	pub fn to_tuple_f(&self) -> (f32, f32, f32, f32) { (self.red_f(), self.green_f(), self.blue_f(), self.alpha_f()) }
	/// ## VxColorU8> conversions> to_array_with_alpha_f()
	/// Returns the `RGBA` components as an [`f32`] array.
	#[inline]
	pub fn to_array_f(&self) -> [f32; 3] { [self.red_f(), self.green_f(), self.blue_f()] }
	/// ## VxColorU8> conversions> to_array_with_alpha_f()
	/// Returns the `RGBA` components as an [`f32`] array.
	#[inline]
	pub fn to_array_with_alpha_f(&self) -> [f32; 4] { [self.red_f(), self.green_f(), self.blue_f(), self.alpha_f()] }
	/// ## VxColorU8> conversions> to_vxcolor()
	/// Returns the `RGBA` components as [`VxColor`].
	#[inline]
	pub fn to_vxcolor(&self) -> VxColor {
		let (r, g, b, a) = self.to_tuple();
		VxColor::rgba_u8(r, g, b, a)
	}
	/// ## VxColorU8> conversions> to_hex()
	/// Returns the color as a [`u32`] in `0xRRGGBBAA` hex format.
	#[inline]
	pub fn to_hex(&self) -> u32 {
		u32::from_be_bytes([self.r, self.g, self.b, self.a])
	}
	/// ## VxColor> conversions> to_hsv_tuple()
	/// Returns the color as `HSV` tuple.
	/// #### Note: All components are normalized to the range `[0.0, 1.0]`
	/// Conversion is handled internally by [`VxColor::to_hsv_tuple`].
	pub fn to_hsv_tuple(&self) -> (f32, f32, f32) {
		let color = self.to_vxcolor();
		color.to_hsv_tuple()
	}

	/// ## VxColorU8> setters> set_red()
	/// Sets the `red` component.
	#[inline]
	pub fn set_red(&mut self, r: u8) { self.r = r; }
	/// ## VxColorU8> setters> set_green()
	/// Sets the `green` component.
	#[inline]
	pub fn set_green(&mut self, g: u8) { self.g = g; }
	/// ## VxColorU8> setters> set_blue()
	/// Sets the `blue` component.
	#[inline]
	pub fn set_blue(&mut self, b: u8) { self.b = b; }
	/// ## VxColorU8> setters> set_alpha()
	/// Sets the `alpha` component.
	#[inline]
	pub fn set_alpha(&mut self, a: u8) { self.a = a; }
	/// ## VxColorU8> setters> set_rgba()
	/// Sets the `RGBA` components.
	#[inline]
	pub fn set_rgba(&mut self, r: u8, g: u8, b: u8, a: u8) {
		self.set_red(r);
		self.set_green(g);
		self.set_blue(b);
		self.set_alpha(a);
	}
	/// ## VxColorU8> chains> with_red()
	/// Returns a new color with the `red` component set to the given value.
	#[inline]
	pub fn with_red(mut self, r: u8) -> Self {
		self.r = r;
		self
	}
	/// ## VxColorU8> chains> with_green()
	/// Returns a new color with the `green` component set to the given value.
	#[inline]
	pub fn with_green(mut self, g: u8) -> Self {
		self.g = g;
		self
	}
	/// ## VxColorU8> chains> with_blue()
	/// Returns a new color with the `blue` component set to the given value.
	#[inline]
	pub fn with_blue(mut self, b: u8) -> Self {
		self.b = b;
		self
	}
	/// ## VxColorU8> chains> with_alpha()
	/// Returns a new color with the `alpha` component set to the given value.
	#[inline]
	pub fn with_alpha(mut self, a: u8) -> Self {
		self.a = a;
		self
	}

	/// ## VxColorU8> setters> swap_channel()
	/// Swaps color channels based on the specified [`VxColorChannelSwap`] variant.
	pub fn swap_channel(&mut self, swap_mode: VxColorChannelSwap) {
		let (r, g, b, a) = swap_color(swap_mode, self.to_hex());
		self.set_rgba(r, g, b, a);
	}
	/// ## VxColorU8> setters> set_color_channel()
	/// Sets the channel ordering of the color.
	pub fn set_color_channel(&mut self, channel_mode: VxColorChannel) {
		let (r, g, b, a) = change_channel(channel_mode, self.to_hex());
		self.set_rgba(r, g, b, a);
	}
}



// 共通処理
fn name_to_color(name: VxColorName) -> (u8, u8, u8, u8) {
	match name {
		VxColorName::White => (255, 255, 255, 255),
		VxColorName::Black => (0, 0, 0, 255),
		VxColorName::Red => (255, 0, 0, 255),
		VxColorName::Green => (0, 255, 0, 255),
		VxColorName::Blue => (0, 0, 255, 255),
		VxColorName::Cyan => (0, 255, 255, 255),
		VxColorName::Magenta => (255, 0, 255, 255),
		VxColorName::Yellow => (255, 255, 0, 255),
		VxColorName::Transparent => (0, 0, 0, 0),
	}
}

fn html_to_color(hex: &str) -> Result<(u8, u8, u8, u8), &'static str> {
	let hex = hex.trim_start_matches('#');

	let (r, g, b, a) = match hex.len() {
		3 => {
			let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).map_err(|_| "Invalid hex")?;
			let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).map_err(|_| "Invalid hex")?;
			let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).map_err(|_| "Invalid hex")?;
			(r, g, b, 255)
		}
		4 => {
			let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).map_err(|_| "Invalid hex")?;
			let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).map_err(|_| "Invalid hex")?;
			let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).map_err(|_| "Invalid hex")?;
			let a = u8::from_str_radix(&hex[3..4].repeat(2), 16).map_err(|_| "Invalid hex")?;
			(r, g, b, a)
		}
		6 => {
			let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| "Invalid hex")?;
			let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| "Invalid hex")?;
			let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| "Invalid hex")?;
			(r, g, b, 255)
		}
		8 => {
			let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| "Invalid hex")?;
			let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| "Invalid hex")?;
			let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| "Invalid hex")?;
			let a = u8::from_str_radix(&hex[6..8], 16).map_err(|_| "Invalid hex")?;
			(r, g, b, a)
		}
		_ => return Err("Invalid hex length"),
	};
	Ok((r, g, b, a))
}

#[inline]
fn hex_to_color(hex: u32) -> (u8, u8, u8, u8) {
	let r = ((hex >> 16) & 0xFF) as u8;
	let g = ((hex >> 8) & 0xFF) as u8;
	let b = (hex & 0xFF) as u8;
	(r, g, b, 255)
}

#[inline]
fn hex_to_color_with_alpha(hex: u32) -> (u8, u8, u8, u8) {
	(
		(hex >> 24) as u8,
		(hex >> 16) as u8,
		(hex >> 8) as u8,
		hex as u8
	)
}

fn swap_color(swap_mode: VxColorChannelSwap, hex: u32) -> (u8, u8, u8, u8) {
	let (r, g, b, a) = hex_to_color_with_alpha(hex);
	match swap_mode {
		VxColorChannelSwap::NoSwap => (r, g, b, a),
		VxColorChannelSwap::SwapRG => (g, r, b, a),
        VxColorChannelSwap::SwapGB => (r, b, g, a),
        VxColorChannelSwap::SwapRB => (b, g, r, a),
        VxColorChannelSwap::SwapRA => (a, g, b, r),
        VxColorChannelSwap::SwapGA => (r, a, b, g),
        VxColorChannelSwap::SwapBA => (r, g, a, b),
	}
}

fn change_channel(channel_mode: VxColorChannel, hex: u32) -> (u8, u8, u8, u8) {
	let (r, g, b, a) = hex_to_color_with_alpha(hex);
	match channel_mode {
		VxColorChannel::RGBA => (r, g, b, a),
		VxColorChannel::ARGB => (a, r, g, b),
		VxColorChannel::BGRA => (b, g, r, a),
		VxColorChannel::ABGR => (a, b, g, r),
		VxColorChannel::GRBA => (g, r, b, a),
		VxColorChannel::RBGA => (r, b, g, a),
	}
}