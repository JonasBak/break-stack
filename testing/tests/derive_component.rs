use break_stack::components::*;

#[derive(Component)]
#[template(source = r#"Hello {{ name }}"#, ext = "html")]
pub struct TestingComponent {
    name: String,
    suffix: Option<char>,
}

#[test]
fn test_component() {
    assert_eq!(
        TestingComponent {
            name: "World".into(),
            suffix: None,
        }
        .as_ref()
        .to_string(),
        "Hello World"
    );
}

#[derive(Component)]
#[template(
    source = r#"{{ field_a }}{% for b in field_b.clone() %} {{ b}}{% endfor %} {{ field_a }}"#,
    ext = "html"
)]
struct TestingNestedInner {
    field_a: String,
    field_b: Vec<String>,
}

#[derive(Component)]
#[template(
    source = r#"{{ field_a }} {{ TestingNestedInnerRef::new("inner", field_b.clone()) }} {{ field_a }}"#,
    ext = "html"
)]
struct TestingNestedMiddle {
    field_a: String,
    field_b: Vec<String>,
}

#[derive(Component)]
#[template(
    source = r#"{{ field_a }} {{ TestingNestedMiddleRef::new("middle", field_b.clone()) }} {{ field_a }}"#,
    ext = "html"
)]
struct TestingNestedOuter {
    field_a: String,
    field_b: Vec<String>,
}

#[test]
fn test_nested() {
    assert_eq!(
        TestingNestedOuterRef {
            field_a: "outer",
            field_b: &vec!["A".to_string(), "B".to_string(), "C".to_string(),]
        }
        .as_ref()
        .to_string(),
        "outer middle inner A B C inner middle outer"
    );
}

#[derive(Component)]
#[template(
    source = r#"{{ count }} {% if count < 5_u8 %}{{ RecursiveComponentRef::new((count + 1).clone(), list.clone()) }}{% else %}{% for b in list.clone() %}{{ b}}{% endfor %}{% endif %}"#,
    ext = "html"
)]
struct RecursiveComponent {
    count: u8,
    list: Vec<String>,
}

#[test]
fn test_recursive() {
    assert_eq!(
        RecursiveComponent {
            count: 1,
            list: vec!["A".to_string(), "B".to_string(), "C".to_string(),]
        }
        .as_ref()
        .to_string(),
        "1 2 3 4 5 ABC"
    );
}
