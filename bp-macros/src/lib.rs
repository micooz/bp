use proc_macro::TokenStream;

#[proc_macro_derive(Protocol)]
pub fn derive_custom_protocol_struct(_item: TokenStream) -> TokenStream {
    todo!()
}
