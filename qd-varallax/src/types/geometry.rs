use derive_more::{Add, AddAssign, Mul, MulAssign, Div, DivAssign, Sub, SubAssign};

use crate::types::transform::VxTransform;

//vector2 -------------------------------------------------------------------------------
#[repr(C)]
#[derive(
	Clone, Copy, PartialEq, PartialOrd, Debug, Default,
	Add, Sub, Div,
	AddAssign, SubAssign, DivAssign,
)]
pub struct VxVec2 {
	x: f32,
	y: f32,
}
impl std::ops::Mul<Self> for VxVec2 {
	type Output = Self;
	fn mul(self, rhs: Self) -> Self::Output {
		Self::new(self.x() * rhs.x(), self.y() * rhs.y())
	}
}
impl std::ops::Mul<f32> for VxVec2 {
	type Output = Self;
	fn mul(self, rhs: f32) -> Self::Output {
		Self::new(self.x() * rhs, self.y() * rhs)
	}
}
impl std::ops::MulAssign<f32> for VxVec2 {
	fn mul_assign(&mut self, rhs: f32) {
		self.set_x(self.x() * rhs);
		self.set_y(self.y() * rhs);
	}
}
impl From<(f32, f32)> for VxVec2 {
	#[inline]
	fn from(value: (f32, f32)) -> Self {
		Self::new(value.0, value.1)
	}
}
impl From<[f32; 2]> for VxVec2 {
	#[inline]
	fn from(value: [f32; 2]) -> Self {
		Self::new(value[0], value[1])
	}
}


//実装ブロック
impl VxVec2 {
	//constructors
	#[inline]
	pub fn new(x: f32, y: f32) -> Self { Self { x, y } }
	#[inline]
	pub fn from_i32(x: i32, y: i32) -> Self {
		Self {
			x: x as f32,
			y: y as f32,
		}
	}
	#[inline]
	pub fn from_u32(x: u32, y: u32) -> Self {
		Self {
			x: x as f32,
			y: y as f32,
		}
	}
	
	//getters
	#[inline]
	pub fn x(&self) -> f32 { self.x }
	#[inline]
	pub fn y(&self) -> f32 { self.y }
	#[inline]
	pub fn x_i32(&self) -> i32 { self.x as i32 }
	#[inline]
	pub fn y_i32(&self) -> i32 { self.y as i32 }
	#[inline]
	pub fn x_u32(&self) -> u32 { self.x as u32 }
	#[inline]
	pub fn y_u32(&self) -> u32 { self.y as u32 }
	#[inline]
	pub fn to_tuple(&self) -> (f32, f32) { (self.x(), self.y()) }
	#[inline]
	pub fn to_tuple_i32(&self) -> (i32, i32) { (self.x_i32(), self.y_i32()) }
	#[inline]
	pub fn to_array(&self) -> [f32; 2] { [self.x(), self.y()] }
	#[inline]
	pub fn to_array_i32(&self) -> [i32; 2] { [self.x_i32(), self.y_i32()] }

	#[inline]
	pub fn length_squared(self) -> f32 {
		self.x() * self.x() + self.y() * self.y()
	}
	#[inline]
	pub fn length(self) -> f32 { self.length_squared().sqrt() }
	#[inline]
	pub fn normalize(self) -> Self {
		let len = self.length();
		if len != 0.0 {
			self / len
		} else {
			Self::default()
		}
	}
	/// このベクトルからのp2への角度を計算
	/// # Returns
	/// `f32(radian)` ラジアン度
	#[inline]
	pub fn angle_to(&self, p2: Self) -> f32 {
		let diff_x = p2.x() - self.x();
		let diff_y = p2.y() - self.y();
		diff_y.atan2(diff_x)
	}
	#[inline]
	pub fn angle_to_degree(&self, p2: Self) -> f32 {
		let rad = self.angle_to(p2);
		let deg = rad.to_degrees();
		if deg < 0.0 {
			deg + 360.0
		} else {
			deg
		}
	}

