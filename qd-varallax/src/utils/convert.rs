use parry2d::{bounding_volume::Aabb, math::Vec2};

use crate::types::geometry::VxRect;



pub struct VxUtilConverter;

impl VxUtilConverter {
	pub fn rect_to_aabb(rect: &VxRect) -> Aabb {
		let left_top = rect.left_top();
		let right_bottom = rect.right_bottom();
		Aabb::new(
			Vec2::new(
				left_top.x(),
				left_top.y()
			),
			Vec2::new(
				right_bottom.x(),
				right_bottom.y()
			)
		)
	}
}