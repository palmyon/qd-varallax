use crate::{
	core::{
		gpu_resource::VxGpuResource,
		resource::VxAppResources
	},
	painter::painter::{
		VxVertexContainer,
	},
	types::{
		transform::VxMatrix4x4,
		vertex::{
			VxSdfVertex,
			VxTexVertex,
			VxVertex
		}
	}
};

pub(crate) struct VxRenderModule {
	pub(crate) module_id: VxModuleId,
	pub(crate) pipeline: wgpu::RenderPipeline,
	pub(crate) vertex_buffer: wgpu::Buffer,
	pub(crate) index_buffer: wgpu::Buffer,
	pub(crate) projection_buffer: wgpu::Buffer,
	pub(crate) projection_bind_group_layout: wgpu::BindGroupLayout,
	pub(crate) projection_bind_group: wgpu::BindGroup,
}

impl VxRenderModule {
	pub fn new(
		gpu: &VxGpuResource,
		surface_config: &wgpu::SurfaceConfiguration,
		module_id: VxModuleId,
		shader: &str,
		bind_group_layout: &[&wgpu::BindGroupLayout],
		buffers: &[wgpu::VertexBufferLayout],
		vertex_buffer_size: u64
	) -> Self {
		// create shader module
		let shader = gpu.device.create_shader_module(
			wgpu::ShaderModuleDescriptor {
				label: Some("Vx Shader Module"),
				source: wgpu::ShaderSource::Wgsl(shader.into()),
			}
		);

		// create vertex buffer
		let vertex_buffer = gpu.device.create_buffer(
			&wgpu::BufferDescriptor {
				label: Some("Vx Vertex Buffer"),
				size: vertex_buffer_size,
				usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
				mapped_at_creation: false,
			}
		);

		// create index buffer
		let index_buffer = gpu.device.create_buffer(
			&wgpu::BufferDescriptor {
				label: Some("Vx Index Buffer"),
				size: std::mem::size_of::<[u16; 10000]>() as u64,
				usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
				mapped_at_creation: false,
			}
		);

		// create projection buffer
		let projection_buffer = gpu.device.create_buffer(
			&wgpu::BufferDescriptor {
				label: Some("Vx Projection Buffer"),
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
				label: Some("Vx Projection Bind Group Layout")
			}
		);

		// create projection bind group
		let projection_bind_group = gpu.device.create_bind_group(
			&wgpu::BindGroupDescriptor {
				label: Some("Vx Projection Bind Group"),
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
				label: Some("Vx Pipeline Layout"),
				bind_group_layouts: &layouts.as_slice(),
				immediate_size: 0,
			}
		);

		// create pipeline
		let pipeline = gpu.device.create_render_pipeline(
			&wgpu::RenderPipelineDescriptor {
				label: Some("Vx Pipeline"),
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
			vertex_buffer,
			index_buffer,
			projection_buffer,
			projection_bind_group_layout,
			projection_bind_group,
		}
	}

	pub fn update_vertex_buffer(&mut self, gpu: &VxGpuResource, target_size: u64) {
		self.vertex_buffer = gpu.device.create_buffer(
			&wgpu::BufferDescriptor {
				label: Some("Vx Vertex Buffer"),
				size: (target_size as f64 * 1.5) as u64,
				usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
				mapped_at_creation: false,
			}
		)
	}

	pub fn update_index_buffer(&mut self, gpu: &VxGpuResource, target_size: u64) {
		self.index_buffer = gpu.device.create_buffer(
			&wgpu::BufferDescriptor {
				label: Some("Vx Index Buffer"),
				size: (target_size as f64 * 1.5) as u64,
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

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub(crate) enum VxModuleId {
	VertexModule,
	SdfModule,
	TextureModule,
	TextModule,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct VxDrawLine {
	module: VxModuleId,
	index_start: u32,
	index_count: u32,
	z_value: i32,
}
impl VxDrawLine {
	pub fn new(module: VxModuleId, index_start: u32, count: u32, z_value: i32) -> Self {
		Self { module, index_start, index_count: count, z_value }
	}
	pub fn module_id(&self) -> u8 {
		match self.module {
			VxModuleId::VertexModule => 0,
			VxModuleId::SdfModule => 1,
			VxModuleId::TextureModule => 2,
			VxModuleId::TextModule => 3,
		}
	}
	#[inline]
	pub fn start(&self) -> u32 {
		self.index_start
	}
	#[inline]
	pub fn count(&self) -> u32 {
		self.index_count
	}
	#[inline]
	pub fn z_value(&self) -> i32 {
		self.z_value
	}
}

pub(crate) struct VxRenderer {
	vertex_module: VxRenderModule,
	sdf_module: VxRenderModule,
	texture_module: VxRenderModule,
	text_module: VxRenderModule,
	draw_lines: Vec<VxDrawLine>,
}

impl VxRenderer {
	pub fn new(gpu: &VxGpuResource, surface_config: &wgpu::SurfaceConfiguration) -> Self {
		let vertex_module = VxRenderModule::new(
			&gpu,
			&surface_config,
			VxModuleId::VertexModule,
			include_str!("shader/vertex_shader.wgsl"),
			&[],
			&[VxVertex::VERTEXBUFFERLAYOUT],
			(std::mem::size_of::<VxVertex>() * 10000) as u64
		);

		let sdf_module = VxRenderModule::new(
			&gpu, 
			&surface_config, 
			VxModuleId::SdfModule,
			include_str!("shader/sdf_shader.wgsl"), 
			&[], 
			&[VxSdfVertex::VERTEXBUFFERLAYOUT], 
			(std::mem::size_of::<VxSdfVertex>() * 10000) as u64,
		);

		let texture_module = VxRenderModule::new(
			&gpu,
			&surface_config,
			VxModuleId::TextureModule,
			include_str!("shader/texture_shader.wgsl"),
			&[&gpu.bind_group_layout],
			&[VxTexVertex::VERTEXBUFFERLAYOUT],
			(std::mem::size_of::<VxVertex>() * 10000) as u64
		);

		let text_module = VxRenderModule::new(
			&gpu, 
			&surface_config,
			VxModuleId::TextModule,
			include_str!("shader/text_shader.wgsl"), 
			&[&gpu.bind_group_layout], 
		&[VxTexVertex::VERTEXBUFFERLAYOUT],
		(std::mem::size_of::<VxVertex>() * 10000) as u64
		);

		Self {
			vertex_module,
			sdf_module,
			texture_module,
			text_module,
			draw_lines: vec![]
		}
	}

	pub(crate) fn update_projection(&self, gpu: &VxGpuResource, orthographic: VxMatrix4x4) {
		self.vertex_module.update_projection(gpu, &orthographic);
		self.sdf_module.update_projection(gpu, &orthographic);
		self.texture_module.update_projection(gpu, &orthographic);
		self.text_module.update_projection(gpu, &orthographic);
	}

	pub fn render(&mut self, res: &mut VxAppResources, surface: &wgpu::Surface) {
		let frame = surface.get_current_texture().expect("VxRenderer> render(): Failed to [get_current_texture]");
		let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
		let mut encoder = res.gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Vx Renderer Encoder") });
		{
			let mut render_pass = encoder.begin_render_pass(
				&wgpu::RenderPassDescriptor {
					label: Some("Vx Render Pass"),
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
					depth_stencil_attachment: None,
					timestamp_writes: None,
					occlusion_query_set: None,
					multiview_mask: None,
				}
			);

			self.draw_lines.sort_by_key(|line| {
				(line.z_value(), line.module_id())
			});

			let mut current_lines: Option<VxDrawLine> = None;

			let lines: Vec<VxDrawLine> = std::mem::take(&mut self.draw_lines);

			for line in lines {
				match current_lines {
					Some(ref mut batch) if batch.module == line.module => {
						batch.index_count += line.index_count;
					}
					Some(batch) => {
						self.exec_draw(&mut render_pass, batch, &res);
						current_lines = Some(line);
					}
					None => {
						current_lines = Some(line);
					}
				}
			}
			if let Some(batch) = current_lines {
				self.exec_draw(&mut render_pass, batch, &res);
			}
		}
		res.gpu.queue.submit(Some(encoder.finish()));
		frame.present();
	}

	fn exec_draw(&self, render_pass: &mut wgpu::RenderPass, line: VxDrawLine, res: &VxAppResources) {
		let module = match line.module {
			VxModuleId::VertexModule => &self.vertex_module,
			VxModuleId::SdfModule => &self.sdf_module,
			VxModuleId::TextureModule => &self.texture_module,
			VxModuleId::TextModule => &self.text_module,
		};
		render_pass.set_pipeline(&module.pipeline);
		render_pass.set_bind_group(0, &module.projection_bind_group, &[]);

		match line.module {
			VxModuleId::TextureModule => {
				render_pass.set_bind_group(1, &res.textures.module.bind_group, &[]);
			}
			VxModuleId::TextModule => {
				render_pass.set_bind_group(1, &res.fonts.module.bind_group, &[]);
			}
			_ => {}
		}

		render_pass.set_vertex_buffer(0, module.vertex_buffer.slice(..));
		render_pass.set_index_buffer(module.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
		render_pass.draw_indexed(line.start()..line.start() + line.count(), 0, 0..1);
	}

	// 頂点, indexをバッファに書き込む
	pub fn set_vertex_vertices(
		&mut self,
		gpu: &VxGpuResource,
		vertices: Vec<VxVertexContainer<VxVertex>>,
	) {
		Self::write_data_to_buffer(gpu, &mut self.vertex_module, vertices, &mut self.draw_lines);
	}

	pub fn set_sdf_vertices(
		&mut self,
		gpu: &VxGpuResource,
		vertices: Vec<VxVertexContainer<VxSdfVertex>>,
	) {
		Self::write_data_to_buffer(gpu, &mut self.sdf_module, vertices, &mut self.draw_lines);
	}

	pub fn set_texture_vertices(
		&mut self,
		gpu: &VxGpuResource,
		vertices: Vec<VxVertexContainer<VxTexVertex>>,
	) {
		Self::write_data_to_buffer(gpu, &mut self.texture_module, vertices, &mut self.draw_lines);
	}

	pub fn set_text_vertices(
		&mut self,
		gpu: &VxGpuResource,
		vertices: Vec<VxVertexContainer<VxTexVertex>>,
	) {
		Self::write_data_to_buffer(gpu, &mut self.text_module, vertices, &mut self.draw_lines);
	}

	fn write_data_to_buffer<T: bytemuck::Pod + bytemuck::Zeroable>(
		gpu: &VxGpuResource,
		module: &mut VxRenderModule,
		mut vertex_container: Vec<VxVertexContainer<T>>,
		draw_lines: &mut Vec<VxDrawLine>,
	) {
		if vertex_container.is_empty() {
			return;
		}

		let mut all_vertices: Vec<T> = vec![];
		let mut all_index: Vec<u16> = vec![];
		let mut current_vertex_offset = 0u16;

		vertex_container.sort_by_key(|c| {
			c.z_value()
		});

		for mut container in vertex_container {
			let mut verts = container.verts();
			let len = verts.len() as u16;
			let before_index_len = all_index.len() as u32;
			all_vertices.append(&mut verts);
			for i in container.index() {
				all_index.push(i + current_vertex_offset);
			}
			current_vertex_offset += len;

			let draw_line = VxDrawLine::new(
				module.module_id,
				before_index_len,
				all_index.len() as u32 - before_index_len,
				container.z_value()
			);
			draw_lines.push(draw_line);
		}

		let vert_data: &[u8] = bytemuck::cast_slice(&all_vertices);
		let index_data: &[u8] = bytemuck::cast_slice(&all_index);

		// buffer size check
		let vertex_data_size = (all_vertices.len() * std::mem::size_of::<T>()) as u64;
		if Self::is_over_vertex_buffer_size(&module,vertex_data_size) {
			module.update_vertex_buffer(&gpu, vertex_data_size);
		}

		let index_data_size = (all_index.len() * 2) as u64;
		if Self::is_over_index_buffer_size(&module, index_data_size) {
			module.update_index_buffer(&gpu, index_data_size);
		}

		// write data to buffer
		gpu.queue.write_buffer(&module.vertex_buffer, 0, vert_data);
		gpu.queue.write_buffer(&module.index_buffer, 0, index_data);
	}

	fn is_over_vertex_buffer_size(module: &VxRenderModule, vertex_size: u64) -> bool {
		vertex_size > module.vertex_buffer.size()
	}
	fn is_over_index_buffer_size(module: &VxRenderModule, index_size: u64) -> bool {
		index_size > module.index_buffer.size()
	}
}