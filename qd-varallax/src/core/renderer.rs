use ahash::AHashMap;
use itertools::Itertools;

use crate::{
	core::{
		gpu_resource::VxGpuResource,
		resource::VxAppResource
	}, types::{
		render_commands::{
			VxDrawLine, VxDrawLineContainer, VxRenderMode, VxRenderModuleId, VxVertexContainer
		}, transform::VxMatrix4x4, vertex::{
			VxSdfVertex,
			VxTextVertex,
			VxTextureVertex,
			VxVertex
		}
	}
};

pub(crate) const RETAINED_VERTEX_BUFFER_NAME: &str = "VxRetainedVertexBuffer";
pub(crate) const IMMEDIATE_VERTEX_BUFFER_NAME: &str = "VxImmediateVertexBuffer";
pub(crate) const RETAINED_INDEX_BUFFER_NAME: &str = "VxRetainedIndexBuffer";
pub(crate) const IMMEDIATE_INDEX_BUFFER_NAME: &str = "VxImmediateIndexBuffer";

pub(crate) struct VxRenderModule {
	pub(crate) module_id: VxRenderModuleId,
	pub(crate) pipeline: wgpu::RenderPipeline,
	pub(crate) retained_vertex_buffer: wgpu::Buffer,
	pub(crate) retained_index_buffer: wgpu::Buffer,
	pub(crate) immediate_vertex_buffer: wgpu::Buffer,
	pub(crate) immediate_index_buffer: wgpu::Buffer,
	pub(crate) projection_buffer: wgpu::Buffer,
	pub(crate) projection_bind_group_layout: wgpu::BindGroupLayout,
	pub(crate) projection_bind_group: wgpu::BindGroup,
}

impl VxRenderModule {
	pub fn new(
		gpu: &VxGpuResource,
		surface_config: &wgpu::SurfaceConfiguration,
		module_id: VxRenderModuleId,
		shader: &str,
		bind_group_layout: &[&wgpu::BindGroupLayout],
		buffers: &[wgpu::VertexBufferLayout],
		vertex_buffer_size: u64
	) -> Self {
		// create shader module
		let shader = gpu.device.create_shader_module(
			wgpu::ShaderModuleDescriptor {
				label: Some("VxShaderModule"),
				source: wgpu::ShaderSource::Wgsl(shader.into()),
			}
		);

		// create vertex buffer
		let retained_vertex_buffer = Self::create_vertex_buffer(
			gpu,
			RETAINED_VERTEX_BUFFER_NAME,
			vertex_buffer_size
		);

		let immediate_vertex_buffer = Self::create_vertex_buffer(
			gpu,
			IMMEDIATE_VERTEX_BUFFER_NAME,
			vertex_buffer_size
		);

		// create index buffer
		let retained_index_buffer = Self::create_index_buffer(
			gpu,
			RETAINED_INDEX_BUFFER_NAME,
			std::mem::size_of::<[u32; 100]>() as u64
		);

		let immediate_index_buffer = Self::create_index_buffer(
			gpu,
			IMMEDIATE_INDEX_BUFFER_NAME,
			std::mem::size_of::<[u32; 100]>() as u64
		);

		// create projection buffer
		let projection_buffer = gpu.device.create_buffer(
			&wgpu::BufferDescriptor {
				label: Some("VxProjectionBuffer"),
				size: std::mem::size_of::<VxMatrix4x4>() as u64,
				usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
				mapped_at_creation: false,
			}
		);

		// create projection bind group layout
		let projection_bind_group_layout = gpu.device.create_bind_group_layout(
			&wgpu::BindGroupLayoutDescriptor {
				entries: &[wgpu::BindGroupLayoutEntry {
					binding: 0,
					visibility: wgpu::ShaderStages::VERTEX,
					ty: wgpu::BindingType::Buffer {
						ty: wgpu::BufferBindingType::Uniform,
						has_dynamic_offset: false,
						min_binding_size: None
					},
					count: None,
				}],
				label: Some("VxProjectionBindGroupLayout")
			}
		);

		// create projection bind group
		let projection_bind_group = gpu.device.create_bind_group(
			&wgpu::BindGroupDescriptor {
				label: Some("VxProjectionBindGroup"),
				layout: &projection_bind_group_layout,
				entries: &[wgpu::BindGroupEntry {
					binding: 0,
					resource: projection_buffer.as_entire_binding()
				}],
			}
		);

		let mut layouts = vec![&projection_bind_group_layout];
		layouts.extend_from_slice(bind_group_layout);

		// create pipeline layout including projection bind group layout
		let pipeline_layout = gpu.device.create_pipeline_layout(
			&wgpu::PipelineLayoutDescriptor {
				label: Some("VxPipelineLayout"),
				bind_group_layouts: layouts.as_slice(),
				immediate_size: 0,
			}
		);

		// create pipeline
		let pipeline = gpu.device.create_render_pipeline(
			&wgpu::RenderPipelineDescriptor {
				label: Some("VxPipeline"),
				layout: Some(&pipeline_layout),
				vertex: wgpu::VertexState {
					module: &shader,
					entry_point: Some("vs_main"),
					buffers,
					compilation_options: Default::default(),
				},
				fragment: Some(wgpu::FragmentState {
					module: &shader,
					entry_point: Some("fs_main"),
					targets: &[Some(wgpu::ColorTargetState {
						format: surface_config.format,
						blend: Some(wgpu::BlendState::ALPHA_BLENDING),
						write_mask: wgpu::ColorWrites::ALL,
					})],
					compilation_options: Default::default(),
				}),
				primitive: Default::default(),
				depth_stencil: None,
				multisample: Default::default(),
				multiview_mask: None,
				cache: None,
			}
		);

		Self {
			module_id,
			pipeline,
			retained_vertex_buffer,
			retained_index_buffer,
			immediate_vertex_buffer,
			immediate_index_buffer,
			projection_buffer,
			projection_bind_group_layout,
			projection_bind_group,
		}
	}

