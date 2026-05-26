use crate::core::{gpu_resource::VxGpuResource, systems::{VxFontSystem, VxTextureSystem}};



pub(crate) struct VxAppResources {
	pub gpu: VxGpuResource,
	pub fonts: VxFontSystem,
	pub textures: VxTextureSystem,
}

impl VxAppResources {
	pub(crate) fn new() -> Self {
		let gpu = pollster::block_on(VxGpuResource::new());
		let textures = VxTextureSystem::new(&gpu);
		let mut fonts = VxFontSystem::new(&gpu);
		fonts.storage.register_font("kokumr", include_bytes!("../../../qd-varallax/src/assets/kokumr.TTF"));
		fonts.set_init_glyphs(&gpu, "kokumr", 32.0, include_str!("../../../qd-varallax/src/assets/pre_load.txt"));
		Self {
			gpu,
			fonts,
			textures,
		}
	}
}