use proc_macro::TokenStream;

mod widget_macro;
mod window_macro;

#[proc_macro_derive(VxWindowImpl, attributes(attr))]
pub fn vx_window_impl(input: TokenStream) -> TokenStream {
	window_macro::vx_window_impl(input)
}

#[proc_macro_derive(VxWidgetImpl, attributes(attr))]
pub fn vx_widget_impl(input: TokenStream) -> TokenStream {
	widget_macro::vx_widget_impl(input)
}

#[proc_macro_derive(VxBoxLayoutImpl, attributes(attr_layout))]
pub fn vx_box_layout_impl(input: TokenStream) -> TokenStream {
	widget_macro::vx_box_layout_impl(input)
}

#[proc_macro_derive(VxWidgetTextImpl, attributes(attr_text))]
pub fn vx_widget_text_impl(input: TokenStream) -> TokenStream {
	widget_macro::vx_widget_text_impl(input)
}

#[proc_macro_derive(VxWidgetButtonImpl, attributes(attr_button))]
pub fn vx_widget_button_impl(input: TokenStream) -> TokenStream {
	widget_macro::vx_widget_button_impl(input)
}