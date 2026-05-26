use crate::types::geometry::VxVec2;

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