	pub fn check_and_update_vertex_buffer(&mut self, gpu: &VxGpuResource, buffer_type: VxRenderMode, target_size: u64) {
		match buffer_type {
			VxRenderMode::Retained => {
				if Self::is_should_extend_buffer_size(&self.retained_vertex_buffer, target_size) {
					self.retained_vertex_buffer = Self::create_vertex_buffer(
						gpu,
						RETAINED_VERTEX_BUFFER_NAME,
						(target_size as f64 * 1.5) as u64
					)
				}
			}
			VxRenderMode::Immediate => {
				if Self::is_should_extend_buffer_size(&self.immediate_vertex_buffer, target_size) {
					self.immediate_vertex_buffer = Self::create_vertex_buffer(
						gpu,
						IMMEDIATE_VERTEX_BUFFER_NAME,
						(target_size as f64 * 1.5) as u64
					)
				}
			}
		}
	}

	pub fn check_and_update_index_buffer(&mut self, gpu: &VxGpuResource, buffer_type: VxRenderMode, target_size: u64) {
		match buffer_type {
			VxRenderMode::Retained => {
				if Self::is_should_extend_buffer_size(&self.retained_index_buffer, target_size) {
					self.retained_index_buffer = Self::create_index_buffer(
						gpu,
						RETAINED_INDEX_BUFFER_NAME,
						(target_size as f64 * 1.5) as u64
					)
				}
			}
			VxRenderMode::Immediate => {
				if Self::is_should_extend_buffer_size(&self.immediate_index_buffer, target_size) {
					self.immediate_index_buffer = Self::create_index_buffer(
						gpu,
						IMMEDIATE_INDEX_BUFFER_NAME,
						(target_size as f64 * 1.5) as u64
					)
				}
			}
		}
	}

	fn is_should_extend_buffer_size(buffer: &wgpu::Buffer, target_size: u64) -> bool {
		buffer.size() < target_size
	}

	pub fn create_vertex_buffer(gpu: &VxGpuResource, label: &str, target_size: u64) -> wgpu::Buffer {
		gpu.device.create_buffer(
			&wgpu::BufferDescriptor {
				label: Some(label),
				size: target_size,
				usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
				mapped_at_creation: false,
			}
		)
	}

	pub fn create_index_buffer(gpu: &VxGpuResource, label: &str, target_size: u64) -> wgpu::Buffer {
		gpu.device.create_buffer(
			&wgpu::BufferDescriptor {
				label: Some(label),
				size: target_size,
				usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
				mapped_at_creation: false,
			}
		)
	}

	pub fn update_projection(&self, gpu: &VxGpuResource, matrix: &VxMatrix4x4) {
		gpu.queue.write_buffer(
			&self.projection_buffer,
			0,
			bytemuck::cast_slice(&[matrix.matrix()])
		);
	}
}

pub(crate) struct VxRenderer {
	modules: AHashMap<VxRenderModuleId, VxRenderModule>,
	retained_draw_lines: VxDrawLineContainer,
	immediate_draw_lines: VxDrawLineContainer,
}