	//setters
	#[inline]
	pub fn set_x(&mut self, x: f32) { self.x = x; }
	#[inline]
	pub fn set_y(&mut self, y: f32) { self.y = y; }
	#[inline]
	pub fn with_x(mut self, x: f32) -> Self {
		self.set_x(x);
		self
	}
	#[inline]
	pub fn with_y(mut self, y: f32) -> Self {
		self.set_y(y);
		self
	}
	#[inline]
	pub fn set_x_i32(&mut self, x: i32) { self.set_x(x as f32); }
	#[inline]
	pub fn set_y_i32(&mut self, y: i32) { self.set_y(y as f32); }
	#[inline]
	pub fn with_x_i32(self, x: i32) -> Self { self.with_x(x as f32) }
	#[inline]
	pub fn with_y_i32(self, y: i32) -> Self { self.with_y(y as f32) }
	#[inline]
	pub fn set_vector(&mut self, x: f32, y: f32) {
		self.set_x(x);
		self.set_y(y);
	}
	#[inline]
	pub fn to_logical(&mut self, scale_factor: f32) -> Self {
		Self::new(self.x() / scale_factor, self.y() / scale_factor)
	}
	#[inline]
	pub fn to_local(&self, pos: Self) -> Self {
		Self::new(self.x() - pos.x(), self.y() - pos.y())
	}

	//transforms
	#[inline]
	pub fn translate(&mut self, vector: Self) {
		self.set_x(self.x() + vector.x());
		self.set_y(self.y() + vector.y());
	}
	#[inline]
	pub fn with_translate(mut self, vector: Self) -> Self {
		self.translate(vector);
		self
	}
}

//size -------------------------------------------------------------------------------
#[repr(C)]
#[derive(
	Clone, Copy, PartialEq, PartialOrd, Debug, Default,
	Add, Sub, Mul, Div,
	AddAssign, SubAssign, MulAssign, DivAssign
)]
#[mul(forward)]
#[div(forward)]
#[mul_assign(forward)]
#[div_assign(forward)]
pub struct VxSize {
	w: f32,
	h: f32,
}

pub enum VxAspectMode {
	IgnoreAspectRatio,
	KeepAspectRatio,
	KeepAspectRatioByExpanding,
}

impl From<(f32, f32)> for VxSize {
	#[inline]
	fn from(value: (f32, f32)) -> Self {
		Self::new(value.0, value.1)
	}
}
impl From<[f32; 2]> for VxSize {
	#[inline]
	fn from(value: [f32; 2]) -> Self {
		Self::new(value[0], value[1])
	}
}
impl From<(i32, i32)> for VxSize {
	#[inline]
	fn from(value: (i32, i32)) -> Self {
		Self::from_i32(value.0, value.1)
	}
}
impl From<[i32; 2]> for VxSize {
	#[inline]
	fn from(value: [i32; 2]) -> Self {
		Self::from_i32(value[0], value[1])
	}
}
impl From<(u32, u32)> for VxSize {
	#[inline]
	fn from(value: (u32, u32)) -> Self {
		Self::from_u32(value.0, value.1)
	}
}
impl From<[u32; 2]> for VxSize {
	#[inline]
	fn from(value: [u32; 2]) -> Self {
		Self::from_u32(value[0], value[1])
	}
}

//実装ブロック
impl VxSize {
	//constructs
	//From実装以外にも一応実装
	#[inline]
	pub fn new(w: f32, h: f32) -> Self { Self { w, h } }
	#[inline]
	pub fn from_i32(w: i32, h: i32) -> Self { Self { w: w as f32, h: h as f32 } }
	#[inline]
	pub fn from_u32(w: u32, h: u32) -> Self { Self { w: w as f32, h: h as f32 } }
	#[inline]
	pub fn from_vec2(vector: VxVec2) -> Self { Self { w: vector.x(), h: vector.y() } }

	//getters
	#[inline]
	pub fn width(self) -> f32 { self.w }
	#[inline]
	pub fn height(self) -> f32 { self.h }
	#[inline]
	pub fn width_i32(self) -> i32 { self.width() as i32 }
	#[inline]
	pub fn height_i32(self) -> i32 { self.height() as i32 }
	#[inline]
	pub fn width_u32(self) -> u32 { self.width() as u32 }
	#[inline]
	pub fn height_u32(self) -> u32 { self.height() as u32 }
	#[inline]
	pub fn to_tuple(&self) -> (f32, f32) { (self.width(), self.height()) }
	#[inline]
	pub fn to_array(&self) -> [f32; 2] { [self.width(), self.height()] }
	#[inline]
	pub fn to_tuple_i32(&self) -> (i32, i32) { (self.width_i32(), self.height_i32()) }
	#[inline]
	pub fn to_array_i32(&self) -> [i32; 2] { [self.width_i32(), self.height_i32()] }
	#[inline]
	pub fn to_tuple_u32(&self) -> (u32, u32) { (self.width_u32(), self.height_u32()) }
	#[inline]
	pub fn to_array_u32(&self) -> [u32; 2] { [self.width_u32(), self.height_u32()] }

