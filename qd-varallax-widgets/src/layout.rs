use qd_varallax_macro::{VxBoxLayoutImpl, VxWidgetImpl};

use qd_varallax_core::{
	abstracts::prelude::*, painter::painter::VxPainter, types::prelude::*,
};


#[derive(VxWidgetImpl, VxBoxLayoutImpl)]
pub struct VxVBoxLayoutWidget {
	#[attr(context)]
	context: VxWidgetContext,
	#[attr_layout(context)]
	layout_context: VxBoxLayoutContext,
}

impl VxWidget for VxVBoxLayoutWidget {
	fn size_hint(&mut self, _: &mut VxBoundingRectCreator) -> VxSizeHint {
		VxSizeHint::ChildrenSize { padding: 0.0 }
	}
	fn paint(&mut self, painter: &mut VxPainter, palette: &qd_varallax_core::types::style::VxColorPalette) {
		painter.draw_rect(self.bounding_rect().with_pos(self.pos()), VxColor::from_hex_with_alpha(0xFF000022));
		let _ = palette;
	}
	#[inline]
	fn as_box_layout(&self) -> Option<&dyn VxBoxLayout> {
		Some(self)
	}
	#[inline]
	fn as_box_layout_mut(&mut self) -> Option<&mut dyn VxBoxLayout> {
		Some(self)
	}
}

impl VxBoxLayout for VxVBoxLayoutWidget {
	#[inline]
	fn layout_fn(&self) -> (
			fn(
				&VxBoxLayoutContext,
				&mut VxBoundingRectCreator<'_>,
				VxRect,&[VxWidgetId],
				&mut VxGenVector<Box<dyn VxWidget>>,
			) -> Vec<(VxWidgetId, VxRect)>,
			VxBoxLayoutContext
		)
	{
		(Self::calc_vbox_layout, self.layout_context)
	}
}

impl VxVBoxLayoutWidget {
	pub fn new(parent: Option<VxWidgetId>) -> Self {
		let mut layout_context = VxBoxLayoutContext::new();
		layout_context.set_orientation(VxOrientation::Vertical);
		let mut context = VxWidgetContext::new(parent);
		context.set_z_value(-5);
		Self {
			context,
			layout_context,
		}
	}

	fn calc_vbox_layout(
		stats: &VxBoxLayoutContext,
		creator: &mut VxBoundingRectCreator,
		container_rect: VxRect,
		children: &[VxWidgetId],
		widgets: &mut VxGenVector<Box<dyn VxWidget>>,
	) -> Vec<(VxWidgetId, VxRect)> {
		let mut results = Vec::with_capacity(children.len());
		if children.is_empty() {
			return results;
		}

		let spacing = stats.spacing();
		let padding = stats.padding();
		let main_align = stats.main_alignment();
		let cross_align = stats.cross_alignment();

		let inner_x = container_rect.x() + padding;
		let inner_y = container_rect.y() + padding;
		let inner_w = (container_rect.width() - padding * 2.0).max(0.0);
		let inner_h = (container_rect.height() - padding * 2.0).max(0.0);

		let mut child_meta = Vec::with_capacity(children.len());
		let mut total_child_height = 0.0;

		for &child_id in children {
			let size_hint = VxSizeHint::resolve_size_hint(child_id, widgets, creator);
			total_child_height += size_hint.height();
			child_meta.push((child_id, size_hint));
		}

		let total_spacing = spacing * (children.len() as f32 - 1.0).max(0.0);
		let free_space = inner_h - total_child_height - total_spacing;

		let mut current_y = inner_y;
		let mut extra_height_per_child = 0.0;
		
		if free_space > 0.0 {
			match main_align {
				VxAlignment::Start => {},
				VxAlignment::Center => current_y += free_space * 0.5,
				VxAlignment::End => current_y += free_space,
				VxAlignment::Stretch => {
					extra_height_per_child = free_space / children.len() as f32;
				}
			}
		}

		for (child_id, size_hint) in child_meta {
			let child_h = size_hint.height() + extra_height_per_child;
			let (child_x, child_w) = match cross_align {
				VxAlignment::Stretch => (inner_x, inner_w),
				VxAlignment::Start => (inner_x, size_hint.width()),
				VxAlignment::Center => (inner_x + (inner_w - size_hint.width()) * 0.5, size_hint.width()),
				VxAlignment::End => (inner_x + inner_w - size_hint.width(), size_hint.width()),
			};
			results.push((child_id, VxRect::new(child_x, current_y, child_w, child_h)));
			current_y += child_h + spacing;
		}

		results
	}
}


