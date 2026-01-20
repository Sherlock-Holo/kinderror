#![allow(unused)]

use std::error::Error as _;
use std::fmt::{Debug, Display, Formatter};
use std::io;

use kinderror::KindError;

#[derive(KindError, Debug, Eq, PartialEq)]
#[kind_error(
    source = "io::Error",
    new_vis = "pub",
    name = "Error",
    type_vis = "pub",
    kind_fn_vis = "pub"
)]
enum ErrorKind {
    First,
    Second,
}

#[test]
fn test() {
    let err = Error::new(ErrorKind::First, io::Error::other("err"));
    assert_eq!(*err.kind(), ErrorKind::First);
    assert!(err.source().is_some());
}

// Test custom error type (does not implement Error trait)
#[derive(Debug, Clone, Copy)]
struct CustomError;

#[derive(KindError, Debug, Eq, PartialEq)]
#[kind_error(source = "CustomError", source_fn = false, name = "CustomErrorWrapper")]
enum CustomErrorKind {
    Kind1,
    Kind2,
}

#[test]
fn test_source_fn_false() {
    let err = CustomErrorWrapper::new(CustomErrorKind::Kind1, CustomError);
    assert_eq!(*err.kind(), CustomErrorKind::Kind1);
    assert!(err.source().is_none());
}

// Test custom display format - using {:?} debug format
#[derive(KindError, Debug)]
#[kind_error(
    source = "io::Error",
    name = "CustomDisplayError",
    type_vis = "pub",
    display = "Custom error: kind={kind:?}, source={source:?}"
)]
enum CustomDisplayKind {
    VariantA,
    VariantB,
}

#[test]
fn test_custom_display() {
    let err = CustomDisplayError::new(CustomDisplayKind::VariantA, io::Error::other("test error"));
    let display_str = format!("{}", err);
    // Use the complete format string provided by the user
    assert!(display_str.contains("Custom error:"));
    assert!(display_str.contains("VariantA"));
    assert!(display_str.contains("test error"));
}

// Test custom display format - using different format specifiers
#[derive(KindError, Debug)]
#[kind_error(
    source = "io::Error",
    name = "CustomDisplayError2",
    type_vis = "pub",
    display = "Error [kind={kind}]: {source}"
)]
enum CustomDisplayKind2 {
    ErrorType1,
    ErrorType2,
}

impl Display for CustomDisplayKind2 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}

#[test]
fn test_custom_display_different_format() {
    let err = CustomDisplayError2::new(
        CustomDisplayKind2::ErrorType1,
        io::Error::other("some issue"),
    );
    let display_str = format!("{}", err);
    // Use different format specifiers
    assert!(display_str.contains("Error [kind="));
    assert!(display_str.contains("ErrorType1"));
    assert!(display_str.contains("some issue"));
}

// Test default display format (without specifying display attribute)
#[derive(KindError, Debug)]
#[kind_error(source = "io::Error", name = "DefaultDisplayError", type_vis = "pub")]
enum DefaultDisplayKind {
    ErrorA,
    ErrorB,
}

#[test]
fn test_default_display() {
    let err = DefaultDisplayError::new(
        DefaultDisplayKind::ErrorA,
        io::Error::other("default error"),
    );
    let display_str = format!("{}", err);
    // Default format should be "error kind: ..., source: ..."
    assert!(display_str.contains("error kind:"));
    assert!(display_str.contains("source:"));
    assert!(display_str.contains("ErrorA"));
    assert!(display_str.contains("default error"));
}

#[derive(KindError, Debug, Eq, PartialEq)]
#[kind_error(source = "io::Error", name = "HasFieldError")]
enum HasFieldErrorKind {
    Field1 { text: String },
    Field2,
}

#[test]
fn test_has_field() {
    let err = HasFieldError::new(
        HasFieldErrorKind::Field1 {
            text: "hello".to_string(),
        },
        io::Error::other("err"),
    );
    assert_eq!(
        *err.kind(),
        HasFieldErrorKind::Field1 {
            text: "hello".to_string()
        }
    );
}
