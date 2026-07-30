use std::{cell::RefCell, rc::Rc};

use ahash::{AHashMap, AHashSet};

use crate::{
	abstractions::{
		abstract_layouts::{VxBoundingRectCreator, VxSpatialLayoutResolver}, abstract_widgets::{
			VxDirtyCommandSender, VxDirtyFlag, VxSpatialHierarchyFlag, VxWidget, VxWidgetGeometryExt, VxWidgetHandler, VxWidgetId
		}
	}, core::{
		resource::VxAppResource, spatial_index::VxSpatialIndex
	}, painter::painter::VxPainter, types::{
		event::{
			VxEventResult,
			VxKeyEvent,
			VxMouseEvent
		}, gen_vector::{VxGenIndexInvalid, VxGenVector}, geometry::{
			VxSize, VxVec2
		}, input::VxInputState, render_commands::{
			VxDirtyCheckResult,
			VxRenderMode
		}
	},
};

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub enum VxSpatialUpdateFlag {
	Flat,
	Hierarchical(VxWidgetId),
}

pub struct VxScene {
	widgets: VxGenVector<Box<dyn VxWidget>>,
	top_level_widgets: Vec<VxWidgetId>,
	immediate_widgets: Vec<VxWidgetId>,
	current_selected_widgets: Option<VxWidgetId>,
	current_hovered_widgets: Option<VxWidgetId>,
	dirty_command_sender: VxDirtyCommandSender,
	dirty_queue: Rc<RefCell<AHashMap<VxWidgetId, VxDirtyFlag>>>,

	flat_spatial_index: VxSpatialIndex<VxWidgetId>,
	hierarchical_spatial_index: AHashMap<VxWidgetId, VxSpatialIndex<VxWidgetId>>,
	need_update_spatial_index: AHashSet<VxSpatialUpdateFlag>,

	layout_resolver: VxSpatialLayoutResolver,
}

