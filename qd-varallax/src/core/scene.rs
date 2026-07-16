use std::{cell::RefCell, rc::Rc};

use ahash::AHashMap;

use crate::{
	abstractions::{
		abstract_layouts::VxAlignment, abstract_widgets::{
			VxDirtyCommandSender, VxDirtyFlag, VxSpatialHierarchyFlag, VxWidget, VxWidgetHandler, VxWidgetId
		}
	}, core::{
		resource::VxAppResource, spatial_index::VxSpatialIndex
	}, painter::painter::VxPainter, types::{
		event::{VxEventResult, VxKeyEvent, VxMouseEvent}, gen_vector::VxGenVector, geometry::{
			VxRect,
			VxSize,
			VxVec2
		}, input::VxInputState, render_commands::{VxDirtyCheckResult, VxRenderMode}, transform::VxTransform
	}, utils::VxUtilConverter,
};


pub struct VxSceneOld {
	widgets: VxGenVector<Box<dyn VxWidget>>,
	top_level_widgets: Vec<VxWidgetId>,

	spatial_index: VxSpatialIndex,

	current_selected_widget: Option<VxWidgetId>,
	current_hovered_widget: Option<VxWidgetId>,
}

impl VxSceneOld {
	pub fn new() -> Self {
		Self {
			widgets: VxGenVector::new(),
			top_level_widgets: Vec::new(),
			spatial_index: VxSpatialIndex::new(),
			current_selected_widget: None,
			current_hovered_widget: None,
		}
	}

	// private methods
	fn paint_widget(
		&mut self,
		res: &mut VxAppResource,
		id: VxWidgetId,
		painter: &mut VxPainter,
	) {
		let Some(widget) = self.widgets.get_mut(id.id()) else { return; };
		if !widget.is_visible() {
			return;
		}

		widget.register_texture_event(&res.gpu, &mut res.textures);

		widget.paint(painter);
		painter.set_vertex_z_value(widget.z_value());
		widget.create_bounding_rect_event(&res.fonts);
		let pos = widget.pos();
		let z = widget.z_value();
		let bounding_rect = widget.bounding_rect();

		let children = widget.children();
		
		for child in children.clone() {
			let Some(c) = self.widgets.get_mut(child.id()) else { continue; };
			c.set_z_value(z + 1);
			painter.push_tranform(
				VxTransform::new(
					Self::apply_alignment_to_rect(pos, c.bounding_rect(), c.stats().alignment(), bounding_rect),
					VxSize::default(),
					VxSize::new(1.0, 1.0),
					0.0.into(),
					VxVec2::default()
			));
			self.paint_widget(res, child, painter);
			painter.pop_transform();
		}
	}

	fn apply_alignment_to_rect(pos: VxVec2, rect: VxRect, alignment: VxAlignment, bounding_rect: VxRect) -> VxVec2 {
		match alignment {
			VxAlignment::LeftTop => pos,
			VxAlignment::LeftCenter => {
				bounding_rect.left_center() - rect.left_center()
			}
			VxAlignment::LeftBottom => {
				bounding_rect.left_bottom() - rect.left_bottom()
			}
			VxAlignment::TopCenter => {
				bounding_rect.top_center() - rect.top_center()
			}
			VxAlignment::Center => {
				bounding_rect.top_center() - rect.top_center()
			}
			VxAlignment::BottomCenter => {
				bounding_rect.bottom_center() - rect.bottom_center()
			}
			VxAlignment::RightTop => {
				bounding_rect.right_top() - rect.right_top()
			}
			VxAlignment::RightCenter => {
				bounding_rect.right_center() - rect.right_center()
			}
			VxAlignment::RightBottom => {
				bounding_rect.right_bottom() - rect.right_bottom()
			}
			VxAlignment::CustomAlignment { pos } => {
				bounding_rect.left_top() - pos
			}
		}
	}

	// 指定場所にウィジェットがあるか特定
	fn find_widget_at(&mut self, pos: VxVec2) -> Option<VxWidgetId> {
		let result = self.spatial_index.hit_test(pos);
		if result.is_empty() { return None; }

		result.into_iter()
			.filter_map(|id| {
				let widget = self.widgets.get(id.id())?;
				if widget.is_visible() && widget.bounding_rect().contains(pos) {
					Some((widget.z_value(), id))
				} else {
					None
				}
			})
			.max_by_key(|(z, _)| *z)
			.map(|(_, id)| id)
	}

