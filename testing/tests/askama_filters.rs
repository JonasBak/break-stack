use askama_axum::Template;
use break_stack::utils::askama::filters;

#[derive(Template)]
#[template(source = r#"{{ field|string_or_empty }}"#, ext = "html")]
struct TemplateWithStringOrEmpty {
    field: Option<String>,
}

#[derive(Template)]
#[template(source = r#"{{ field|string_or_empty }}"#, ext = "html")]
struct TemplateWithStringOrEmptyRefStr<'a> {
    field: Option<&'a str>,
}

#[test]
fn test_string_or_empty() {
    assert_eq!(TemplateWithStringOrEmpty { field: None }.to_string(), "");
    assert_eq!(
        TemplateWithStringOrEmpty {
            field: Some("abc".to_string())
        }
        .to_string(),
        "abc"
    );
    assert_eq!(
        TemplateWithStringOrEmptyRefStr { field: None }.to_string(),
        ""
    );
    assert_eq!(
        TemplateWithStringOrEmptyRefStr { field: Some("abc") }.to_string(),
        "abc"
    );
}

#[derive(Template)]
#[template(source = r#"{{ field|some_matches(compare_with) }}"#, ext = "html")]
struct TemplateWithSomeMatches {
    field: Option<u8>,
    compare_with: u8,
}

#[test]
fn test_with_some_matches() {
    assert_eq!(
        TemplateWithSomeMatches {
            field: None,
            compare_with: 1
        }
        .to_string(),
        "false"
    );
    assert_eq!(
        TemplateWithSomeMatches {
            field: Some(2),
            compare_with: 1
        }
        .to_string(),
        "false"
    );
    assert_eq!(
        TemplateWithSomeMatches {
            field: Some(1),
            compare_with: 1
        }
        .to_string(),
        "true"
    );
}

#[derive(Template)]
#[template(source = r#"{{ field|string_if_true("abc") }}"#, ext = "html")]
struct TemplateWithStringIfTrue {
    field: bool,
}

#[test]
fn test_string_if_true() {
    assert_eq!(TemplateWithStringIfTrue { field: false }.to_string(), "");
    assert_eq!(TemplateWithStringIfTrue { field: true }.to_string(), "abc");
}
