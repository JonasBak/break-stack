use super::utils::get_field_attr;
use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use std::collections::BTreeMap;
use syn::spanned::Spanned;
use syn::{Attribute, DataStruct, Expr, Field, Fields, FieldsNamed, Ident, Type};

pub fn impl_component_macro(ast: &syn::DeriveInput) -> TokenStream {
    let component_impl_component = component_impl_component(ast);
    let component_impl_component_as_ref = component_impl_component_as_ref(ast);
    let component_impl_into_response = component_impl_into_response(ast);

    let declare_component_ref = declare_component_ref(ast);
    let component_ref_impl_from_component = component_ref_impl_from_component(ast);
    let component_ref_impl = component_ref_impl(ast);
    let component_ref_impl_component = component_ref_impl_component(ast);
    let component_ref_impl_component_as_ref = component_ref_impl_component_as_ref(ast);

    let gen = quote! {
        #component_impl_component
        #component_impl_component_as_ref
        #component_impl_into_response

        #declare_component_ref
        #component_ref_impl_from_component
        #component_ref_impl
        #component_ref_impl_component
        #component_ref_impl_component_as_ref
    };

    if std::env::var("BREAK_STACK_PRINT_DERIVE")
        .map(|s| s == "1")
        .unwrap_or(false)
    {
        println!("Generated code: {}", gen.clone().to_string());
    }

    gen.into()
}

fn input_needs_lifetime(ast: &syn::DeriveInput) -> bool {
    match &ast.data {
        syn::Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => fields
            .named
            .iter()
            .filter(|field| !field_is_primitive(field))
            .next()
            .is_some(),

        _ => false,
    }
}

