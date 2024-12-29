use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use std::collections::BTreeMap;
use std::collections::HashMap;
use syn::spanned::Spanned;
use syn::{
    parse::Parser, punctuated::Punctuated, token::Comma, Attribute, DataStruct, Expr, Field,
    Fields, FieldsNamed, Ident, Lit, LitStr, MetaList, MetaNameValue, Type,
};

pub fn get_attr(attrs: &Vec<Attribute>, attr_name: &str) -> Option<HashMap<String, LitStr>> {
    let mut matched_args = None;
    for attr in attrs {
        if !attr.path().is_ident(attr_name) {
            continue;
        }

        match attr.parse_args_with(Punctuated::<MetaNameValue, syn::Token![,]>::parse_terminated) {
            Ok(args) if matched_args.is_none() => matched_args = Some(args),
            Ok(_) => panic!("duplicated '{}' attribute", attr_name),
            Err(e) => panic!("unable to parse template arguments: {e}"),
        };
    }
    Some(
        matched_args?
            .into_iter()
            .map(|kv| {
                (
                    kv.path
                        .require_ident()
                        .expect("key should be simple ident")
                        .to_string(),
                    match kv.value {
                        Expr::Lit(syn::ExprLit {
                            lit: Lit::Str(lit_str),
                            ..
                        }) => lit_str,
                        _ => panic!(),
                    },
                )
            })
            .collect(),
    )
}

pub fn get_input_attr(ast: &syn::DeriveInput, attr_name: &str) -> Option<HashMap<String, LitStr>> {
    get_attr(&ast.attrs, attr_name)
}

pub fn get_field_attr(field: &Field, attr_name: &str) -> Option<HashMap<String, LitStr>> {
    get_attr(&field.attrs, attr_name)
}