impl VxRenderer {
	pub fn new(gpu: &VxGpuResource, surface_config: &wgpu::SurfaceConfiguration) -> Self {
		let vertex_module = VxRenderModule::new(
			&gpu,
			&surface_config,
			VxRenderModuleId::VERTEX,
			include_str!("shader/vertex_shader.wgsl"),
			&[],
			&[VxVertex::VERTEXBUFFERLAYOUT],
			(std::mem::size_of::<VxVertex>() * 100) as u64
		);

		let sdf_module = VxRenderModule::new(
			&gpu,
			&surface_config,
			VxRenderModuleId::SDF,
			include_str!("shader/sdf_shader.wgsl"),
			&[],
			&[VxSdfVertex::VERTEXBUFFERLAYOUT],
			(std::mem::size_of::<VxSdfVertex>() * 100) as u64,
		);

		let texture_module = VxRenderModule::new(
			&gpu,
			&surface_config,
			VxRenderModuleId::TEXTURE,
			include_str!("shader/texture_shader.wgsl"),
			&[&gpu.bind_group_layout],
			&[VxTextureVertex::VERTEXBUFFERLAYOUT],
			(std::mem::size_of::<VxTextureVertex>() * 100) as u64
		);

		let text_module = VxRenderModule::new(
			&gpu,
			&surface_config,
			VxRenderModuleId::TEXT,
			include_str!("shader/text_shader.wgsl"),
			&[&gpu.bind_group_layout],
		&[VxTextVertex::VERTEXBUFFERLAYOUT],
		(std::mem::size_of::<VxTextVertex>() * 100) as u64
		);

		let mut modules = AHashMap::new();
		modules.insert(VxRenderModuleId::VERTEX, vertex_module);
		modules.insert(VxRenderModuleId::SDF, sdf_module);
		modules.insert(VxRenderModuleId::TEXTURE, texture_module);
		modules.insert(VxRenderModuleId::TEXT, text_module);

		Self {
			modules,
			retained_draw_lines: VxDrawLineContainer::new(),
			immediate_draw_lines: VxDrawLineContainer::new(),
		}
	}

	pub(crate) fn update_projection(&self, gpu: &VxGpuResource, orthographic: VxMatrix4x4) {
		self.modules.values().for_each(|m| m.update_projection(gpu, &orthographic));
	}

	pub fn render(&mut self, res: &mut VxAppResource, surface: &wgpu::Surface) {
		let Ok(frame) = surface.get_current_texture() else { return; };
		let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
		let mut encoder = res.gpu.device.create_command_encoder(
			&wgpu::CommandEncoderDescriptor {
				label: Some("VxRendererEncoder")
		});

		self.retained_draw_lines.sort_if_needed();
		self.immediate_draw_lines.sort_if_needed();

		let retained_draw_lines = self.retained_draw_lines.draw_lines();
		let immediate_draw_lines = self.immediate_draw_lines.draw_lines_take();

		let mut all_draw_lines: Vec<_> = Vec::with_capacity(
			retained_draw_lines.len() + immediate_draw_lines.len()
		);

		let retained_iter = retained_draw_lines.iter().copied();
		let immediate_iter = immediate_draw_lines.into_iter();

		let merged_iter = retained_iter.merge_by(immediate_iter, |r, i| {
			let r_key = (r.z_value(), r.module_id().id());
			let i_key = (i.z_value(), i.module_id().id());
			r_key <= i_key
		});

		all_draw_lines.extend(merged_iter);

		{
			let mut render_pass = encoder.begin_render_pass(
				&wgpu::RenderPassDescriptor {
					label: Some("VxRenderPass"),
					color_attachments: &[Some(
						wgpu::RenderPassColorAttachment {
							view: &view,
							resolve_target: None,
							depth_slice: None,
							ops: wgpu::Operations {
								load: wgpu::LoadOp::Clear(
									wgpu::Color::BLACK,
								),
								store: wgpu::StoreOp::Store,
							},
						}
					)],
					..Default::default()
				}
			);

			let mut current_lines: Option<VxDrawLine> = None;

			for line in all_draw_lines.drain(..) {
				match current_lines {
					Some(ref mut batch) if batch.module_id() == line.module_id()
					&& batch.render_mode() == line.render_mode() => {
						batch.set_index_count(batch.count() + line.count());
					}
					Some(batch) => {
						self.exec_draw(res, &mut render_pass, batch.render_mode(), batch);
						current_lines = Some(line);
					}
					None => {
						current_lines = Some(line);
					}
				}
			}
			if let Some(batch) = current_lines {
				self.exec_draw(res, &mut render_pass, batch.render_mode(), batch);
			}
		}
		res.gpu.queue.submit(Some(encoder.finish()));
		frame.present();
	}

