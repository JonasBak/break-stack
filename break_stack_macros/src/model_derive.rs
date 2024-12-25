use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use std::collections::BTreeMap;
use std::collections::HashMap;
use syn::spanned::Spanned;
use syn::{
    parse::Parser, punctuated::Punctuated, token::Comma, Attribute, DataStruct, Expr, Field,
    Fields, FieldsNamed, Ident, Lit, LitStr, MetaList, MetaNameValue, Type,
};

fn get_attr(ast: &syn::DeriveInput, attr_name: &str) -> Option<HashMap<String, LitStr>> {
    let mut matched_args = None;
    for attr in &ast.attrs {
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
                    kv.path.require_ident().expect("key should be simple ident").to_string(),
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

pub fn impl_model_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;

    let args = get_attr(ast, "model").expect("deriving Model requires a model attribute");

    let model_name = args.get("name").expect("model attribute requires a field called name");

    let gen = quote! {
        impl Model for #name {
            const MODEL_NAME: &'static str = #model_name;
        }
    };

    if std::env::var("BREAK_STACK_PRINT_DERIVE")
        .map(|s| s == "1")
        .unwrap_or(false)
    {
        println!("Generated code: {}", gen.clone().to_string());
    }

    gen.into()
}

pub fn impl_model_read_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;

    let args = get_attr(ast, "model_read").expect("deriving ModelRead requires a model_read attribute");

    let query = args.get("query").expect("model_read attribute requires a field called query");

    let gen = quote! {
        impl ModelRead for #name {
            async fn read(conn: &mut ::break_stack::models::DBConn, id: i64) -> Result<Option<Self>, ::break_stack::errors::ModelError> {
                let row = sqlx::query_as!(Self, #query, id)
                    .fetch_optional(&mut **conn)
                    .await?;

                Ok(row)
            }
        }
    };

    if std::env::var("BREAK_STACK_PRINT_DERIVE")
        .map(|s| s == "1")
        .unwrap_or(false)
    {
        println!("Generated code: {}", gen.clone().to_string());
    }

    gen.into()
}

pub fn impl_model_write_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;

    let args = get_attr(ast, "model_write").expect("deriving ModelWrite requires a model_write attribute");

    let query = args.get("query").expect("model_write attribute requires a field called query");
    let data_type = args
        .get("data_type")
        .expect("model_write attribute requires a field called data_type")
        .parse::<Ident>()
        .unwrap();
    let fields = args
        .get("fields")
        .expect("model_write attribute requires a field called fields")
        .parse_with(Punctuated::<Expr, Comma>::parse_terminated)
        .unwrap();

    let gen = quote! {
        impl ModelWrite for #name {
            type Write = #data_type;

            async fn write(
                conn: &mut ::break_stack::models::DBConn,
                id: i64,
                data: Self::Write,
            ) -> Result<Option<Self>, ::break_stack::errors::ModelError> {
                let row = sqlx::query_as!(
                    Self,
                    #query,
                    #fields,
                )
                .fetch_optional(&mut **conn)
                .await?;

                Ok(row)
            }
        }
    };

    if std::env::var("BREAK_STACK_PRINT_DERIVE")
        .map(|s| s == "1")
        .unwrap_or(false)
    {
        println!("Generated code: {}", gen.clone().to_string());
    }

    gen.into()
}


pub fn impl_model_create_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;

    let args = get_attr(ast, "model_create").expect("deriving ModelCreate requires a model_create attribute");

    let query = args.get("query").expect("model_create attribute requires a field called query");
    let data_type = args
        .get("data_type")
        .expect("model_create attribute requires a field called data_type")
        .parse::<Ident>()
        .unwrap();
    let fields = args
        .get("fields")
        .expect("model_create attribute requires a field called fields")
        .parse_with(Punctuated::<Expr, Comma>::parse_terminated)
        .unwrap();

    let gen = quote! {
        impl ModelCreate for #name {
            type Create = #data_type;

            async fn create(
                conn: &mut ::break_stack::models::DBConn,
                data: Self::Create,
            ) -> Result<Self, ::break_stack::errors::ModelError> {
                let t = sqlx::query_as!(
                    Self,
                    #query,
                    #fields,
                )
                .fetch_one(&mut **conn)
                .await?;
                Ok(t)
            }
        }
    };

    if std::env::var("BREAK_STACK_PRINT_DERIVE")
        .map(|s| s == "1")
        .unwrap_or(false)
    {
        println!("Generated code: {}", gen.clone().to_string());
    }

    gen.into()
}

