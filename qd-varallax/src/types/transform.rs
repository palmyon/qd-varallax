use derive_more::{Add, AddAssign, Div, DivAssign, MulAssign, Sub, SubAssign};

use crate::types::geometry::{VxSize, VxVec2};


#[derive(
	Debug, Clone, Copy, PartialEq,
	Add, Sub, Div,
	AddAssign, SubAssign, MulAssign, DivAssign
)]
pub struct VxAngle {
	radians: f32,
}

impl From<f32> for VxAngle {
	#[inline]
	fn from(value: f32) -> Self {
		Self::from_degrees(value)
	}
}

impl Default for VxAngle {
	#[inline]
	fn default() -> Self {
		Self::RIGHT
	}
}
impl std::fmt::Display for VxAngle {
	#[inline]
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:.1}°", self.to_degrees())
	}
}

impl std::ops::Mul<f32> for VxAngle {
	type Output = Self;
	#[inline]
	fn mul(self, rhs: f32) -> Self::Output {
		Self::from_radians(self.radians * rhs)
	}
}

impl VxAngle {
	pub const RIGHT: Self = Self { radians: 0.0 };
	pub const UP: Self = Self { radians: std::f32::consts::FRAC_PI_2 };
	pub const LEFT: Self = Self { radians: std::f32::consts::PI };
	pub const DOWN: Self = Self { radians: 3.0 * std::f32::consts::FRAC_PI_2 };

	/// 定数π
	pub const PI: f32 = std::f32::consts::PI;
	pub const TWO_PI: f32 = std::f32::consts::PI * 2.0;
	pub const HALF_PI: f32 = std::f32::consts::FRAC_PI_2;

	#[inline]
	pub fn from_degrees(degrees: f32) -> Self { Self { radians: degrees.to_radians() } }
	#[inline]
	pub fn from_radians(radians: f32) -> Self { Self { radians } }
	#[inline]
	pub fn from_degrees_i32(degrees: i32) -> Self { Self::from_degrees(degrees as f32) }
	#[inline]
	pub fn from_vector(vector: VxVec2) -> Self {
		Self::from_radians(vector.y().atan2(vector.x()))
	}
	#[inline]
	pub fn from_points(from: VxVec2, to: VxVec2) -> Self {
		let rad = from.angle_to(to);
		Self::from_radians(rad)
	}
	/// 回転数から作成
	#[inline]
	pub fn from_turns(turns: f32) -> Self {
		Self::from_degrees(360.0 * turns)
	}
	/// 0.0 ~ 100.0、自動クランプ
	#[inline]
	pub fn from_percent(percent: f32) -> Self {
		let scale = percent.clamp(0.0, 100.0) / 100.0;
		Self::from_degrees(360.0 * scale)
	}

	// getters
	// degrees
	#[inline]
	pub fn to_degrees(self) -> f32 { self.radians.to_degrees() }
	#[inline]
	pub fn to_degrees_i32(self) -> i32 { self.to_degrees() as i32 }
	#[inline]
	pub fn to_degrees_abs(self) -> f32 { self.to_degrees().abs() }
	#[inline]
	pub fn to_degrees_abs_i32(self) -> i32 { self.to_degrees_abs() as i32 }
	#[inline]
	pub fn to_invert_degrees(self) -> f32 { self.to_invert_radians().to_degrees() }
	#[inline]
	pub fn to_invert_degrees_i32(self) -> i32 { self.to_invert_degrees() as i32 }
	#[inline]
	pub fn to_invert_degrees_abs(self) -> f32 { self.to_invert_degrees().abs() }
	#[inline]
	pub fn to_invert_degrees_abs_i32(self) -> i32 { self.to_invert_degrees_abs() as i32 }

	//radians
	#[inline]
	pub fn to_radians(self) -> f32 { self.radians }
	#[inline]
	pub fn to_radians_abs(self) -> f32 { self.radians.abs() }
	#[inline]
	pub fn to_invert_radians(self) -> f32 {
		(self.radians + Self::PI).rem_euclid(Self::TWO_PI)
	}