	fn exec_draw(&self, res: &VxAppResource, render_pass: &mut wgpu::RenderPass, render_mode: VxRenderMode, line: VxDrawLine) {
		let Some(module) = self.modules.get(&line.module_id()) else { return; };
		render_pass.set_pipeline(&module.pipeline);
		render_pass.set_bind_group(0, &module.projection_bind_group, &[]);

		match line.module_id() {
			VxRenderModuleId::TEXTURE => {
				render_pass.set_bind_group(1, &res.textures.module.bind_group, &[]);
			}
			VxRenderModuleId::TEXT => {
				render_pass.set_bind_group(1, &res.fonts.module.bind_group, &[]);
			}
			_ => {}
		}

		let (vertex_buffer, index_buffer) = match render_mode {
			VxRenderMode::Retained => (&module.retained_vertex_buffer, &module.retained_index_buffer),
			VxRenderMode::Immediate => (&module.immediate_vertex_buffer, &module.immediate_index_buffer),
		};

		render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
		render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
		render_pass.draw_indexed(line.start()..line.start() + line.count(), 0, 0..1);
	}

	pub fn prepare_render(&mut self, render_mode: VxRenderMode) {
		match render_mode {
			VxRenderMode::Retained => self.retained_draw_lines.clear(),
			VxRenderMode::Immediate => self.immediate_draw_lines.clear(),
		}
	}

	pub fn set_vertices<T>(
		&mut self,
		gpu: &VxGpuResource,
		render_mode: VxRenderMode,
		vertices_container: Vec<VxVertexContainer<T>>,
	) where
		T: VxVertexRenderModuleTarget,
	{
		let lines = match render_mode {
			VxRenderMode::Retained => &mut self.retained_draw_lines,
			VxRenderMode::Immediate => &mut self.immediate_draw_lines,
		};
		let Some(module) = self.modules.get_mut(&T::MODULE_ID) else { return; };
		Self::write_data_to_buffer(
			gpu,
			module,
			render_mode,
			vertices_container,
			lines
		);
	}

	fn write_data_to_buffer<T: VxVertexRenderModuleTarget>(
		gpu: &VxGpuResource,
		module: &mut VxRenderModule,
		render_mode: VxRenderMode,
		mut vertex_container: Vec<VxVertexContainer<T>>,
		draw_lines: &mut VxDrawLineContainer,
	) {
		if vertex_container.is_empty() {
			return;
		}

		let mut total_vert_len = 0;
		let mut total_index_len = 0;
		for container in &vertex_container {
			total_vert_len += container.verts.len();
			total_index_len += container.index.len();
		}

		let mut all_vertices: Vec<T> = Vec::with_capacity(total_vert_len);
		let mut all_index: Vec<u32> = Vec::with_capacity(total_index_len);
		draw_lines.reserve(vertex_container.len());

		let mut current_vertex_offset = 0u32;

		vertex_container.sort_by_key(|c| c.z_value());

		for mut container in vertex_container.drain(..) {
			let mut verts = container.verts();
			let len = verts.len() as u32;
			let before_index_len = all_index.len() as u32;
			all_vertices.append(&mut verts);

			all_index.extend(
				container.index()
					.iter()
					.map(|&i| i + current_vertex_offset)
			);

			current_vertex_offset += len;

			let draw_line = VxDrawLine::new(
				T::MODULE_ID,
				before_index_len,
				all_index.len() as u32 - before_index_len,
				container.z_value(),
				render_mode,
			);
			draw_lines.push(draw_line);
		}

		let vert_data: &[u8] = bytemuck::cast_slice(&all_vertices);
		let index_data: &[u8] = bytemuck::cast_slice(&all_index);

		// buffer size check
		let vertex_data_size = (all_vertices.len() * std::mem::size_of::<T>()) as u64;
		module.check_and_update_vertex_buffer(gpu, render_mode, vertex_data_size);

		let index_data_size = (all_index.len() * 4) as u64;
		module.check_and_update_index_buffer(gpu, render_mode, index_data_size);

		let (vertex_buffer, index_buffer) = match render_mode {
			VxRenderMode::Retained => (&module.retained_vertex_buffer, &module.retained_index_buffer),
			VxRenderMode::Immediate => (&module.immediate_vertex_buffer, &module.immediate_index_buffer)
		};

		// write data to buffer
		gpu.queue.write_buffer(vertex_buffer, 0, vert_data);
		gpu.queue.write_buffer(index_buffer, 0, index_data);
	}
}

pub trait VxVertexRenderModuleTarget: bytemuck::Pod + bytemuck::Zeroable {
	const MODULE_ID: VxRenderModuleId;
}

impl VxVertexRenderModuleTarget for VxVertex {
	const MODULE_ID: VxRenderModuleId = VxRenderModuleId::VERTEX;
}
impl VxVertexRenderModuleTarget for VxSdfVertex {
	const MODULE_ID: VxRenderModuleId = VxRenderModuleId::SDF;
}
impl VxVertexRenderModuleTarget for VxTextureVertex {
	const MODULE_ID: VxRenderModuleId = VxRenderModuleId::TEXTURE;
}
impl VxVertexRenderModuleTarget for VxTextVertex {
	const MODULE_ID: VxRenderModuleId = VxRenderModuleId::TEXT;
}