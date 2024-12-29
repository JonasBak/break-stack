mod bundle_files;
mod component_derive;
mod model_derive;
mod utils;

#[proc_macro_derive(Component, attributes(template, component))]
pub fn component_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse(input).unwrap();

    component_derive::impl_component_macro(&ast).into()
}

#[proc_macro_derive(Model, attributes(model))]
pub fn model_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse(input).unwrap();

    model_derive::impl_model_macro(&ast).into()
}

#[proc_macro_derive(ModelRead, attributes(model_read))]
pub fn model_read_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse(input).unwrap();

    model_derive::impl_model_read_macro(&ast).into()
}

#[proc_macro_derive(ModelWrite, attributes(model_write))]
pub fn model_write_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse(input).unwrap();

    model_derive::impl_model_write_macro(&ast).into()
}

#[proc_macro_derive(ModelCreate, attributes(model_create))]
pub fn model_create_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse(input).unwrap();

    model_derive::impl_model_create_macro(&ast).into()
}

#[proc_macro_derive(ModelDelete, attributes(model_delete))]
pub fn model_delete_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse(input).unwrap();

    model_derive::impl_model_delete_macro(&ast).into()
}

#[proc_macro_derive(WithOwnerModel, attributes(with_owner_model))]
pub fn with_owner_model_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse(input).unwrap();

    model_derive::impl_with_owner_model_macro(&ast).into()
}

#[proc_macro]
pub fn bundle_files(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse(input).unwrap();

    bundle_files::impl_bundle_files(&ast).into()
}