	#[inline]
	pub fn to_angle_vector(self) -> VxVec2 {
		VxVec2::new(self.radians.cos(), self.radians.sin())
	}
	#[inline]
	pub fn to_turns(self) -> f32 {
		let deg = self.to_degrees();
		deg / 360.0
	}
	#[inline]
	pub fn to_percent(self) -> f32 {
		let turn = self.to_turns();
		(turn * 100.0).clamp(0.0, 100.0)
	}
	#[inline]
	pub fn normalize(self) -> Self {
		Self::from_radians(self.radians.rem_euclid(Self::TWO_PI))
	}
	#[inline]
	pub fn normalize_signed(mut self) -> Self {
		let pi = Self::PI;
		self = self.normalize();
		if self.radians >= pi {
			self.radians -= Self::TWO_PI;
		}
		self
	}

	// setters
	#[inline]
	pub fn set_angle_degrees(&mut self, new_degrees: f32) {
		self.radians = new_degrees.to_radians();
	}
	#[inline]
	pub fn set_angle_degrees_i32(&mut self, new_degrees: i32) {
		self.set_angle_degrees(new_degrees as f32);
	}
	#[inline]
	pub fn set_angle_radians(&mut self, new_radians: f32) {
		self.radians = new_radians;
	}
	#[inline]
	pub fn invert(&mut self) {
		self.radians = self.to_invert_radians();
	}

	#[inline]
	pub fn with_angle_degrees(&self, new_degrees: f32) -> Self {
		Self::from_degrees(new_degrees)
	}
	#[inline]
	pub fn with_angle_radians(self, new_radians: f32) -> Self {
		Self::from_radians(new_radians)
	}
	#[inline]
	pub fn with_invert_angle(&self) -> Self {
		Self::from_radians(self.to_invert_radians())
	}

	#[inline]
	pub fn angle_between(self, other: Self) -> Self {
		let pi = Self::PI;
		let diff = (other.to_radians() - self.radians + pi).rem_euclid(Self::TWO_PI) - pi;
		Self::from_radians(diff)
	}
	#[inline]
	pub fn lerp_shortest(self, other: Self, t: f32) -> Self {
		let pi = Self::PI;
		let diff = (other.to_radians() - self.radians + pi)
			.rem_euclid(Self::TWO_PI) - pi;
		Self::from_radians(self.radians + diff * t)
	}

	#[inline]
	pub fn is_between(self, start: Self, end: Self) -> bool {
		let s = start.normalize().to_radians();
		let e = end.normalize().to_radians();
		let target = self.normalize().to_radians();

		if s <= e {
			target >= s && target <= e
		} else {
			target >= s || target <= e
		}
	}
}


#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VxMatrix3x3 {
	matrix: [[f32; 3]; 3],
}

impl Default for VxMatrix3x3 {
	#[inline]
	fn default() -> Self {
		Self::identity()
	}
}

impl std::ops::Mul for VxMatrix3x3 {
	type Output = Self;
	#[inline]
	fn mul(self, rhs: Self) -> Self::Output {
		let mut res = [[0.0_f32; 3]; 3];
		for i in 0..3 {
			for j in 0..3 {
				for k in 0..3 {
					res[i][j] += self.matrix[i][k] * rhs.matrix[k][j];
				}
			}
		}
		VxMatrix3x3::new(res)
	}
}

impl VxMatrix3x3 {
	#[inline]
	pub fn new(matrix: [[f32; 3]; 3]) -> Self {
		Self { matrix }
	}
	#[inline]
	pub fn identity() -> Self {
		Self {
			matrix: [
				[1.0, 0.0, 0.0],
				[0.0, 1.0, 0.0],
				[0.0, 0.0, 1.0],
			]
		}
	}
	pub fn from_transform(transform: &VxTransform) -> Self {
		let (sin, cos) = transform.rotation().to_radians().sin_cos();
		let x = transform.pos().x();
		let y = transform.pos().y();
		let sx = transform.scale().width();
		let sy = transform.scale().height();
		let px = transform.pivot().x() * transform.size().width() * sx;
		let py = transform.pivot().y() * transform.size().height() * sy;

		let m00 = cos * sx;
		let m01 = -sin * sy;
		let m10 = sin * sx;
		let m11 = cos * sy;

		let m02 = x - (m00 * px + m01 * py);
		let m12 = y - (m10 * px + m11 * py);
		
		Self::new([
			[m00, m01, m02],
			[m10, m11, m12],
			[0.0, 0.0, 1.0],
		])
	}
	#[inline]
	pub fn transform_point(&self, point: VxVec2) -> VxVec2 {
		let x = point.x();
		let y = point.y();

		let nx = self.matrix[0][0] * x + self.matrix[0][1] * y + self.matrix[0][2];
		let ny = self.matrix[1][0] * x + self.matrix[1][1] * y + self.matrix[1][2];

		VxVec2::new(nx, ny)
	}