#[derive(VxWidgetImpl, VxBoxLayoutImpl)]
pub struct VxHBoxLayoutWidget {
	#[attr(context)]
	context: VxWidgetContext,
	#[attr_layout(context)]
	layout_context: VxBoxLayoutContext,
}

impl VxWidget for VxHBoxLayoutWidget {
	fn size_hint(&mut self, _: &mut VxBoundingRectCreator) -> VxSizeHint {
		VxSizeHint::ChildrenSize { padding: 0.0 }
	}
	fn paint(&mut self, painter: &mut VxPainter, palette: &qd_varallax_core::types::style::VxColorPalette) {
		painter.draw_rect(self.bounding_rect().with_pos(self.pos()), VxColor::from_hex_with_alpha(0xFF000022));
		let _ = palette;
	}
	fn as_box_layout(&self) -> Option<&dyn VxBoxLayout> {
		Some(self)
	}
	fn as_box_layout_mut(&mut self) -> Option<&mut dyn VxBoxLayout> {
		Some(self)
	}
}

impl VxBoxLayout for VxHBoxLayoutWidget {
	fn layout_fn(&self) -> (
			fn(
				&VxBoxLayoutContext,
				&mut VxBoundingRectCreator<'_>,
				VxRect,&[VxWidgetId],
				&mut VxGenVector<Box<dyn VxWidget>>,
			) -> Vec<(VxWidgetId, VxRect)>,
			VxBoxLayoutContext
		)
	{
		(Self::calc_hbox_layout, self.layout_context)
	}
}

impl VxHBoxLayoutWidget {
	#[inline]
	pub fn new(parent: Option<VxWidgetId>) -> Self {
		let mut layout_context = VxBoxLayoutContext::new();
		layout_context.set_orientation(VxOrientation::Horizontal);
		let mut context = VxWidgetContext::new(parent);
		context.set_z_value(-5);
		Self {
			context,
			layout_context,
		}
	}
	fn calc_hbox_layout(
		stats: &VxBoxLayoutContext,
		creator: &mut VxBoundingRectCreator,
		container_rect: VxRect,
		children: &[VxWidgetId],
		widgets: &mut VxGenVector<Box<dyn VxWidget>>,
	) -> Vec<(VxWidgetId, VxRect)> {
		let mut results = Vec::with_capacity(children.len());
		if children.is_empty() {
			return results;
		}

		let spacing = stats.spacing();
		let padding = stats.padding();
		let main_align = stats.main_alignment();
		let cross_align = stats.cross_alignment();

		let inner_x = container_rect.x() + padding;
		let inner_y = container_rect.y() + padding;
		let inner_w = (container_rect.width() - padding * 2.0).max(0.0);
		let inner_h = (container_rect.height() - padding * 2.0).max(0.0);

		let mut child_meta = Vec::with_capacity(children.len());
		let mut total_child_width = 0.0;

		for &child_id in children {
			let size_hint = VxSizeHint::resolve_size_hint(child_id, widgets, creator);
			total_child_width += size_hint.width();
			child_meta.push((child_id, size_hint));
		}

		let total_spacing = spacing * (children.len() as f32 - 1.0).max(0.0);
		let free_space = inner_w - total_child_width - total_spacing;

		let mut current_x = inner_x;
		let mut extra_width_per_child = 0.0;

		if free_space > 0.0 {
			match main_align {
				VxAlignment::Start => {},
				VxAlignment::Center => current_x += free_space * 0.5,
				VxAlignment::End => current_x += free_space,
				VxAlignment::Stretch => {
					extra_width_per_child = free_space / children.len() as f32;
				}
			}
		}

		for (child_id, size_hint) in child_meta {
			let child_w = size_hint.width() + extra_width_per_child;
			let (child_y, child_h) = match cross_align {
				VxAlignment::Stretch => (inner_y, inner_h),
				VxAlignment::Start => (inner_y, size_hint.height()),
				VxAlignment::Center => (inner_y + (inner_h - size_hint.height()) * 0.5, size_hint.height()),
				VxAlignment::End => (inner_y + inner_h - size_hint.height(), size_hint.height()),
			};
			results.push((child_id, VxRect::new(current_x, child_y, child_w, child_h)));
			current_x += child_w + spacing;
		}
		
		results
	}
}