	// pub(crate) fn check_dirty(&mut self) -> bool {
	// 	let mut dirty = false;
	// 	for (id, widget) in self.widgets.iter_with_id_mut() {
	// 		if widget.dirty_flag() != VxDirtyFlag::Clean {
	// 			dirty = true;
	// 			widget.set_dirty_flag(VxDirtyFlag::Clean);
	// 			self.spatial_index.update_at(
	// 				VxWidgetId::new(id),
	// 				VxUtilConverter::rect_to_aabb(widget.bounding_rect())
	// 			);
	// 		}
	// 	}

	// 	if dirty {
	// 		self.spatial_index.optimize_incremental();
	// 	}

	// 	dirty
	// }

	// methods
	pub fn add_widget(&mut self, mut widget: Box<dyn VxWidget>) -> VxWidgetId {
		let children = widget.stats_mut().children_widgets();

		let id = VxWidgetId::new(self.widgets.insert(widget));
		let widget_ref = self.widgets.get_mut(id.id()).unwrap();
		widget_ref.stats_mut().set_widget_id(id);

		if widget_ref.parent().is_none() {
			self.top_level_widgets.push(id);
		}

		for mut w in children {
			w.set_parent(id);
			let child_id = self.add_widget(w);

			if let Some(parent) = self.widgets.get_mut(id.id()) {
				parent.stats_mut().add_child(child_id);
			}
		}

		id
	}

	pub fn get_widget<W: VxWidget>(&self, widget: VxWidgetHandler<W>) -> Option<&W> {
		self.widgets.get(widget.id().id())?
			.as_any()
			.downcast_ref::<W>()
	}
	pub fn get_widget_mut<W: VxWidget>(&mut self, widget: VxWidgetHandler<W>) -> Option<&mut W> {
		self.widgets.get_mut(widget.id().id())?
			.as_any_mut()
			.downcast_mut::<W>()
	}

	pub(crate) fn refresh_spatial_index(&mut self) {
		let mut data = vec![];
		for (index, widget) in self.widgets.iter_with_id() {
			let rect = widget.bounding_rect();
			let aabb = VxUtilConverter::rect_to_aabb(rect);
			data.push((VxWidgetId::new(index), aabb));
		}
		self.spatial_index.rebuild_bvh(&data);
	}

	// イベントハンドラー
	fn send_event_to_widget<E>(
		&mut self,
		start_id: Option<VxWidgetId>,
		event: &E,
		handler: impl Fn(&mut Box<dyn VxWidget>, &E) -> VxEventResult
	) -> VxEventResult {
		let mut current_id = start_id;

		while let Some(id) = current_id {
			let widget = self.widgets.get_mut(id.id()).unwrap();
			let result = handler(widget, event);
			if result == VxEventResult::Accept {
				return VxEventResult::Accept;
			}
			current_id = self.widgets.get(id.id()).unwrap().parent();
		}
		VxEventResult::Ignore
	}
	// Events
	pub fn paint_event(
		&mut self,
		res: &mut VxAppResource,
		painter: &mut VxPainter,
	) {
		for id in self.top_level_widgets.clone() {
			self.paint_widget(res, id, painter);
		};
	}
	// MouseEvents
	pub fn mouse_press_event(&mut self, event: &VxMouseEvent)  -> VxEventResult {
		let current_id = self.find_widget_at(event.pos());
		self.current_selected_widget = current_id;
		self.send_event_to_widget(
			current_id,
			event,
			|w, e | w.mouse_press_event(e)
		)
	}
	pub fn mouse_release_event(&mut self, event: &VxMouseEvent) -> VxEventResult {
		let current_id = self.find_widget_at(event.pos());
		self.current_selected_widget = current_id;
		self.send_event_to_widget(
			current_id,
			event,
			|w, e| w.mouse_release_event(e)
		)
	}
	pub fn mouse_move_event(&mut self, event: &VxMouseEvent) -> VxEventResult {
		let new_hover_id = self.find_widget_at(event.pos());
		let old_hover_id = self.current_hovered_widget;

		if new_hover_id != old_hover_id {
			if let Some(old_id) = old_hover_id {
				self.send_event_to_widget(
					Some(old_id),
					event,
					|w, e| w.mouse_leave_event(e)
				);
			}
			if let Some(new_id) = new_hover_id {
				self.send_event_to_widget(
					Some(new_id),
					event,
					|w, e| w.mouse_enter_event(e)
				);
			}
			self.current_hovered_widget = new_hover_id;
		}

		self.send_event_to_widget(
			new_hover_id,
			event,
			|w, e| w.mouse_move_event(e)
		)
	}
	pub fn mouse_wheel_event(&mut self, event: &VxMouseEvent) -> VxEventResult {
		let current_id = self.find_widget_at(event.pos());
		self.send_event_to_widget(
			current_id,
			event,
			|w, e| w.mouse_wheel_event(e)
		)
	}
	// KeyboardEvents
	pub fn key_press_event(&mut self, event: &VxKeyEvent) -> VxEventResult {
		let current_id = self.current_selected_widget;
		self.send_event_to_widget(
			current_id,
			event,
			|w, e| w.key_press_event(e)
		)
	}
	pub fn key_release_event(&mut self, event: &VxKeyEvent) -> VxEventResult {
		let current_id = self.current_selected_widget;
		self.send_event_to_widget(
			current_id,
			event,
			|w, e| w.key_release_event(e)
		)
	}
}

