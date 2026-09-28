use qd_varallax_macro::VxWindowImpl;

use crate::abstracts::prelude::*;

#[derive(VxWindowImpl)]
pub struct VxDefaultWindow {
	#[attr(stat)]
	stats: Option<VxWindowStats>,
	#[attr(w_attr)]
	attr: VxWindowAttributes,
}

impl VxWindow for VxDefaultWindow {}

impl VxDefaultWindow {
	pub fn new(attr: VxWindowAttributes) -> Self {
		Self {
			stats: None,
			attr,
		}
	}
}