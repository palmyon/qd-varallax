use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, parse_macro_input};

#[proc_macro_derive(VxWindowDerive, attributes(vx))]
pub fn vx_window_derive(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);
	let name = input.ident;

	let mut stat_field = None;
	let mut attr_field = None;

	if let Data::Struct(data) = input.data {
		if let Fields::Named(fields) = data.fields {
			for field in fields.named {
				let field_name = field.ident.unwrap();

				for attr in field.attrs {
					if attr.path().is_ident("vx") {
						let _ = attr.parse_nested_meta(|meta| {
							if meta.path.is_ident("Stat") {
								stat_field = Some(field_name.clone());
							} else if meta.path.is_ident("WindowAttr") {
								attr_field = Some(field_name.clone());
							}
							Ok(())
						});
					}
				}
			}
		}
	}
	
	let stat = stat_field.expect("Need #[vx(Stat)]");
	let window_attr = attr_field.expect("Need #[vx(WindowAttr)]");

	let expanded = quote! {
		impl VxWindowInternal for #name {
			fn stats(&self) -> &Option<VxWindowStats> {
				&self.#stat
			}
			fn stats_mut(&mut self) -> &mut Option<VxWindowStats> {
				&mut self.#stat
			}
			fn set_stats(&mut self, stat: VxWindowStats) {
				self.#stat = Some(stat);
			}
			fn window_attr(&self) -> &VxWindowAttributes {
				&self.#window_attr
			}
		}

		impl VxWindowBuilder for #name {
			fn build(self: Box<Self>) -> Box<dyn VxWindow> {
				self as Box<dyn VxWindow>
			}
			fn window_attr_b(&self) -> &VxWindowAttributes {
				self.window_attr()
			}
		}

		unsafe impl Send for #name {}
	};
	TokenStream::from(expanded)
}

#[proc_macro_derive(VxWidgetDerive, attributes(vx))]
pub fn vx_widget_derive(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);
	let name = input.ident;

	let mut stat_field = None;

	if let Data::Struct(data) = input.data {
		if let Fields::Named(fields) = data.fields {
			for field in fields.named {
				let field_name = field.ident.unwrap();

				for attr in field.attrs {
					if attr.path().is_ident("vx") {
						let _ = attr.parse_nested_meta(|meta|{
							if meta.path.is_ident("Stat") {
								stat_field = Some(field_name.clone());
							}
							Ok(())
						});
					}
				}
			}
		}
	}

	let stat = stat_field.expect("VxWidgetDerive> Need #[vx(Stat)] on [VxWidgetStats].");

	let expanded = quote! {
		impl VxWidgetInternal for #name {
			fn stats(&self) -> &VxWidgetStats { &self.#stat }
			fn stats_mut(&mut self) -> &mut VxWidgetStats { &mut self.#stat }
			fn as_any(&self) -> &dyn ::std::any::Any { self }
			fn as_any_mut(&mut self) -> &mut dyn ::std::any::Any { self }
		}
	};
	TokenStream::from(expanded)
}