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
		fonts.preload_glyphs(&gpu, "kokumr", include_str!("../../../qd-varallax/src/assets/pre_load.txt"));

		Self {
			gpu,
			fonts,
			textures,
		}
	}
}