#[cfg(test)]
mod test {
    use super::*;

    fn remove_whitespace(s: &str) -> String {
        s.replace(" ", "").replace("\n", "")
    }
    #[test]
    fn test_impl_model_macro() {
        let input = syn::parse_str::<syn::DeriveInput>(
            r#"
            #[derive(Model)]
            #[model(name = "Test")]
            struct TestModel {
                pub id: i64,
                pub field: String,
            }
            "#,
        )
        .unwrap();

        let result = impl_model_macro(&input);
        let expected = r#"
            impl Model for TestModel {
                const MODEL_NAME: &'static str = "Test";
            }
            "#;

        assert_eq!(
            remove_whitespace(&result.to_string()),
            remove_whitespace(&expected.to_string())
        );
    }

    #[test]
    fn test_impl_model_read_macro() {
        let input = syn::parse_str::<syn::DeriveInput>(
            r#"
            #[derive(ModelRead)]
            #[model_read(query = "SELECT * FROM test WHERE id = ?")]
            struct TestModel {
                pub id: i64,
                pub field: String,
            }
            "#,
        )
        .unwrap();

        let result = impl_model_read_macro(&input);
        let expected = r#"
            impl ModelRead for TestModel {
                async fn read(conn: &mut ::break_stack::models::DBConn, id: i64) -> Result<Option<Self>, ::break_stack::errors::ModelError> {
                    let row = sqlx::query_as!(Self, "SELECT * FROM test WHERE id = ?", id)
                        .fetch_optional(&mut **conn)
                        .await?;

                    Ok(row)
                }
            }
            "#;

        assert_eq!(
            remove_whitespace(&result.to_string()),
            remove_whitespace(&expected.to_string())
        );
    }

    #[test]
    fn test_impl_model_write_macro() {
        let input = syn::parse_str::<syn::DeriveInput>(
            r#"
            #[derive(ModelWrite)]
            #[model_write(
                data_type = "TestModelWrite",
                query = "UPDATE test SET field = ? WHERE id = ? RETURNING *",
                fields = "data.field, id",
            )]
            struct TestModel {
                pub id: i64,
                pub field: String,
            }
            "#,
        )
        .unwrap();

        let result = impl_model_write_macro(&input);
        let expected = r#"
            impl ModelWrite for TestModel {
                type Write = TestModelWrite;

                async fn write(
                    conn: &mut ::break_stack::models::DBConn,
                    id: i64,
                    data: Self::Write,
                ) -> Result<Option<Self>, ::break_stack::errors::ModelError> {
                    let row = sqlx::query_as!(
                        Self,
                        "UPDATE test SET field = ? WHERE id = ? RETURNING *",
                        data.field,
                        id,
                    )
                    .fetch_optional(&mut **conn)
                    .await?;

                    Ok(row)
                }
            }
            "#;

        assert_eq!(
            remove_whitespace(&result.to_string()),
            remove_whitespace(&expected.to_string())
        );
    }

    #[test]
    fn test_impl_model_create_macro() {
        let input = syn::parse_str::<syn::DeriveInput>(
            r#"
            #[derive(ModelCreate)]
            #[model_create(
                data_type = "TestModelCreate",
                query = "INSERT INTO test (field) VALUES (?) RETURNING *",
                fields = "data.field",
            )]
            struct TestModel {
                pub id: i64,
                pub field: String,
            }
            "#,
        )
        .unwrap();

        let result = impl_model_create_macro(&input);
        let expected = r#"
            impl ModelCreate for TestModel {
                type Create = TestModelCreate;

                async fn create(
                    conn: &mut ::break_stack::models::DBConn,
                    data: Self::Create,
                ) -> Result<Self, ::break_stack::errors::ModelError> {
                    let t = sqlx::query_as!(
                        Self,
                        "INSERT INTO test (field) VALUES (?) RETURNING *",
                        data.field,
                    )
                    .fetch_one(&mut **conn)
                    .await?;
                    Ok(t)
                }
            }
            "#;

        assert_eq!(
            remove_whitespace(&result.to_string()),
            remove_whitespace(&expected.to_string())
        );
    }
}
