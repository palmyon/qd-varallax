use crate::core::{
	gpu_resource::VxGpuResource,
	systems::{
		VxFontSystem,
		VxTextureSystem
	}
};



pub(crate) struct VxAppResource {
	pub gpu: VxGpuResource,
	pub fonts: VxFontSystem,
	pub textures: VxTextureSystem,
}

impl VxAppResource {
	pub(crate) fn new() -> Self {
		let gpu = pollster::block_on(VxGpuResource::new());
		let textures = VxTextureSystem::new(&gpu);
		let mut fonts = VxFontSystem::new(&gpu);

		fonts.register_font("kokumr", include_bytes!("../../../qd-varallax/src/assets/kokumr.TTF"));

		Self {
			gpu,
			fonts,
			textures,
		}
	}
}