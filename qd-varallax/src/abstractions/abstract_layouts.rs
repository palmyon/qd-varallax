use ahash::{AHashMap, AHashSet};

use crate::{
	abstractions::abstract_widgets::{
		VxWidget,
		VxWidgetId,
		VxWidgetLayoutExt
	},
	core::{
		glyph::VxFont,
		resource::VxAppResource
	},
	types::{
		gen_vector::VxGenVector,
		geometry::{
			VxRect,
			VxSize,
			VxVec2
		}
	}
};


pub struct VxBoundingRectCreator<'a> {
	res: &'a mut VxAppResource
}
impl<'a> VxBoundingRectCreator<'a> {
	pub(crate) fn new(res: &'a mut VxAppResource) -> Self {
		Self { res }
	}
	pub fn create_text_bounding_rect(&mut self, text: &str, font: VxFont) -> VxRect {
		self.res.fonts.create_text_bounding_rect(&self.res.gpu, font, text)
	}
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum VxSpatialLayoutAnchor {
	#[default]
	Window,
	ParentWidget,
	Widget(VxWidgetId),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum VxSpatialLayoutAxisRule {
	Fixed { offset: f32, size: f32 },
	Content { offset: f32 },
	Ratio { offset: f32, ratio: f32 },
	Align {
		target_edge: VxSpatialLayoutEdgeAlignment,
		my_edge: VxSpatialLayoutEdgeAlignment,
		offset: f32,
		size: VxSpatialLayoutSizeRule,
	},
	Fill { margin_start: f32, margin_end: f32 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum VxSpatialLayoutEdgeAlignment {
	/// AxisX => Left, AxisY => Top
	Start,
	#[default]
	Center,
	/// AxisX => Right, AxisY => Bottom
	End,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum VxSpatialLayoutSizeRule {
	Fixed(f32),
	Content,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VxLayout {
	anchor_x: VxSpatialLayoutAnchor,
	rule_x: VxSpatialLayoutAxisRule,
	anchor_y: VxSpatialLayoutAnchor,
	rule_y: VxSpatialLayoutAxisRule,

	intrinsic_size_cache: Option<VxSize>,
}

impl Default for VxLayout {
	#[inline]
	fn default() -> Self {
		Self {
			anchor_x: VxSpatialLayoutAnchor::ParentWidget,
			rule_x: VxSpatialLayoutAxisRule::Fixed { offset: 0.0, size: 100.0 },
			anchor_y: VxSpatialLayoutAnchor::ParentWidget,
			rule_y: VxSpatialLayoutAxisRule::Fixed { offset: 0.0, size: 100.0 },
			intrinsic_size_cache: None,
		}
	}
}

impl VxLayout {
	pub const fn new() -> Self {
		Self {
			anchor_x: VxSpatialLayoutAnchor::ParentWidget,
			rule_x: VxSpatialLayoutAxisRule::Align {
				target_edge: VxSpatialLayoutEdgeAlignment::Center,
				my_edge: VxSpatialLayoutEdgeAlignment::Center,
				offset: 0.0,
				size: VxSpatialLayoutSizeRule::Content,
			},
			anchor_y: VxSpatialLayoutAnchor::ParentWidget,
			rule_y: VxSpatialLayoutAxisRule::Align {
				target_edge: VxSpatialLayoutEdgeAlignment::Center,
				my_edge: VxSpatialLayoutEdgeAlignment::Center,
				offset: 0.0,
				size: VxSpatialLayoutSizeRule::Content,
			},
			intrinsic_size_cache: None,
		}
	}
	// (pos(min), size) for 1D 
	pub fn compute_axis_1d(
		rule: VxSpatialLayoutAxisRule,
		target_min: f32,
		target_size: f32,
		intrinsic_size: Option<f32>,
	) -> (f32, f32) {
		match rule {
			VxSpatialLayoutAxisRule::Fixed { offset, size } => {
				(target_min + offset, size.max(0.0))
			}
			VxSpatialLayoutAxisRule::Content { offset } => {
				let size = intrinsic_size.unwrap_or(0.0).max(0.0);
				(target_min + offset, size)
			}
			VxSpatialLayoutAxisRule::Ratio { offset, ratio } => {
				let size = (target_size * ratio).max(0.0);
				(target_min + offset, size)
			}
			VxSpatialLayoutAxisRule::Align { target_edge, my_edge, offset, size } => {
				let my_size = match size {
					VxSpatialLayoutSizeRule::Fixed(s) => s,
					VxSpatialLayoutSizeRule::Content => intrinsic_size.unwrap_or(0.0)
				}.max(0.0);
				let target_pos = match target_edge {
					VxSpatialLayoutEdgeAlignment::Start => target_min,
					VxSpatialLayoutEdgeAlignment::Center => target_min + target_size * 0.5,
					VxSpatialLayoutEdgeAlignment::End => target_min + target_size,
				};
				let my_min = match my_edge {
					VxSpatialLayoutEdgeAlignment::Start => target_pos + offset,
					VxSpatialLayoutEdgeAlignment::Center => target_pos - (my_size * 0.5) + offset,
					VxSpatialLayoutEdgeAlignment::End => target_pos - my_size + offset,
			};
				(my_min, my_size)
			}
			VxSpatialLayoutAxisRule::Fill { margin_start, margin_end } => {
				let my_min = target_min + margin_start;
				let my_size = (target_size - margin_start - margin_end).max(0.0);
				(my_min, my_size)
			}
		}
	}

	#[inline]
	pub fn with_anchor_x(mut self, anchor_x: VxSpatialLayoutAnchor) -> Self {
		self.anchor_x = anchor_x;
		self
	}
	#[inline]
	pub fn with_anchor_y(mut self, anchor_y: VxSpatialLayoutAnchor) -> Self {
		self.anchor_y = anchor_y;
		self
	}
	#[inline]
	pub fn with_rule_x(mut self, rule_x: VxSpatialLayoutAxisRule) -> Self {
		self.rule_x = rule_x;
		self
	}
	#[inline]
	pub fn with_rule_y(mut self, rule_y: VxSpatialLayoutAxisRule) -> Self {
		self.rule_y = rule_y;
		self
	}
}

struct VxSpatialLayoutVisitingGuard<'a> {
	resolver: &'a mut VxSpatialLayoutResolver,
	target_id: VxWidgetId,
}
impl<'a> Drop for VxSpatialLayoutVisitingGuard<'a> {
	#[inline]
	fn drop(&mut self) {
		self.resolver.visiting.remove(&self.target_id);
	}
}
impl<'a> VxSpatialLayoutVisitingGuard<'a> {
	#[inline]
	pub const fn new(resolver: &'a mut VxSpatialLayoutResolver, target_id: VxWidgetId) -> Self {
		Self {
			resolver, target_id
		}
	}
}

pub struct VxSpatialLayoutResolver {
	computed_rects: AHashMap<VxWidgetId, VxRect>,
	visiting: AHashSet<VxWidgetId>,
}

impl VxSpatialLayoutResolver {
	#[inline]
	pub fn new() -> Self {
		Self { computed_rects: AHashMap::new(), visiting: AHashSet::new() }
	}

	pub fn resolve_at(
		&mut self,
		bounging_rect_creator: &mut VxBoundingRectCreator,
		target_widget_id: VxWidgetId,
		all_widgets: &mut VxGenVector<Box<dyn VxWidget>>,
		window_size: VxSize,
	) -> Option<VxRect> {
		if let Some(&rect) = self.computed_rects.get(&target_widget_id) {
			return Some(rect);
		}

		if self.visiting.contains(&target_widget_id) {
			eprintln!("VxSpatialLayoutResolver> CircularError: Circular layout dependency detacted on Widget: {:?}", target_widget_id);
			return None;
		}

		let size_hint: Option<VxSize>;
		{
			let widget = all_widgets.get_mut(target_widget_id.id())?;
			size_hint = widget.size_hint(bounging_rect_creator);
		}
		
		let widget = all_widgets.get(target_widget_id.id())?;
		self.visiting.insert(target_widget_id);
		let guard = VxSpatialLayoutVisitingGuard::new(self, target_widget_id);

		let layout = widget.layout();
		let anchor_x = layout.anchor_x;
		let anchor_y = layout.anchor_y;
		let rule_x = layout.rule_x;
		let rule_y = layout.rule_y;
		let parent = widget.parent();

		let (x_min, x_size) = guard.resolver.get_anchor_bounds_1d(
			bounging_rect_creator,
			anchor_x,
			parent,
			true,
			all_widgets,
			window_size
		);
		let (x, width) = VxLayout::compute_axis_1d(
			rule_x,
			x_min,
			x_size,
			size_hint.map(|s| s.width())
		);

		let (y_min, y_size) = guard.resolver.get_anchor_bounds_1d(
			bounging_rect_creator,
			anchor_y,
			parent,
			false,
			all_widgets,
			window_size
		);
		let (y, height) = VxLayout::compute_axis_1d(
			rule_y,
			y_min,
			y_size,
			size_hint.map(|s| s.height())
		);

		let final_rect = VxRect::new(x, y, width, height);
		guard.resolver.computed_rects.insert(target_widget_id, final_rect);

		Some(final_rect)
	}

	fn get_anchor_bounds_1d(
		&mut self,
		bounging_rect_creator: &mut VxBoundingRectCreator,
		anchor: VxSpatialLayoutAnchor,
		parent_id: Option<VxWidgetId>,
		is_x: bool,
		widgets: &mut VxGenVector<Box<dyn VxWidget>>,
		window_size: VxSize,
	) -> (f32, f32) {
		match anchor {
			VxSpatialLayoutAnchor::Window => {
				if is_x {
					(0.0, window_size.width())
				} else {
					(0.0, window_size.height())
				}
			}
			VxSpatialLayoutAnchor::ParentWidget => {
				if let Some(parent) = parent_id {
					if let Some(parent_rect) = self.resolve_at(bounging_rect_creator, parent, widgets, window_size) {
						return if is_x {
							(parent_rect.x(), parent_rect.width())
						} else {
							(parent_rect.y(), parent_rect.height())
						}
					}
				}
				self.get_anchor_bounds_1d(bounging_rect_creator, VxSpatialLayoutAnchor::Window, None, is_x, widgets, window_size)
			}
			VxSpatialLayoutAnchor::Widget(target_id) => {
				if let Some(target_rect) = self.resolve_at(bounging_rect_creator, target_id, widgets, window_size) {
					if is_x {
						(target_rect.x(), target_rect.width())
					} else {
						(target_rect.y(), target_rect.height())
					}
				} else {
					self.get_anchor_bounds_1d(bounging_rect_creator, VxSpatialLayoutAnchor::ParentWidget, parent_id, is_x, widgets, window_size)
				}
			}
		}
	}

	#[inline]
	pub fn clear(&mut self) {
		self.computed_rects.clear();
		self.visiting.clear();
	}

	#[inline]
	pub fn invalidate(&mut self, target_id: VxWidgetId) {
		self.computed_rects.remove(&target_id);
	}
}


#[derive(Clone, Copy)]
pub struct VxImmediateLayoutContext {
	cursor: VxVec2,
	available_size: VxSize,
}
impl VxImmediateLayoutContext {
	#[inline]
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