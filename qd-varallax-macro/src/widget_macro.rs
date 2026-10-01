use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, parse_macro_input};

pub fn vx_widget_impl(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);
	let name = input.ident;

	let mut ctx_field = None;

	if let Data::Struct(data) = input.data {
		if let Fields::Named(fields) = data.fields {
			for field in fields.named {
				let field_name = field.ident.unwrap();

				for attr in field.attrs {
					if attr.path().is_ident("attr") {
						let _ = attr.parse_nested_meta(|meta|{
							if meta.path.is_ident("context") {
								ctx_field = Some(field_name.clone());
							}
							Ok(())
						});
					}
				}
			}
		}
	}

	let ctx = ctx_field.expect("VxWidgetImpl> Need #[attr(context)] on [VxWidgetContext].");

	let expanded = quote! {
		impl VxWidgetAccessor for #name {
			#[inline]
			fn context(&self) -> &VxWidgetContext { &self.#ctx }
			#[inline]
			fn context_mut(&mut self) -> &mut VxWidgetContext { &mut self.#ctx }
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

pub fn vx_box_layout_impl(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);
	let name = input.ident;

	let mut ctx_field = None;

	if let Data::Struct(data) = input.data {
		if let Fields::Named(fields) = data.fields {
			for field in fields.named {
				let field_name = field.ident.unwrap();

				for attr in field.attrs {
					if attr.path().is_ident("attr_layout") {
						let _ = attr.parse_nested_meta(|meta| {
							if meta.path.is_ident("context") {
								ctx_field = Some(field_name.clone());
							}
							Ok(())
						});
					}
				}
			}
		}
	}

	let ctx = ctx_field.expect("VxBoxLayoutImpl> Need #[attr_layout(context)] on [VxBoxLayoutContext].");

	let expanded = quote! {
		impl VxBoxLayoutAccessor for #name {
			#[inline]
			fn layout_context(&self) -> &VxBoxLayoutContext { &self.#ctx }
			#[inline]
			fn layout_context_mut(&mut self) -> &mut VxBoxLayoutContext { &mut self.#ctx }
			#[inline]
			fn as_any_layout(&self) -> &dyn ::std::any::Any { self }
			#[inline]
			fn as_any_layout_mut(&mut self) -> &mut dyn ::std::any::Any { self }
		}
	};
	TokenStream::from(expanded)
}

pub fn vx_widget_text_impl(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);
	let name = input.ident;

	let mut component_field = None;

	if let Data::Struct(data) = input.data {
		if let Fields::Named(fields) = data.fields {
			for field in fields.named {
				let field_name = field.ident.unwrap();

				for attr in field.attrs {
					if attr.path().is_ident("attr_text") {
						let _ = attr.parse_nested_meta(|meta| {
							if meta.path.is_ident("component") {
								component_field = Some(field_name.clone());
							}
							Ok(())
						});
					}
				}
			}
		}
	}

	let component = component_field.expect("VxWidgetTextImpl> Need #[attr_text(component)] on [VxTextWidgetComponent].");

	let expanded = quote! {
		impl VxWidgetTextExtension for #name {
			#[inline]
			fn text_component(&self) -> &VxTextWidgetComponent {
				&self.#component
			}
			#[inline]
			fn text_component_mut(&mut self) -> &mut VxTextWidgetComponent {
				&mut self.#component
			}
		}
	};
	TokenStream::from(expanded)
}

pub fn vx_widget_button_impl(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);
	let name = input.ident;

	let mut component_field = None;

	if let Data::Struct(data) = input.data {
		if let Fields::Named(fields) = data.fields {
			for field in fields.named {
				let field_name = field.ident.unwrap();

				for attr in field.attrs {
					if attr.path().is_ident("attr_button") {
						let _ = attr.parse_nested_meta(|meta| {
							if meta.path.is_ident("component") {
								component_field = Some(field_name.clone());
							}
							Ok(())
						});
					}
				}
			}
		}
	}

	let component = component_field.expect("VxWidgetButtonImpl> Need #[attr_button(component)] on [VxButtonWidgetComponent].");

	let expanded = quote! {
		impl VxWidgetButtonExtension for #name {
			#[inline]
			fn button_component(&self) -> &VxButtonWidgetComponent {
				&self.#component
			}
			#[inline]
			fn button_component_mut(&mut self) -> &mut VxButtonWidgetComponent {
				&mut self.#component
			}
		}
	};
	TokenStream::from(expanded)
}