pub struct VxScene {
	widgets: VxGenVector<Box<dyn VxWidget>>,
	top_level_widgets: Vec<VxWidgetId>,
	current_selected_widgets: Option<VxWidgetId>,
	current_hovered_widgets: Option<VxWidgetId>,
	dirty_command_sender: VxDirtyCommandSender,
	dirty_queue: Rc<RefCell<Vec<(VxWidgetId, VxDirtyFlag)>>>,

	flat_spatial_index: VxSpatialIndex,
	hierarchical_spatial_index: AHashMap<VxWidgetId, VxSpatialIndex>,
}

impl VxScene {
	pub fn new() -> Self {
		let dirty_queue = Rc::new(RefCell::new(Vec::new()));
		let queue_clone = dirty_queue.clone();
		Self {
			widgets: VxGenVector::new(),
			top_level_widgets: Vec::new(),
			current_selected_widgets: None,
			current_hovered_widgets: None,
			dirty_command_sender: VxDirtyCommandSender::new(move |id, flag| {
				queue_clone.borrow_mut().push((id, flag));
			}),
			dirty_queue,
			flat_spatial_index: VxSpatialIndex::new(),
			hierarchical_spatial_index: AHashMap::new(),
		}
	}
	pub fn paint_event(&mut self, res: &mut VxAppResource, painter: &mut VxPainter) {
		self.top_level_widgets.iter().for_each(|id| {
			Self::paint_widget(&mut self.widgets, res, painter, id.clone());
		});
	}
	pub fn immediate_paint_event(&mut self, res: &mut VxAppResource, input: &VxInputState, painter: &mut VxPainter) {
		self.top_level_widgets.iter().for_each(|id| {
			Self::immediate_paint_widget(&mut self.widgets, res, input, painter, id.clone());
		});
	}
	fn paint_widget(
		widgets: &mut VxGenVector<Box<dyn VxWidget>>,
		res: &mut VxAppResource,
		painter: &mut VxPainter,
		id: VxWidgetId
	) {
		let Some(widget) = widgets.get_mut(id.id()) else { return; };
		if !widget.is_visible() {
			return;
		}

		widget.paint(painter);
		painter.set_vertex_z_value(widget.z_value());

		for child in widget.children().clone() {
			Self::paint_widget(widgets, res, painter, child);
		}
	}
	fn immediate_paint_widget(
		widgets: &mut VxGenVector<Box<dyn VxWidget>>,
		res: &mut VxAppResource,
		input: &VxInputState,
		painter: &mut VxPainter,
		id: VxWidgetId,
	) {
		let Some(widget) = widgets.get_mut(id.id()) else { return; };
		if !widget.is_visible() {
			return;
		}

		widget.immediate_paint(input, painter);
		painter.set_vertex_z_value(widget.z_value());

		for child in widget.children().clone() {
			Self::immediate_paint_widget(widgets, res, input, painter, child);
		}
	}

