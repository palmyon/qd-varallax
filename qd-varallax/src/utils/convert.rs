use std::{collections::HashSet, hash::{DefaultHasher, Hash, Hasher}};

use parry2d::{bounding_volume::Aabb, math::Vec2};

use crate::types::geometry::VxRect;



pub struct VxUtilConverter;

impl VxUtilConverter {
	pub fn str_to_hash(key: &String) -> u64 {
		let mut hasher = DefaultHasher::new();
		key.hash(&mut hasher);
		hasher.finish()
	}

	pub fn str_to_unique_str(base: &str) -> String {
		let unique: HashSet<char> = base.chars().collect();
		unique.into_iter().collect()
	}

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