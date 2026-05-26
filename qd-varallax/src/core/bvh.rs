use parry2d::{bounding_volume::Aabb, math::Vec2, partitioning::{Bvh, BvhWorkspace}};

use crate::{abstractions::abstract_widgets::VxWidgetId, types::geometry::VxVec2};



pub struct VxSpatialIndex {
	tree: Bvh,
	workspace: BvhWorkspace,
	indices: Vec<VxWidgetId>
}

impl VxSpatialIndex {
	pub fn new() -> Self {
		Self {
			tree: Bvh::new(),
			workspace: BvhWorkspace::default(),
			indices: vec![],
		}
	}

	pub fn rebuild_bvh(&mut self, widgets: &[(VxWidgetId, Aabb)]) {
		let aabbs: Vec<Aabb> = widgets.iter().map(|(_, aabb)| *aabb).collect();
		self.indices = widgets.iter().map(|(id, _)| *id).collect();

		self.tree = Bvh::from_leaves(Default::default(), &aabbs);
	}

	pub fn udpate_at(&mut self, id: VxWidgetId, aabb: Aabb) {
		self.tree.insert_or_update_partially(aabb, id.id.index as u32, 0.0);
	}

	pub fn finalize_update(&mut self) {
		self.tree.refit(&mut self.workspace);
		self.tree.optimize_incremental(&mut self.workspace);
	}

	pub fn hit_test(&self, point: &VxVec2) -> Vec<VxWidgetId> {
		let pos = Vec2::new(point.x(), point.y());
		let aabb = Aabb::new(pos, pos);
		self.tree.intersect_aabb(&aabb)
			.map(|id| self.indices[id as usize])
			.collect()
	}
}