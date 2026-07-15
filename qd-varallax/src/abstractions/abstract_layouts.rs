use crate::types::geometry::{VxSize, VxVec2};

/// ## QD-Varallax> abstracts> layouts> VxAlignment
/// An enum for parent-child widget relative alignment.
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub enum VxAlignment {
	/// Aligns the child's `left-top` corner with the parents `left-top` corner. This is `Default`.
	#[default]
	LeftTop,
	/// Aligns the child's `left-center` point with the parents `left-center` point.
	LeftCenter,
	/// Aligns the child's `left-bottom` point with the parents `left-bottom` point.
	LeftBottom,
	/// Aligns the child's `top-center` point with the parents `top-center` point.
	TopCenter,
	/// Aligns the child's `center` point with the parents `center` point.
	Center,
	/// Aligns the child's `bottom-center` point with the parents `bottom-center` point.
	BottomCenter,
	/// Aligns the child's `right-top` corner with the parents `right-top` corner.
	RightTop,
	/// Aligns the child's `right-center` point with the parents `right-center` point.
	RightCenter,
	/// Aligns the child's `right-bottom` point with the parents `right-bottom` point.
	RightBottom,
	/// Aligns the child using relative `pos` with the parents `left-center` point.
	CustomAlignment { pos: VxVec2 },
}

#[derive(Clone, Copy)]
pub struct VxImmediateLayoutContext {
	cursor: VxVec2,
	available_size: VxSize,
}
impl VxImmediateLayoutContext {
	pub const fn new(available_size: VxSize) -> Self {
		Self {
			cursor: VxVec2::new(0.0, 0.0),
			available_size,
		}
	}
	#[inline]
	pub const fn cursor(&self) -> VxVec2 {
		self.cursor
	}
	#[inline]
	pub const fn set_cursor(&mut self, cursor: VxVec2) {
		self.cursor = cursor;
	}
}