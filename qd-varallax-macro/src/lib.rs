use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, parse_macro_input};

#[proc_macro_derive(VxWindowImpl, attributes(attr))]
pub fn vx_window_impl(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);
	let name = input.ident;

	let mut stat_field = None;
	let mut attr_field = None;

	if let Data::Struct(data) = input.data {
		if let Fields::Named(fields) = data.fields {
			for field in fields.named {
				let field_name = field.ident.unwrap();

				for attr in field.attrs {
					if attr.path().is_ident("attr") {
						let _ = attr.parse_nested_meta(|meta| {
							if meta.path.is_ident("stat") {
								stat_field = Some(field_name.clone());
							} else if meta.path.is_ident("w_attr") {
								attr_field = Some(field_name.clone());
							}
							Ok(())
						});
					}
				}
			}
		}
	}
	
	let stat = stat_field.expect("VxWindowImpl> Need #[attr(stat)] on [Option<VxWindowStats>].");
	let window_attr = attr_field.expect("VxWindowImpl> Need #[attr(w_attr)] on [VxWindowAttributes].");

	let expanded = quote! {
		impl VxWindowAccessor for #name {
			#[inline]
			fn stats(&self) -> &Option<VxWindowStats> {
				&self.#stat
			}
			#[inline]
			fn stats_mut(&mut self) -> &mut Option<VxWindowStats> {
				&mut self.#stat
			}
			#[inline]
			fn set_stats(&mut self, stat: VxWindowStats) {
				self.#stat = Some(stat);
			}
			#[inline]
			fn window_attr(&self) -> &VxWindowAttributes {
				&self.#window_attr
			}
		}

		impl VxWindowBuilder for #name {
			#[inline]
			fn build(self: Box<Self>) -> Box<dyn VxWindow> {
				self as Box<dyn VxWindow>
			}
			#[inline]
			fn window_attr_b(&self) -> &VxWindowAttributes {
				self.window_attr()
			}
		}

		unsafe impl Send for #name {}
	};
	TokenStream::from(expanded)
}

#[proc_macro_derive(VxWidgetImpl, attributes(attr))]
pub fn vx_widget_impl(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);
	let name = input.ident;

	let mut stat_field = None;

	if let Data::Struct(data) = input.data {
		if let Fields::Named(fields) = data.fields {
			for field in fields.named {
				let field_name = field.ident.unwrap();

				for attr in field.attrs {
					if attr.path().is_ident("attr") {
						let _ = attr.parse_nested_meta(|meta|{
							if meta.path.is_ident("stat") {
								stat_field = Some(field_name.clone());
							}
							Ok(())
						});
					}
				}
			}
		}
	}

	let stat = stat_field.expect("VxWidgetImpl> Need #[attr(stat)] on [VxWidgetStats].");

	let expanded = quote! {
		impl VxWidgetAccessor for #name {
			#[inline]
			fn stats(&self) -> &VxWidgetStats { &self.#stat }
			#[inline]
			fn stats_mut(&mut self) -> &mut VxWidgetStats { &mut self.#stat }
			#[inline]
			fn as_any(&self) -> &dyn ::std::any::Any { self }
			#[inline]
			fn as_any_mut(&mut self) -> &mut dyn ::std::any::Any { self }
			#[inline]
			fn into_any(self: Box<Self>) -> Box<dyn ::std::any::Any> { self }
		}
	};
	TokenStream::from(expanded)
}

#[proc_macro_derive(VxBoxLayoutImpl, attributes(attr_layout))]
pub fn vx_box_layout_impl(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);
	let name = input.ident;

	let mut stat_field = None;

	if let Data::Struct(data) = input.data {
		if let Fields::Named(fields) = data.fields {
			for field in fields.named {
				let field_name = field.ident.unwrap();

				for attr in field.attrs {
					if attr.path().is_ident("attr_layout") {
						let _ = attr.parse_nested_meta(|meta| {
							if meta.path.is_ident("stat") {
								stat_field = Some(field_name.clone());
							}
							Ok(())
						});
					}
				}
			}
		}
	}

	let stat = stat_field.expect("VxBoxLayoutImpl> Need #[attr_layout(stat)] on [VxBoxLayoutStats].");

	let expanded = quote! {
		impl VxBoxLayoutAccessor for #name {
			#[inline]
			fn layout_stats(&self) -> &VxBoxLayoutStats { &self.#stat }
			#[inline]
			fn layout_stats_mut(&mut self) -> &mut VxBoxLayoutStats { &mut self.#stat }
			#[inline]
			fn as_any_layout(&self) -> &dyn ::std::any::Any { self }
			#[inline]
			fn as_any_layout_mut(&mut self) -> &mut dyn ::std::any::Any { self }
		}
	};
	TokenStream::from(expanded)
}