	#[inline]
	pub fn matrix(&self) -> [[f32; 3]; 3] {
		self.matrix
	}
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, bytemuck::Pod, bytemuck::Zeroable)]
pub struct VxMatrix4x4 {
	matrix: [[f32; 4]; 4],
}

impl std::ops::Mul for VxMatrix4x4 {
	type Output = Self;
	fn mul(self, rhs: Self) -> Self::Output {
		let mut res = [[0.0; 4]; 4];
		for i in 0..4 {
			for j in 0..4 {
				res[i][j] = self.matrix[0][j] * rhs.matrix[i][0] +
							self.matrix[1][j] * rhs.matrix[i][1] +
							self.matrix[2][j] * rhs.matrix[i][2] +
							self.matrix[3][j] * rhs.matrix[i][3];
			}
		}
		Self { matrix: res }
	}
}

impl VxMatrix4x4 {
	pub fn from_3x3(m3x3: &VxMatrix3x3) -> Self {
		let m = m3x3.matrix();
		Self {
			matrix: [
				[m[0][0], m[1][1], 0.0, 0.0],
				[m[0][1], m[1][1], 0.0, 0.0],
				[0.0, 0.0, 1.0, 0.0],
				[m[0][2], m[1][2], 0.0, 1.0],
			]
		}
	}

	pub fn orthographic(size: VxSize) -> Self {
		let sx = 2.0 / size.width();
		let sy = -2.0 / size.height();

		Self {
			matrix: [
				[sx, 0.0, 0.0, 0.0],
				[0.0, sy, 0.0, 0.0],
				[0.0, 0.0, 1.0, 0.0],
				[-1.0, 1.0, 0.0, 1.0],
			]
		}
	}

	pub fn matrix(&self) -> [[f32; 4]; 4] {
		self.matrix
	}
}



#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VxTransform {
	pos: VxVec2,
	size: VxSize,
	scale: VxSize,
	rotation: VxAngle,
	pivot: VxVec2,
}

impl Default for VxTransform {
	#[inline]
	fn default() -> Self {
		Self::new(
			VxVec2::default(),
			VxSize::new(0.0, 0.0),
			VxSize::new(1.0, 1.0),
			VxAngle::default(),
			VxVec2::default(),
		)
	}
}


impl VxTransform {
	#[inline]
	pub fn new(
		pos: VxVec2,
		size: VxSize,
		scale: VxSize,
		rotation: VxAngle,
		pivot: VxVec2
	) -> Self {
		Self{
			pos,
			size,
			scale,
			rotation,
			pivot
		}
	}

	// getters
	#[inline]
	pub fn pos(self) -> VxVec2 { self.pos }
	#[inline]
	pub fn size(self) -> VxSize { self.size }
	#[inline]
	pub fn scale(self) -> VxSize { self.scale }
	#[inline]
	pub fn rotation(self) -> VxAngle { self.rotation }
	#[inline]
	pub fn pivot(self) -> VxVec2 { self.pivot }

	#[inline]
	pub fn set_pos(&mut self, pos: VxVec2) {
		self.pos = pos;
	}
	#[inline]
	pub fn set_size(&mut self, size: VxSize) {
		self.size = size;
	}
	#[inline]
	pub fn set_scale(&mut self, scale: VxSize) {
		self.scale = scale;
	}
	#[inline]
	pub fn set_rotation(&mut self, rotation: VxAngle) {
		self.rotation = rotation;
	}

	#[inline]
	pub fn translate(&mut self, vector: VxVec2) {
		self.pos.translate(vector);
	}
	#[inline]
	pub fn scale_by(&mut self, scale: VxSize) {
		self.scale.set_size(
			self.scale.width() * scale.width(),
			self.scale.height() * scale.height()
		);
	}
	#[inline]
	pub fn rotate(&mut self, rotation: VxAngle) {
		self.rotation += rotation;
	}
	#[inline]
	pub fn set_center_pivot(&mut self, pivot: VxVec2) {
		self.pivot = pivot;
	}
}