impl VxScene {
	pub fn new() -> Self {
		let dirty_queue = Rc::new(RefCell::new(AHashMap::new()));
		let queue_clone = dirty_queue.clone();
		Self {
			widgets: VxGenVector::new(),
			top_level_widgets: Vec::new(),
			immediate_widgets: Vec::new(),
			current_selected_widgets: None,
			current_hovered_widgets: None,
			dirty_command_sender: VxDirtyCommandSender::new(move |id, flag| {
				*queue_clone.borrow_mut().entry(id).or_insert(flag) |= flag;
			}),
			dirty_queue,
			flat_spatial_index: VxSpatialIndex::new(),
			hierarchical_spatial_index: AHashMap::new(),
			need_update_spatial_index: AHashSet::new(),
			layout_resolver: VxSpatialLayoutResolver::new(),
		}
	}
	pub fn paint_event(&mut self, res: &mut VxAppResource, painter: &mut VxPainter) {
		self.top_level_widgets.iter().for_each(|id| {
			Self::paint_widget(&mut self.widgets, res, painter, id.clone());
		});
	}
	pub fn immediate_paint_event(&mut self, res: &mut VxAppResource, input: &VxInputState, painter: &mut VxPainter) {
		self.immediate_widgets.iter().for_each(|id| {
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

		painter.push_tranform(widget.transform());
		for child in widget.children().clone() {
			Self::paint_widget(widgets, res, painter, child);
		}
		painter.pop_transform();
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

	fn find_widget_at(&self, target_spatial_index: &VxSpatialIndex<VxWidgetId>, pos: VxVec2) -> Option<VxWidgetId> {
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
		match widget.spatial_hierarchy_flag() {
			VxSpatialHierarchyFlag::Flat => {},
			VxSpatialHierarchyFlag::HierarchyParent => {},
			VxSpatialHierarchyFlag::HierarchyChild => {
				let spatial_parent = widget.stats().spatial_hierarchy_parent();
				return self.find_widget_at(self.hierarchical_spatial_index.get(&spatial_parent)?, pos - widget.pos());
			}
		}
		Some(res_id)
	}

	pub(crate) fn check_dirty(&mut self, res: &mut VxAppResource, window_size: VxSize) -> VxDirtyCheckResult {
		let mut result = VxDirtyCheckResult::None;
		let mut has_immediate = false;
		let mut bounding_rect_creator = VxBoundingRectCreator::new(res);
		let queue = self.dirty_queue.take();
		for (id, flag) in queue.into_iter() {
			if flag == VxDirtyFlag::CLEAN { continue; }
			let Some(widget) = self.widgets.get_mut(id.id()) else { continue; };

			if widget.update_mode() == VxRenderMode::Immediate {
				has_immediate = true;
			}

			match flag {
				VxDirtyFlag::LAYOUT | VxDirtyFlag::REBUILD_ALL => {
					self.layout_resolver.invalidate(id);

					if let Some(new_rect) = self.layout_resolver.resolve_at(
						&mut bounding_rect_creator, id, &mut self.widgets, window_size
					) {
						let widget = self.widgets.get_mut(id.id()).unwrap();
						widget.stats_mut().set_block_dirty(true);
						widget.stats_mut().set_bounding_rect(new_rect.with_pos(VxVec2::default()));
						widget.stats_mut().set_pos(new_rect.pos());
						widget.stats_mut().set_block_dirty(false);

						match widget.spatial_hierarchy_flag() {
							VxSpatialHierarchyFlag::HierarchyChild => {
								let spatial_id = widget.stats().spatial_hierarchy_parent();
								if let Some(spatial_index) = self.hierarchical_spatial_index.get_mut(
									&spatial_id
								) {
									let local_rect = new_rect.with_pos(new_rect.pos() - widget.pos());
									spatial_index.update_at(id, local_rect);
									self.need_update_spatial_index.insert(VxSpatialUpdateFlag::Hierarchical(spatial_id));
								}
							}
							_ => {
								self.flat_spatial_index.update_at(id, new_rect);
								self.need_update_spatial_index.insert(VxSpatialUpdateFlag::Flat);
							}
						}
					}
					result = VxDirtyCheckResult::All;
				}
				VxDirtyFlag::REPAINT => {
					result = VxDirtyCheckResult::All;
				}
				_ => {}
			}
		}

		for update_flag in self.need_update_spatial_index.drain() {
			match update_flag {
				VxSpatialUpdateFlag::Flat => self.flat_spatial_index.optimize(),
				VxSpatialUpdateFlag::Hierarchical(id) => {
					if let Some(spatial_index) = self.hierarchical_spatial_index.get_mut(&id) {
						spatial_index.optimize();
					}
				}
			}
		}

		if result == VxDirtyCheckResult::None && (has_immediate || !self.immediate_widgets.is_empty()) {
			result = VxDirtyCheckResult::OnlyImmediate;
		}
		result
	}

	fn find_spatial_index(&self, mut current_id: VxWidgetId) -> VxWidgetId {
		while let Some(parent_id) = self.widgets.get(current_id.id()).and_then(|w| w.parent()) {
			let Some(parent) = self.widgets.get(parent_id.id()) else { break; };
			if parent.spatial_hierarchy_flag() == VxSpatialHierarchyFlag::HierarchyParent {
				return parent_id;
			}
			current_id = parent_id;
		}
		current_id
	}

	pub(crate) fn refresh_spatial_index(&mut self) {
		let data = self.widgets.iter_with_id()
			.map(|(id, widget)| {
				(VxWidgetId::new(id), widget.bounding_rect())
			})
			.collect::<Vec<_>>();
		self.flat_spatial_index.rebuild_bvh(&data);
	}

	fn register_widget<W: VxWidget + ?Sized>(
		widget: &mut Box<W>,
		widget_id: VxWidgetId,
		dirty_command_sender: &VxDirtyCommandSender,
		top_level_widgets: &mut Vec<VxWidgetId>,
		immediate_widgets: &mut Vec<VxWidgetId>,
		hierarchical_spatial_index: &mut AHashMap<VxWidgetId, VxSpatialIndex<VxWidgetId>>,
	) {
		widget.stats_mut().set_widget_id(widget_id);
		widget.stats_mut().set_dirty_command_sender(dirty_command_sender.clone());
		if widget.parent().is_none() {
			top_level_widgets.push(widget_id);
		}
		if widget.update_mode() == VxRenderMode::Immediate {
			immediate_widgets.push(widget_id);
		}
		match widget.spatial_hierarchy_flag() {
			VxSpatialHierarchyFlag::HierarchyParent => {
				hierarchical_spatial_index.insert(widget_id, VxSpatialIndex::new());
			}
			_ => {}
		}
	}

	pub fn add_widget<W: VxWidget>(&mut self, widget: W) -> VxWidgetHandler<W> {
		let id = self.add_widget_box(Box::new(widget));
		VxWidgetHandler::<W>::new(id)
	}
	pub fn add_widget_box(&mut self, mut widget: Box<dyn VxWidget>) -> VxWidgetId {
		let children = widget.stats_mut().children_widgets_take();
		let id = VxWidgetId::new(self.widgets.insert_with_key(|id| {
			let widget_id = VxWidgetId::new(id);
			Self::register_widget(
				&mut widget, widget_id,
				&self.dirty_command_sender,
				&mut self.top_level_widgets,
				&mut self.immediate_widgets,
				&mut self.hierarchical_spatial_index
			);
			widget
		}));

		for mut child in children {
			child.set_parent(id);
			let child_id = self.add_widget_box(child);
			let parent = self.widgets.get_mut(id.id()).expect("VxScene> Critical: Parent widget not found");
			parent.stats_mut().add_child(child_id);
		}

		let parent_spatial_index = match self.widgets.get(id.id()).unwrap().spatial_hierarchy_flag() {
			VxSpatialHierarchyFlag::Flat => VxWidgetId::new_invalid(),
			VxSpatialHierarchyFlag::HierarchyParent => id,
			VxSpatialHierarchyFlag::HierarchyChild => self.find_spatial_index(id),
		};

		if let Some(w) = self.widgets.get_mut(id.id()) {
			w.stats_mut().set_spatial_hierarchy_parent(parent_spatial_index);
		}

		id
	}

	pub fn get_widget<W: VxWidget>(&self, handler: VxWidgetHandler<W>) -> Option<&W> {
		self.widgets.get(handler.id().id())?
			.as_any()
			.downcast_ref::<W>()
	}
	pub fn get_widget_mut<W: VxWidget>(&mut self, handler: VxWidgetHandler<W>) -> Option<&mut W> {
		self.widgets.get_mut(handler.id().id())?
			.as_any_mut()
			.downcast_mut::<W>()
	}

	// input event handler ===========================================================================================
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
	// =============================================================================================================
}