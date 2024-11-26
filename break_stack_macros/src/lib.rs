mod component_derive;
mod bundle_files;

#[proc_macro_derive(Component, attributes(template))]
pub fn component_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse(input).unwrap();

    component_derive::impl_component_macro(&ast).into()
}

#[proc_macro]
pub fn bundle_files(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse(input).unwrap();

    bundle_files::impl_bundle_files(&ast).into()
}
