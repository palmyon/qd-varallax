use qd_varallax_core::abstracts::prelude::*;

#[derive(Debug)]
pub struct VxValueWidgetComponent {
	value: f64,
	range_min: f64,
	range_max: f64,
	step: f64,
}
impl VxValueWidgetComponent {
	pub fn new(range_min: f64, range_max: f64) -> Self {
		Self {
			value: 0.0,
			range_min,
			range_max,
			step: 1.0,
		}
	}
	#[inline]
	pub fn value(&self) -> f64 { self.value }
	#[inline]
	pub fn range_min(&self) -> f64 { self.range_min }
	#[inline]
	pub fn range_max(&self) -> f64 { self.range_max }
	#[inline]
	pub fn step(&self) -> f64 { self.step }
	#[inline]
	pub fn range(&self) -> f64 { self.range_max - self.range_min }
	#[inline]
	pub fn set_value(&mut self, value: f64) {
		self.value = value.min(self.range_max).max(self.range_min);
	}
	#[inline]
	pub fn set_range_min(&mut self, min: f64) { self.range_min = min; }
	#[inline]
	pub fn set_range_max(&mut self, max: f64) { self.range_max = max; }
	#[inline]
	pub fn set_step(&mut self, step: f64) { self.step = step; }

	#[inline]
	pub fn ratio(&self) -> f64 {
		((self.value - self.range_min) / (self.range_max - self.range_min)).min(1.0).max(0.0)
	}
	#[inline]
	pub fn percent(&self) -> f64 {
		(self.ratio() * 100.0).min(100.0).max(0.0)
	}
}

pub trait VxWidgetValueExtension: VxWidget {
	fn value_component(&self) -> &VxValueWidgetComponent;
	fn value_component_mut(&mut self) -> &mut VxValueWidgetComponent;
}

pub trait VxWidgetValueExtensionExt: VxWidgetValueExtension {
	#[inline]
	fn value(&self) -> f64 { self.value_component().value() }
	#[inline]
	fn range_min(&self) -> f64 { self.value_component().range_min() }
	#[inline]
	fn range_max(&self) -> f64 { self.value_component().range_max() }
	#[inline]
	fn step(&self) -> f64 { self.value_component().step() }
	#[inline]
	fn range(&self) -> f64 { self.value_component().range() }
	#[inline]
	fn set_value(&mut self, value: f64) {
		self.value_component_mut().set_value(value);
		self.set_dirty(VxDirtyFlag::REPAINT);
	}
	#[inline]
	fn set_range_min(&mut self, min: f64) {
		self.value_component_mut().set_range_min(min);
		self.set_dirty(VxDirtyFlag::REPAINT);
	}
	#[inline]
	fn set_range_max(&mut self, max: f64) {
		self.value_component_mut().set_range_max(max);
		self.set_dirty(VxDirtyFlag::REPAINT);
	}
	#[inline]
	fn set_step(&mut self, step: f64) {
		self.value_component_mut().set_step(step);
	}

	#[inline]
	fn ratio(&self) -> f64 { self.value_component().ratio() }
	#[inline]
	fn percent(&self) -> f64 { self.value_component().percent() }
}
impl<T: VxWidgetValueExtension + ?Sized> VxWidgetValueExtensionExt for T {}