	#[inline]
	pub fn is_empty(self) -> bool {
		self.width() <= 0.0 || self.height() <= 0.0
	}

	//setters
	#[inline]
	pub fn set_width(&mut self, w: f32) { self.w = w; }
	#[inline]
	pub fn set_height(&mut self, h: f32) { self.h = h; }
	#[inline]
	pub fn with_width(mut self, w: f32) -> Self {
		self.set_width(w);
		self
	}
	#[inline]
	pub fn with_height(mut self, h: f32) -> Self {
		self.set_height(h);
		self
	}
	#[inline]
	pub fn set_width_i32(&mut self, w: i32) { self.set_width(w as f32); }
	#[inline]
	pub fn set_height_i32(&mut self, h: i32) { self.set_height(h as f32); }
	#[inline]
	pub fn with_width_i32(self, w: i32) -> Self { self.with_width(w as f32) }
	#[inline]
	pub fn with_height_i32(self, h: i32) -> Self { self.with_height(h as f32) }
	#[inline]
	pub fn set_width_u32(&mut self, w: u32) { self.set_width(w as f32); }
	#[inline]
	pub fn set_height_u32(&mut self, h: u32) { self.set_height(h as f32); }
	#[inline]
	pub fn with_width_u32(self, w: u32) -> Self { self.with_width(w as f32) }
	#[inline]
	pub fn with_height_u32(self, h: u32) -> Self { self.with_height(h as f32) }
	#[inline]
	pub fn set_size(&mut self, w: f32, h: f32) {
		self.set_width(w);
		self.set_height(h);
	}

	//transfroms
	pub fn scaled(self, target: VxSize, mode: VxAspectMode) -> Self {
		match mode {
			VxAspectMode::IgnoreAspectRatio => {
				Self::new(target.width(), target.height())
			}
			VxAspectMode::KeepAspectRatio => {
				let ratio_w = target.width() / self.width();
				let ratio_h = target.height() / self.height();
				let ratio = ratio_w.min(ratio_h);
				Self::new(self.width() * ratio, self.height() * ratio)
			}
			VxAspectMode::KeepAspectRatioByExpanding => {
				let ratio_w = target.width() / self.width();
				let ratio_h = target.height() / self.height();
				let ratio = ratio_w.max(ratio_h);
				Self::new(self.width() * ratio, self.height() * ratio)
			}
		}
	}
}

//Rect ---------------------------------------------------------------------------
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd)]
pub struct VxRect {
	pos: VxVec2,
	size: VxSize,
}

impl From<(f32, f32, f32, f32)> for VxRect {
	#[inline]
	fn from(value: (f32, f32, f32, f32)) -> Self {
		Self::new(value.0, value.1, value.2, value.3)
	}
}
impl From<[f32; 4]> for VxRect {
	#[inline]
	fn from(value: [f32; 4]) -> Self {
		Self::new(value[0], value[1], value[2], value[3])
	}
}
impl From<(i32, i32, i32, i32)> for VxRect {
	#[inline]
	fn from(value: (i32, i32, i32, i32)) -> Self {
		Self::from_i32(value.0, value.1, value.2, value.3)
	}
}
impl From<[i32; 4]> for VxRect {
	#[inline]
	fn from(value: [i32; 4]) -> Self {
		Self::from_i32(value[0], value[1], value[2], value[3])
	}
}
//実装ブロック