fn declare_component_ref(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let name = format_ident!("{}Ref", name);
    let fields = match &ast.data {
        syn::Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => fields,
        _ => todo!(),
    };
    let generics = component_ref_generics_from_types(fields);

    let generics_decl = generics.values().map(|g| component_ref_generic_decl(g));

    let fields_decl = fields.named.iter().map(|field| {
        let ident = field
            .ident
            .as_ref()
            .expect("only named fields are supported");
        if let Some((generic, _)) = generics.get(ident) {
            quote_spanned! {field.span()=>pub #ident: #generic}
        } else {
            let ty = component_ref_field_type(&field, false);
            quote_spanned! {field.span()=>pub #ident: #ty}
        }
    });

    let template_attr = template_attributes(&ast.attrs);

    let lifetime = input_needs_lifetime(ast).then(|| quote! {'a}).into_iter();

    let decl = quote! {
        #[derive(::askama_axum::Template)]
        #template_attr
        pub struct #name<#(#lifetime)* #(, #generics_decl)*> {
            #(#fields_decl,)*
        }
    };
    decl.into()
}

fn component_ref_impl(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let name = format_ident!("{}Ref", name);
    let fields = match &ast.data {
        syn::Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => fields,
        _ => todo!(),
    };
    let generics = component_ref_generics_from_types(fields);

    let generics_decl = generics.values().map(|g| component_ref_generic_decl(g));

    let generic_names = generics.values().map(|(generic, _)| generic);

    let args_decl = fields.named.iter().map(|field| {
        let ident = field
            .ident
            .as_ref()
            .expect("only named fields are supported");
        if let Some((generic, _)) = generics.get(ident) {
            quote_spanned! {field.span()=>#ident: #generic}
        } else {
            let ty = component_ref_field_type(&field, false);
            quote_spanned! {field.span()=>#ident: #ty}
        }
    });

    let args = fields.named.iter().map(|field| {
        let ident = field
            .ident
            .as_ref()
            .expect("only named fields are supported");
        quote! {#ident}
    });

    let lifetime = input_needs_lifetime(ast).then(|| quote! {'a}).into_iter();
    let lifetime2 = lifetime.clone();

    let decl = quote! {
        impl <#(#lifetime)*  #(, #generics_decl)*> #name <#(#lifetime2)* #(, #generic_names)*> {
            pub fn new(#(#args_decl),*) -> Self {
                Self {
                    #(#args,)*
                }
            }
        }
    };
    decl.into()
}

fn component_ref_impl_component(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let name = format_ident!("{}Ref", name);
    let fields = match &ast.data {
        syn::Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => fields,
        _ => todo!(),
    };
    let generics = component_ref_generics_from_types(fields);

    let generics_decl = generics.values().map(|g| component_ref_generic_decl(g));

    let generic_names = generics.values().map(|(generic, _)| generic);

    let lifetime = input_needs_lifetime(ast).then(|| quote! {'a}).into_iter();
    let lifetime2 = lifetime.clone();

    let decl = quote! {
        impl<#(#lifetime)* #(, #generics_decl)*> Component for #name<#(#lifetime2)* #(, #generic_names)*> {}
    };
    decl.into()
}

fn component_ref_impl_component_as_ref(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let name = format_ident!("{}Ref", name);
    let fields = match &ast.data {
        syn::Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => fields,
        _ => todo!(),
    };
    let generics = component_ref_generics_from_types(fields);

    let generics_decl = generics.values().map(|g| component_ref_generic_decl(g));

    let generic_names = generics.values().map(|(generic, _)| generic);

    let lifetime = input_needs_lifetime(ast).then(|| quote! {'a}).into_iter();
    let lifetime2 = lifetime.clone();

    let decl = quote! {
        impl<#(#lifetime)* #(, #generics_decl)*> ComponentAsRef for #name<#(#lifetime2)* #(, #generic_names)*> {
            type Ref = Self;

            fn as_ref(self) -> Self::Ref {
                self
            }
        }
    };
    decl.into()
}

fn component_ref_impl_from_component(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let name_ref = format_ident!("{}Ref", name);

    let fields = match &ast.data {
        syn::Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => fields,
        _ => todo!(),
    };
    let generics = component_ref_generics_from_types(fields);
    let generics_ext = generics.values().map(|(_, ref g)| match g {
        GenericForRef::Vec(ty) => quote! {Vec<#ty>},
    });

    let fields = fields.named.iter().map(|field| {
        let ident = field
            .ident
            .as_ref()
            .expect("only named fields are supported");
        if let Some(attrs) = get_field_attr(field, "component") {
            if let Some(to_ref_setter_str) = attrs.get("to_ref_setter") {
                let to_ref_setter = to_ref_setter_str
                    .parse::<Expr>()
                    .expect("to_ref_setter should be string containing valid expression");
                return quote_spanned! {to_ref_setter_str.span()=>#ident: #to_ref_setter};
            }
        }
        if field_is_primitive(field) {
            quote!(#ident: value.#ident)
        } else if matches!(field_type_path_ident(field), Some(ident) if ident == "Option") {
            quote!(#ident: value.#ident.as_ref())
        } else {
            quote!(#ident: &value.#ident)
        }
    });

    let lifetime = input_needs_lifetime(ast).then(|| quote! {'a}).into_iter();

    let decl = quote! {
        impl<'a> From<&'a #name> for #name_ref<#(#lifetime)* #(, &'a #generics_ext)*> {
            fn from(value: &'a #name) -> Self {
                Self {
                    #(#fields,)*
                }
            }
        }
    };
    decl.into()
}

fn field_type_path_ident(field: &Field) -> Option<&Ident> {
    let ty = &field.ty;
    match ty {
        Type::Path(type_path) => type_path.path.segments.first().map(|s| &s.ident),
        _ => None,
    }
}

fn component_ref_field_type(field: &Field, ref_primitive: bool) -> TokenStream {
    if let Some(attrs) = get_field_attr(field, "component") {
        if let Some(ref_type_str) = attrs.get("ref_type") {
            let ty = ref_type_str
                .parse::<Type>()
                .expect("ref_type should be string containing valid type");
            return quote_spanned! {ref_type_str.span()=>#ty};
        }
    }
    let ty = &field.ty;
    match ty {
        Type::Path(type_path) => {
            match type_path
                .path
                .segments
                .first()
                .map(|s| (&s.ident, &s.arguments))
            {
                Some((
                    ident,
                    syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
                        args,
                        ..
                    }),
                )) if ident == "Option" => match args.first() {
                    Some(syn::GenericArgument::Type(ty)) => {
                        return quote_spanned! {field.span()=> Option<&'a #ty>};
                    }
                    _ => {}
                },
                Some((ident, _)) if ident == "String" => {
                    return quote_spanned! {field.span()=> &'a str};
                }
                Some((ident, _)) if ident_type_is_primitive(ident) && !ref_primitive => {
                    return quote_spanned! {field.span()=> #ident};
                }
                _ => {}
            }
        }
        _ => {}
    }
    return quote_spanned! {field.span()=> &'a #ty};
}

enum GenericForRef {
    Vec(Type),
}

fn component_ref_generic_decl((ref ident, ref g): &(Ident, GenericForRef)) -> TokenStream {
    let r = match g {
        GenericForRef::Vec(t) => {
            quote_spanned! {ident.span()=>: IntoIterator<Item = &'a #t> + Clone}
        }
    };
    quote! {#ident #r}
}

fn component_ref_generics_from_types<'a>(
    fields: &'a FieldsNamed,
) -> BTreeMap<Ident, (Ident, GenericForRef)> {
    fields
        .named
        .iter()
        .filter(|field| {
            if let Some(attrs) = get_field_attr(field, "component") {
                if let Some(to_ref_setter_str) = attrs.get("ref_type") {
                    return false;
                }
            };
            true
        })
        .filter_map(|field| match &field.ty {
            Type::Path(type_path) => {
                match type_path
                    .path
                    .segments
                    .first()
                    .map(|s| (&s.ident, &s.arguments))
                {
                    Some((
                        ident,
                        syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
                            args,
                            ..
                        }),
                    )) if ident == "Vec" => match args.first() {
                        Some(syn::GenericArgument::Type(ty)) => Some((
                            field.ident.clone()?,
                            (
                                Ident::new(
                                    &snake_case_to_pascal_case(&field.ident.as_ref()?.to_string()),
                                    field.span(),
                                ),
                                GenericForRef::Vec(ty.clone()),
                            ),
                        )),
                        _ => None,
                    },
                    _ => None,
                }
            }
            _ => None,
        })
        .collect()
}

fn ident_type_is_primitive(ident: &Ident) -> bool {
    [
        "u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64", "i128", "isize",
        "f32", "f64", "char", "bool", "()",
    ]
    .iter()
    .find(|ty| ident == ty)
    .is_some()
}

fn field_is_primitive(field: &Field) -> bool {
    match &field.ty {
        Type::Path(type_path) => match type_path.path.segments.first().map(|s| &s.ident) {
            Some(ident) => ident_type_is_primitive(ident),
            _ => false,
        },
        _ => false,
    }
}

fn component_impl_component(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;

    let decl = quote! {
        impl Component for #name {}
    };
    decl.into()
}

fn component_impl_component_as_ref(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let name_ref = format_ident!("{}Ref", name);

    let fields = match &ast.data {
        syn::Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => fields,
        _ => todo!(),
    };
    let generics = component_ref_generics_from_types(fields);
    let generics_ext = generics.values().map(|(_, ref g)| match g {
        GenericForRef::Vec(ty) => quote! {Vec<#ty>},
    });

    let lifetime = input_needs_lifetime(ast).then(|| quote! {'a}).into_iter();

    let decl = quote! {
        impl<'a> ComponentAsRef for &'a #name {
            type Ref = #name_ref<#(#lifetime)* #(, &'a #generics_ext)*>;
            fn as_ref(self) -> Self::Ref {
                self.into()
            }
        }
    };
    decl.into()
}

fn component_impl_into_response(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;

    let decl = quote! {
        impl ::axum::response::IntoResponse for #name {
            fn into_response(self) -> askama_axum::Response {
                <&#name as ComponentAsRef>::as_ref(&self).into_response()
            }
        }
    };
    decl.into()
}

fn template_attributes(attrs: &Vec<Attribute>) -> TokenStream {
    let attr = attrs
        .iter()
        .find(|attr| attr.path().is_ident("template"))
        .expect("struct deriving Component should have a template attribute");
    quote! {
        #attr
    }
}

fn capitalize(s: &str) -> String {
    let mut i = s.chars();
    i.next()
        .map(|c| c.to_uppercase().chain(i).collect())
        .unwrap_or_default()
}

fn snake_case_to_pascal_case(s: &str) -> String {
    s.split('_').map(capitalize).collect()
}

#[cfg(test)]
mod test {
    use super::*;

    fn remove_whitespace(s: &str) -> String {
        s.replace(" ", "").replace("\n", "")
    }
    #[test]
    fn test_component_ref_field_type() {
        let field_named = syn::parse_str::<FieldsNamed>(
            r#"{
            field_a: A,
            field_b: bool,
            field_c: String,
            field_d: usize,
            field_e: Option<Abc>,
            field_f: Result<A, B>,
        }"#,
        )
        .unwrap();

        let result: Vec<_> = field_named
            .named
            .iter()
            .map(|f| component_ref_field_type(f, false).to_string())
            .collect();
        let expected = [
            "& 'a A",
            "bool",
            "& 'a str",
            "usize",
            "Option < & 'a Abc >",
            "& 'a Result < A , B >",
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_component_ref_generics_from_types() {
        let field_named = syn::parse_str::<FieldsNamed>(
            r#"{
            field_a: A,
            field_b: bool,
            field_c: String,
            field_d: usize,
            field_e: Option<Abc>,
            field_f: Result<A, B>,
            field_g: Vec<Abc>,
        }"#,
        )
        .unwrap();

        let result: Vec<_> = component_ref_generics_from_types(&field_named)
            .iter()
            .map(|(field, g)| (field.to_string(), component_ref_generic_decl(g).to_string()))
            .collect();
        let expected = [(
            "field_g".to_string(),
            "FieldG : IntoIterator < Item = & 'a Abc > + Clone".to_string(),
        )];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_declare_component_ref() {
        let derive_input = syn::parse_str(
            r#"
        #[template(source = r_"Hello World"_, ext = "html")]
        struct MyComponent {
            field_a: A,
            field_b: bool,
            field_c: String,
            field_d: usize,
            field_e: Option<Abc>,
            field_f: Result<A, B>,
            field_g: Vec<Abc>,
            #[component(ref_type = "(&'a StructA, &'a StructB)", to_ref_setter = "(&value.with_attr.0, &value.with_attr.1)")]
            with_attr: (StructA, StructB),
            #[component(ref_type = "Vec<(&'a StructA, &'a StructB)>", to_ref_setter = "TODO")]
            vec_with_attr: Vec<(StructA, StructB)>,
        }
        "#,
        )
        .unwrap();

        let result = remove_whitespace(&declare_component_ref(&derive_input).to_string());
        let expected = remove_whitespace(
            r#"
            #[derive(::askama_axum::Template)]
            #[template(source = r_"Hello World"_, ext = "html")]
            pub struct MyComponentRef<'a, FieldG: IntoIterator<Item = &'a Abc> + Clone> {
                pub field_a: &'a A,
                pub field_b: bool,
                pub field_c: &'a str,
                pub field_d: usize,
                pub field_e: Option<&'a Abc>,
                pub field_f: &'a Result<A, B>,
                pub field_g: FieldG,
                pub with_attr: (&'a StructA, &'a StructB),
                pub vec_with_attr: Vec<(&'a StructA, &'a StructB)>,
            }"#,
        );
        assert_eq!(result, expected);
    }

    #[test]
    fn test_declare_component_ref_no_fields() {
        let derive_input = syn::parse_str(
            r#"
        #[template(source = r_"Hello World"_, ext = "html")]
        struct MyComponent {
        }
        "#,
        )
        .unwrap();

        let result = remove_whitespace(&declare_component_ref(&derive_input).to_string());
        let expected = remove_whitespace(
            r#"
            #[derive(::askama_axum::Template)]
            #[template(source = r_"Hello World"_, ext = "html")]
            pub struct MyComponentRef<> {
            }"#,
        );
        assert_eq!(result, expected);
    }

    #[test]
    fn test_declare_component_ref_only_primitives() {
        let derive_input = syn::parse_str(
            r#"
        #[template(source = r_"Hello World"_, ext = "html")]
        struct MyComponent {
            pub field_a: bool,
            pub field_b: usize,
        }
        "#,
        )
        .unwrap();

        let result = remove_whitespace(&declare_component_ref(&derive_input).to_string());
        let expected = remove_whitespace(
            r#"
            #[derive(::askama_axum::Template)]
            #[template(source = r_"Hello World"_, ext = "html")]
            pub struct MyComponentRef<> {
                pub field_a: bool,
                pub field_b: usize,
            }"#,
        );
        assert_eq!(result, expected);
    }

    #[test]
    fn test_component_ref_impl() {
        let derive_input = syn::parse_str(
            r#"
        struct MyComponent {
            pub field_a: A,
            pub field_b: bool,
            pub field_c: String,
            pub field_d: usize,
            pub field_e: Option<Abc>,
            pub field_f: Result<A, B>,
            pub field_g: Vec<Abc>,
            #[component(ref_type = "(&'a StructA, &'a StructB)", to_ref_setter = "(&value.with_attr.0, &value.with_attr.1)")]
            with_attr: (StructA, StructB),
            #[component(ref_type = "Vec<(&'a StructA, &'a StructB)>", to_ref_setter = "TODO")]
            vec_with_attr: Vec<(StructA, StructB)>,
        }
        "#,
        )
        .unwrap();

        let result = remove_whitespace(&component_ref_impl(&derive_input).to_string());
        let expected = remove_whitespace(
            r#"
            impl<'a, FieldG: IntoIterator<Item = &'a Abc> + Clone> MyComponentRef<'a, FieldG> {
                pub fn new(field_a: &'a A, field_b: bool, field_c: &'a str, field_d: usize, field_e: Option<&'a Abc>, field_f: &'a Result<A, B>, field_g: FieldG, with_attr: (&'a StructA, &'a StructB), vec_with_attr: Vec<(&'a StructA, &'a StructB)>) -> Self {
                    Self {
                        field_a,
                        field_b,
                        field_c,
                        field_d,
                        field_e,
                        field_f,
                        field_g,
                        with_attr,
                        vec_with_attr,
                    }
                }
            }"#,
        );
        assert_eq!(result, expected);
    }

    #[test]
    fn test_component_ref_impl_from_component() {
        let derive_input = syn::parse_str(
            r#"
        struct MyComponent {
            pub field_a: A,
            pub field_b: bool,
            pub field_c: String,
            pub field_d: usize,
            pub field_e: Option<Abc>,
            pub field_f: Result<A, B>,
            pub field_g: Vec<Abc>,
            #[component(ref_type = "(&'a StructA, &'a StructB)", to_ref_setter = "(&value.with_attr.0, &value.with_attr.1)")]
            with_attr: (StructA, StructB),
            #[component(
                ref_type = "Vec<TestingComponentRef<'a>>",
                to_ref_setter = "value.testing.iter().map(|v| v.as_ref()).collect()"
            )]
            vec_with_attr: Vec<TestingComponent>,
        }
        "#,
        )
        .unwrap();

        let result =
            remove_whitespace(&component_ref_impl_from_component(&derive_input).to_string());
        let expected = remove_whitespace(
            r#"
        impl<'a> From<&'a MyComponent> for MyComponentRef<'a, &'a Vec<Abc>> {
            fn from(value: &'a MyComponent) -> Self {
                Self {
                    field_a: &value.field_a,
                    field_b: value.field_b,
                    field_c: &value.field_c,
                    field_d: value.field_d,
                    field_e: value.field_e.as_ref(),
                    field_f: &value.field_f,
                    field_g: &value.field_g,
                    with_attr: (&value.with_attr.0, &value.with_attr.1),
                    vec_with_attr: value.testing.iter().map(|v| v.as_ref()).collect(),
                }
            }
        }
            "#,
        );
        assert_eq!(result, expected);
    }

    #[test]
    fn test_component_impl_component_as_ref() {
        let derive_input = syn::parse_str(
            r#"
        struct MyComponent {
            pub field_a: A,
            pub field_b: bool,
            pub field_c: String,
            pub field_d: usize,
            pub field_e: Option<Abc>,
            pub field_f: Result<A, B>,
            pub field_g: Vec<Abc>,
        }
        "#,
        )
        .unwrap();

        let result = remove_whitespace(&component_impl_component_as_ref(&derive_input).to_string());
        let expected = remove_whitespace(
            r#"
        impl<'a> ComponentAsRef for &'a MyComponent {
            type Ref = MyComponentRef<'a, &'a Vec<Abc>>;

            fn as_ref(self) -> Self::Ref {
                self.into()
            }
        }
            "#,
        );
        assert_eq!(result, expected);
    }
}