	fn find_widget_at(&self, target_spatial_index: &VxSpatialIndex, pos: VxVec2) -> Option<VxWidgetId> {
		let result = target_spatial_index.hit_test(pos);
		if result.is_empty() { return None; }

		let res_id = result.into_iter()
			.filter_map(|id| {
				let widget = self.widgets.get(id.id())?;
				if widget.is_visible() && widget.bounding_rect().contains(pos) {
					Some((widget.z_value(), id))
				} else {
					None
				}
			})
			.max_by_key(|(z, _)| *z)
			.map(|(_, id)| id)?;

		let widget = self.widgets.get(res_id.id())?;
		if widget.spatial_hierarchy_flag() == VxSpatialHierarchyFlag::Hierarchical {
			return self.find_widget_at(self.hierarchical_spatial_index.get(&res_id)?, pos);
		}
		Some(res_id)
	}

	pub fn check_dirty(&mut self) -> VxDirtyCheckResult {
		let mut result = VxDirtyCheckResult::None;
		let mut has_immediate = false;
		let mut queue = self.dirty_queue.take();
		for (id, flag) in queue.drain(..) {
			if flag == VxDirtyFlag::CLEAN { continue; }
			let widget = self.widgets.get_mut(id.id())
				.expect("VxScene> Critical: Invalid widget called [set_dirty]");
			match flag {
				VxDirtyFlag::BOUNDING_RECT => {
					// if widget.spatial_hierarchy_flag() == VxSpatialHierarchyFlag::Hierarchical {
					// 	let Some(spatial_index) = self.hierarchical_spatial_index.get_mut(&VxWidgetId::new(id)) else { continue; };
					// 	Self::update_spatial_index(
					// 		spatial_index,
					// 		widget
					// 	);
					// } else {
					// 	Self::update_spatial_index(&mut self.flat_spatial_index, widget);
					// }
					
					// テスト用に一旦全部フラット
					Self::update_spatial_index(&mut self.flat_spatial_index, widget);
					result = VxDirtyCheckResult::All;
				}
				VxDirtyFlag::REPAINT => {
					result = VxDirtyCheckResult::All;
				}
				// 機能は後々追加
				_ => {}
			}
			match widget.update_mode() {
				VxRenderMode::Immediate => { has_immediate = true; }
				_ => {}
			}
		}
		if result == VxDirtyCheckResult::None && has_immediate {
			result = VxDirtyCheckResult::OnlyImmediate;
		}
		result
	}

	fn update_spatial_index(target_spatial_index: &mut VxSpatialIndex, target_widget: &Box<dyn VxWidget>) {
		target_spatial_index.update_at(
			target_widget.widget_id().unwrap(),
			VxUtilConverter::rect_to_aabb(target_widget.bounding_rect())
		);
	}

	pub fn add_widget<W: VxWidget>(&mut self, mut widget: W) -> VxWidgetHandler<W> {
		let children = widget.stats_mut().children_widgets();

		let id = VxWidgetId::new(self.widgets.insert_with_key(|id | {
			let widget_id = VxWidgetId::new(id);
			widget.stats_mut().set_widget_id(widget_id);
			widget.stats_mut().set_dirty_command_sender(self.dirty_command_sender.clone());
			if widget.parent().is_none() {
				self.top_level_widgets.push(widget_id);
			}
			if widget.spatial_hierarchy_flag() == VxSpatialHierarchyFlag::Hierarchical {
				self.hierarchical_spatial_index.insert(widget_id, VxSpatialIndex::new());
			}
			Box::new(widget)
		}));

		for mut child in children {
			child.set_parent(id);
			let child_id = self.add_widget_box(child);
			let parent = self.widgets.get_mut(id.id()).expect("VxScene> Critical: Parent widget not found");
			parent.stats_mut().add_child(child_id);
		}

		VxWidgetHandler::<W>::new(id)
	}
	pub fn add_widget_box(&mut self, mut widget: Box<dyn VxWidget>) -> VxWidgetId {
		let children = widget.stats_mut().children_widgets();
		let id = VxWidgetId::new(self.widgets.insert_with_key(|id| {
			let widget_id = VxWidgetId::new(id);
			widget.stats_mut().set_widget_id(widget_id);
			widget.stats_mut().set_dirty_command_sender(self.dirty_command_sender.clone());
			if widget.parent().is_none() {
				self.top_level_widgets.push(widget_id);
			}
			if widget.spatial_hierarchy_flag() == VxSpatialHierarchyFlag::Hierarchical {
				self.hierarchical_spatial_index.insert(widget_id, VxSpatialIndex::new());
			}
			widget
		}));

		for mut child in children {
			child.set_parent(id);
			let child_id = self.add_widget_box(child);
			let parent = self.widgets.get_mut(id.id()).expect("VxScene> Critical: Parent widget not found");
			parent.stats_mut().add_child(child_id);
		}

		id
	}

