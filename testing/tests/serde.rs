use break_stack::utils::serde::*;
use serde::Deserialize;

#[derive(Deserialize, Debug, PartialEq)]
pub struct StructWithEmptyStringAsNone {
    #[serde(deserialize_with = "empty_string_as_none")]
    field: Option<String>,
}

#[test]
fn test_empty_string_as_none() {
    assert_eq!(
        serde_json::from_str::<StructWithEmptyStringAsNone>(r#"{"field": ""}"#).unwrap(),
        StructWithEmptyStringAsNone { field: None }
    );
    assert_eq!(
        serde_json::from_str::<StructWithEmptyStringAsNone>(r#"{"field": "abc"}"#).unwrap(),
        StructWithEmptyStringAsNone {
            field: Some("abc".to_string())
        }
    );
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct StructWithFromStringEmptyStringAsNone {
    #[serde(deserialize_with = "from_string_empty_string_as_none")]
    field: Option<u8>,
}

#[test]
fn test_from_string_empty_string_as_none() {
    assert_eq!(
        serde_json::from_str::<StructWithFromStringEmptyStringAsNone>(r#"{"field": ""}"#).unwrap(),
        StructWithFromStringEmptyStringAsNone { field: None }
    );
    assert_eq!(
        serde_json::from_str::<StructWithFromStringEmptyStringAsNone>(r#"{"field": "2"}"#).unwrap(),
        StructWithFromStringEmptyStringAsNone { field: Some(2) }
    );
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct StructWithFromString {
    #[serde(deserialize_with = "from_string")]
    field: u8,
}

#[test]
fn test_from_string() {
    assert_eq!(
        serde_json::from_str::<StructWithFromString>(r#"{"field": "2"}"#).unwrap(),
        StructWithFromString { field: 2 }
    );
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct StructWithVecFromString {
    #[serde(deserialize_with = "vec_from_string")]
    field: Vec<u8>,
}

#[test]
fn test_vec_from_string() {
    assert_eq!(
        serde_json::from_str::<StructWithVecFromString>(r#"{"field": ["1", "2", "3", "4", "5"]}"#)
            .unwrap(),
        StructWithVecFromString {
            field: vec![1, 2, 3, 4, 5]
        }
    );
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct StructWithF64VecFromString {
    #[serde(deserialize_with = "vec_from_string")]
    field: Vec<f64>,
}

#[test]
fn test_f64_vec_from_string() {
    assert_eq!(
        serde_json::from_str::<StructWithF64VecFromString>(r#"{"field": ["7", "1", "2", "3.", "4.0", "5.5"]}"#)
            .unwrap(),
        StructWithF64VecFromString {
            field: vec![7.0, 1.0, 2.0, 3.0, 4.0, 5.5]
        }
    );
}
