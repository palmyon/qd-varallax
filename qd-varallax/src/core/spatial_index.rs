use ahash::AHashMap;
use parry2d::{
	bounding_volume::Aabb,
	math::Vec2,
	partitioning::{
		Bvh,
		BvhWorkspace
	}
};

use crate::{
	types::{gen_vector::VxGenIndexConvertToRawIndex, geometry::{VxRect, VxVec2}}, utils::VxUtilConverter
};

pub struct VxSpatialIndex<T: VxGenIndexConvertToRawIndex + Clone + Copy> {
	tree: Bvh,
	workspace: BvhWorkspace,
	id_map: AHashMap<u32, T>,
}

impl<T: VxGenIndexConvertToRawIndex + Clone + Copy> VxSpatialIndex<T> {
	#[inline]
	pub fn new() -> Self {
		Self {
			tree: Bvh::new(),
			workspace: BvhWorkspace::default(),
			id_map: AHashMap::new(),
		}
	}

	#[inline]
	pub fn insert(&mut self, id: T, rect: VxRect) {
		self.tree.insert(
			VxUtilConverter::rect_to_aabb(rect),
			id.raw_index_u32()
		);
		self.id_map.insert(id.raw_index_u32(), id);
	}

	pub fn update_at(&mut self, id: T, rect: VxRect) {
		self.tree.insert_or_update_partially(
			VxUtilConverter::rect_to_aabb(rect),
			id.raw_index_u32(),
			0.0
		);
		self.id_map.insert(id.raw_index_u32(), id);
	}

	#[inline]
	pub fn optimize(&mut self) {
		self.tree.refit(&mut self.workspace);
		self.tree.optimize_incremental(&mut self.workspace);
	}

	pub fn hit_test(&self, point: VxVec2) -> Vec<T> {
		let pos = Vec2::new(point.x(), point.y());
		let point_aabb = Aabb::new(pos, pos);
		self.tree.intersect_aabb(&point_aabb)
			.filter_map(|data_id| {
				self.id_map.get(&data_id).copied()
			})
			.collect()
	}

	pub fn rebuild_bvh(&mut self, widgets: &[(T, VxRect)]) {
		let leaves_iter = widgets.iter()
			.map(|(id, rect)| {
				(id.raw_index(), VxUtilConverter::rect_to_aabb(*rect))
			}
		);
		self.tree = Bvh::from_iter(Default::default(), leaves_iter);
		self.id_map.clear();
		self.id_map.reserve(widgets.len());
		for (id, _) in widgets {
			self.id_map.insert(id.raw_index_u32(), *id);
		}

		self.tree.refit(&mut self.workspace);
	}

	pub fn remove(&mut self, id: T) {
		self.id_map.remove(&id.raw_index_u32());
		self.tree.remove(id.raw_index_u32());
	}
}