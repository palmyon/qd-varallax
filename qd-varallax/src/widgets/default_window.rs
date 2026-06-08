use vx_macro::VxWindowDerive;

use crate::abstractions::abstract_windows::{
	VxWindow,
	VxWindowAttributes,
	VxWindowStats,
	VxWindowInternal,
	VxWindowBuilder,
};

#[derive(VxWindowDerive)]
pub struct VxDefaultWindow {
	#[vx(Stat)]
	stats: Option<VxWindowStats>,
	#[vx(WindowAttr)]
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