use proc_macro2::TokenStream;
use quote::quote;

pub fn impl_bundle_files(ast: &syn::LitStr) -> TokenStream {
    let base_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let dir = std::path::Path::new(&base_dir).join(&ast.value());
    let files = std::fs::read_dir(&dir)
        .expect(&format!(
            "Should be able to list directory '{}'",
            dir.to_string_lossy()
        ))
        .filter_map(|path| path.ok())
        .filter(|path| path.path().is_file())
        .filter_map(|path| {
            let p = path.path();
            let relatie_path = p.strip_prefix(&dir).unwrap().to_str().unwrap();
            let inlude_path = p.to_str().unwrap();
            Some(quote! {(#relatie_path, include_bytes!(#inlude_path))})
        });
    quote! {
        [
            #(
                #files,
            )*
        ]
    }
}