	pub fn get_widget<W: VxWidget>(&self, widget: VxWidgetHandler<W>) -> Option<&W> {
		self.widgets.get(widget.id().id())?
			.as_any()
			.downcast_ref::<W>()
	}
	pub fn get_widget_mut<W: VxWidget>(&mut self, widget: VxWidgetHandler<W>) -> Option<&mut W> {
		self.widgets.get_mut(widget.id().id())?
			.as_any_mut()
			.downcast_mut::<W>()
	}

	fn send_event_to_widget<E>(
		&mut self,
		mut start_id: Option<VxWidgetId>,
		event: &E,
		handler: impl Fn(&mut Box<dyn VxWidget>, &E) -> VxEventResult
	) -> VxEventResult {
		while let Some(id) = start_id {
			let Some(widget) = self.widgets.get_mut(id.id()) else { break; };
			let result = handler(widget, event);
			if result == VxEventResult::Accept {
				return VxEventResult::Accept;
			}
			start_id = widget.parent();
		}
		VxEventResult::Ignore
	}

	pub fn mouse_press_event(&mut self, event: &VxMouseEvent)  -> VxEventResult {
		let current_id = self.find_widget_at(&self.flat_spatial_index, event.pos());
		self.current_selected_widgets = current_id;
		self.send_event_to_widget(
			current_id,
			event,
			|w, e | w.mouse_press_event(e)
		)
	}
	pub fn mouse_release_event(&mut self, event: &VxMouseEvent) -> VxEventResult {
		let current_id = self.find_widget_at(&self.flat_spatial_index, event.pos());
		self.current_selected_widgets = current_id;
		self.send_event_to_widget(
			current_id,
			event,
			|w, e| w.mouse_release_event(e)
		)
	}
	pub fn mouse_move_event(&mut self, event: &VxMouseEvent) -> VxEventResult {
		let new_hover_id = self.find_widget_at(&self.flat_spatial_index, event.pos());
		let old_hover_id = self.current_hovered_widgets;

		if new_hover_id != old_hover_id {
			if let Some(old_id) = old_hover_id {
				self.send_event_to_widget(
					Some(old_id),
					event,
					|w, e| w.mouse_leave_event(e)
				);
			}
			if let Some(new_id) = new_hover_id {
				self.send_event_to_widget(
					Some(new_id),
					event,
					|w, e| w.mouse_enter_event(e)
				);
			}
			self.current_hovered_widgets = new_hover_id;
		}

		self.send_event_to_widget(
			new_hover_id,
			event,
			|w, e| w.mouse_move_event(e)
		)
	}
	pub fn mouse_wheel_event(&mut self, event: &VxMouseEvent) -> VxEventResult {
		let current_id = self.find_widget_at(&self.flat_spatial_index, event.pos());
		self.send_event_to_widget(
			current_id,
			event,
			|w, e| w.mouse_wheel_event(e)
		)
	}
	// KeyboardEvents
	pub fn key_press_event(&mut self, event: &VxKeyEvent) -> VxEventResult {
		let current_id = self.current_selected_widgets;
		self.send_event_to_widget(
			current_id,
			event,
			|w, e| w.key_press_event(e)
		)
	}
	pub fn key_release_event(&mut self, event: &VxKeyEvent) -> VxEventResult {
		let current_id = self.current_selected_widgets;
		self.send_event_to_widget(
			current_id,
			event,
			|w, e| w.key_release_event(e)
		)
	}
}