impl VxRect {
	#[inline]
	pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
		Self {
			pos: VxVec2::new(x, y),
			size: VxSize::new(w, h),
		}
	}
	#[inline]
	pub fn from_i32(x: i32, y: i32, w: i32, h: i32) -> Self {
		Self {
			pos: VxVec2::from_i32(x, y),
			size: VxSize::from_i32(w, h),
		}
	}
	#[inline]
	pub fn from_u32(x: u32, y: u32, w: u32, h: u32) -> Self {
		Self {
			pos: VxVec2::from_u32(x, y),
			size: VxSize::from_u32(w, h),
		}
	}
	#[inline]
	pub fn from_pos_size(pos: VxVec2, size: VxSize) -> Self {
		Self {
			pos,
			size,
		}
	}
	#[inline]
	pub fn from_points(p1: VxVec2, p2: VxVec2) -> Self {
		let x = p1.x().min(p2.x());
		let y = p1.y().min(p2.y());
		let w = (p2.x() - p1.x()).abs();
		let h = (p2.y() - p1.y()).abs();
		Self::new(x, y, w, h)
	}
	
	//getters
	#[inline]
	pub fn pos(self) -> VxVec2 { self.pos }
	#[inline]
	pub fn size(self) -> VxSize { self.size }
	#[inline]
	pub fn x(self) -> f32 { self.pos().x() }
	#[inline]
	pub fn y(self) -> f32 { self.pos().y() }
	#[inline]
	pub fn width(self) -> f32 { self.size().width() }
	#[inline]
	pub fn height(self) -> f32 { self.size().height() }
	#[inline]
	pub fn half_width(self) -> f32 { self.width() / 2.0 }
	#[inline]
	pub fn half_height(self) -> f32 { self.height() / 2.0 }

	#[inline]
	pub fn left(self) -> f32 { self.x() }
	#[inline]
	pub fn right(self) -> f32 { self.x() + self.width() }
	#[inline]
	pub fn top(self) -> f32 { self.y() }
	#[inline]
	pub fn bottom(self) -> f32 { self.y() + self.height() }

	#[inline]
	pub fn left_top(&self) -> VxVec2 { self.pos() }
	#[inline]
	pub fn right_top(&self) -> VxVec2 {
		VxVec2::new(
			self.right(),
			self.top(),
		)
	}
	#[inline]
	pub fn left_bottom(&self) -> VxVec2 {
		VxVec2::new(
			self.left(),
			self.bottom(),
		)
	}
	#[inline]
	pub fn right_bottom(&self) -> VxVec2 {
		VxVec2::new(
			self.right(),
			self.bottom(),
		)
	}
	#[inline]
	pub fn top_center(&self) -> VxVec2 {
		VxVec2::new(
			self.x() + self.width() / 2.0,
			self.top(),
		)
	}
	#[inline]
	pub fn bottom_center(&self) -> VxVec2 {
		VxVec2::new(
			self.x() + self.width() / 2.0,
			self.bottom(),
		)
	}
	#[inline]
	pub fn left_center(&self) -> VxVec2 {
		VxVec2::new(
			self.left(),
			self.y() + self.height() / 2.0,
		)
	}
	#[inline]
	pub fn right_center(&self) -> VxVec2 {
		VxVec2::new(
			self.right(),
			self.y() + self.height() / 2.0,
		)
	}
	#[inline]
	pub fn center(&self) -> VxVec2 {
		VxVec2::new(
			self.top_center().x(),
			self.right_center().y(),
		)
	}
	#[inline]
	pub fn aspect_ratio(&self) -> f32 {
		self.width() / self.height()
	}

	//setters
	#[inline]
	pub fn set_x(&mut self, x: f32) { self.pos.x = x; }
	#[inline]
	pub fn set_y(&mut self, y: f32) { self.pos.y = y; }
	#[inline]
	pub fn set_width(&mut self, w: f32) { self.size.w = w; }
	#[inline]
	pub fn set_height(&mut self, h: f32) { self.size.h = h; }
	#[inline]
	pub fn with_x(mut self, x: f32) -> Self {
		self.set_x(x);
		self
	}
	#[inline]
	pub fn with_y(mut self, y: f32) -> Self {
		self.set_y(y);
		self
	}
	#[inline]
	pub fn with_width(mut self, w: f32) -> Self {
		self.set_width(w);
		self
	}
	#[inline]
	pub fn with_height(mut self, h: f32) -> Self {
		self.set_height(h);
		self
	}
	
	#[inline]
	pub fn set_pos(&mut self, pos: VxVec2) { self.pos = pos; }
	#[inline]
	pub fn set_size(&mut self, size: VxSize) { self.size = size; }
	#[inline]
	pub fn with_pos(mut self, pos: VxVec2) -> Self {
		self.pos = pos;
		self
	}
	#[inline]
	pub fn with_size(mut self, size: VxSize) -> Self {
		self.size = size;
		self
	}
	
	//transforms
	#[inline]
	pub fn translate(&mut self, vector: VxVec2) {
		self.pos += vector;
	}
	#[inline]
	pub fn with_translate(mut self, vector: VxVec2) -> Self {
		self.translate(vector);
		self
	}
	pub fn set_transform(&mut self, transform: VxTransform) {
		let scale = self.size() * transform.scale();
		let (sw, sh) = scale.to_tuple();

		let rad = transform.rotation().to_radians();
		let (sin, cos) = rad.sin_cos();

		let px = transform.pivot().x() * sw;
		let py = transform.pivot().y() * sh;

		let corners = [
			(-px, -py), // 左上
			(sw - px, -py), // 右上
			(-px, sh - py), // 左下
			(sw - px, sh - py), // 右下
		];

		let mut min_x = f32::MAX;
		let mut max_x = f32::MIN;
		let mut min_y = f32::MAX;
		let mut max_y = f32::MIN;

		for (cx, cy) in corners {
			let rx = cx * cos - cy * sin;
			let ry = cx * sin + cy * cos;

			min_x = min_x.min(rx);
			max_x = max_x.max(rx);
			min_y = min_y.min(ry);
			max_y = max_y.max(ry);
		}

		self.set_size(
			VxSize::new(
				max_x - min_x,
				max_y - min_y
			)
		);

		self.set_pos(VxVec2::new(
			transform.pos().x() + min_x,
			transform.pos().y() + min_y
		));
	}
	pub fn with_transform(self, transform: VxTransform) -> Self {
		let mut rect = Self::from_pos_size(self.pos(), self.size());
		rect.set_transform(transform);
		rect
	}

	//regions
	#[inline]
	pub fn intersect(self, other: Self) -> Option<Self> {
		let x1 = self.left().max(other.left());
		let y1 = self.top().max(other.top());
		let x2 = self.right().min(other.right());
		let y2 = self.bottom().min(other.bottom());
		
		if x1 < x2 && y1 < y2 {
			Some(Self::new(x1, y1, x2 - x1, y2 - y1))
		} else {
			None
		}
	}
	#[inline]
	pub fn intersects(self, other: Self) -> bool {
		self.left() < other.right() &&
		self.right() > other.left() &&
		self.top() < other.bottom() &&
		self.bottom() > other.top()
	}
	#[inline]
	pub fn union(self, other: Self) -> Self {
		let x1 = self.left().min(other.left());
		let y1 = self.top().min(other.top());
		let x2 = self.right().max(other.right());
		let y2 = self.bottom().max(other.bottom());

		Self::new(x1, y1, x2 - x1, y2 - y1)
	}
	#[inline]
	pub fn contains(&self, point: VxVec2) -> bool {
		point.x() >= self.left() &&
		point.x() < self.right() &&
		point.y() >= self.top() &&
		point.y() < self.bottom()
	}
	#[inline]
	pub fn encloses(&self, other: Self) -> bool {
		self.left() <= other.left() &&
		self.right() >= other.right() &&
		self.top() <= other.top() &&
		self.bottom() >= other.bottom()
	}
	#[inline]
	pub fn area(&self) -> f32 {
		self.width() * self.height()
	}
}


#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct VxRectR {
	rect: VxRect,
	corner_radius: f32,
}

impl VxRectR {
	pub fn new(rect: VxRect, corner_radius: f32) -> Self {
		Self { rect, corner_radius }
	}

	// getter
	pub fn rect(&self) -> VxRect { self.rect }
	pub fn corner_radius(&self) -> f32 { self.corner_radius }

	// setter
	pub fn set_rect(&mut self, rect: VxRect) {
		self.rect = rect;
	}
	pub fn set_corner_radius(&mut self, corner_radius: f32) {
		self.corner_radius = corner_radius;
	}
}