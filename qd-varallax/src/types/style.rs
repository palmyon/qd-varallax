use crate::types::{color::VxColor, geometry::VxRectR};

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
	pub fn new(
		rectr: VxRectR,
		color: VxColor,
		outline_color: VxColor,
		outline_width: f32,
		blur_radius: f32,
	) -> Self {
		Self {
			rectr,
			color,
			outline_color,
			outline_width,
			blur_radius
		}
	}

	#[inline]
	pub fn rectr(&self) -> VxRectR { self.rectr }
	#[inline]
	pub fn color(&self) -> VxColor { self.color }
	#[inline]
	pub fn outline_color(&self) -> VxColor { self.outline_color }
	#[inline]
	pub fn outline_width(&self) -> f32 { self.outline_width }
	#[inline]
	pub fn blur_radius(&self) -> f32 { self.blur_radius }
}