use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, parse_macro_input};
	
pub fn vx_window_impl(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);
	let name = input.ident;

	let mut ctx_field = None;
	let mut attr_field = None;

	if let Data::Struct(data) = input.data {
		if let Fields::Named(fields) = data.fields {
			for field in fields.named {
				let field_name = field.ident.unwrap();

				for attr in field.attrs {
					if attr.path().is_ident("attr") {
						let _ = attr.parse_nested_meta(|meta| {
							if meta.path.is_ident("context") {
								ctx_field = Some(field_name.clone());
							} else if meta.path.is_ident("options") {
								attr_field = Some(field_name.clone());
							}
							Ok(())
						});
					}
				}
			}
		}
	}
	
	let context = ctx_field.expect("VxWindowImpl> Need #[attr(context)] on [Option<VxWindowContext>].");
	let window_attr = attr_field.expect("VxWindowImpl> Need #[attr(options)] on [VxWindowAttributes].");

	let expanded = quote! {
		impl VxWindowAccessor for #name {
			#[inline]
			fn context(&self) -> &Option<VxWindowContext> {
				&self.#context
			}
			#[inline]
			fn context_mut(&mut self) -> &mut Option<VxWindowContext> {
				&mut self.#context
			}
			#[inline]
			fn set_context(&mut self, context: VxWindowContext) {
				self.#context = Some(context);
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