use parenthesis::{from_str, read::ReadError, FromParens, Symbol, Value};

#[test]
#[cfg(feature = "macros")]
pub fn positional() {
    #[derive(FromParens)]
    struct Test {
        first: Symbol,
        second: String,
    }

    let test = from_str::<Test>(r#"symbol "string""#).unwrap();

    assert_eq!(test.first, Symbol::new("symbol"));
    assert_eq!(test.second, "string");
}

#[test]
#[cfg(feature = "macros")]
pub fn optional_given() {
    #[derive(FromParens)]
    struct Test {
        #[sexpr(optional)]
        field: Option<String>,
    }

    let test = from_str::<Test>(r#"(field "string")"#).unwrap();

    assert_eq!(test.field.unwrap(), "string");
}

#[test]
#[cfg(feature = "macros")]
pub fn optional_absent() {
    #[derive(FromParens)]
    struct Test {
        #[sexpr(optional)]
        field: Option<String>,
    }

    let test = from_str::<Test>(r#""#).unwrap();

    assert_eq!(test.field, None);
}

#[test]
#[cfg(feature = "macros")]
pub fn optional_duplicate() {
    #[derive(Debug, FromParens)]
    struct Test {
        #[allow(dead_code)]
        #[sexpr(optional)]
        field: Option<String>,
    }

    let text = r#"(field "string") (field "another")"#;
    let result = from_str::<Test>(text);

    println!("{:#?}", from_str::<Vec<Value>>(text));

    assert!(matches!(result, Err(ReadError::Parse(_))));
}

#[test]
#[cfg(feature = "macros")]
pub fn required_given() {
    #[derive(FromParens)]
    struct Test {
        #[sexpr(required)]
        field: String,
    }

    let test = from_str::<Test>(r#"(field "string")"#).unwrap();

    assert_eq!(test.field, "string");
}

#[test]
#[cfg(feature = "macros")]
pub fn required_absent() {
    #[derive(FromParens)]
    struct Test {
        #[allow(dead_code)]
        #[sexpr(required)]
        field: String,
    }

    let result = from_str::<Test>(r#""#);

    assert!(matches!(result, Err(ReadError::Parse(_))));
}

#[test]
#[cfg(feature = "macros")]
pub fn required_duplicate() {
    #[derive(FromParens)]
    struct Test {
        #[allow(dead_code)]
        #[sexpr(required)]
        field: String,
    }

    let result = from_str::<Test>(r#"(field "string") (field "another")"#);

    assert!(matches!(result, Err(ReadError::Parse(_))));
}

#[test]
#[cfg(feature = "macros")]
pub fn repeated() {
    #[derive(FromParens)]
    struct Test {
        #[sexpr(repeated)]
        values: Vec<String>,
    }

    let mut text = String::new();
    let mut expected = Vec::new();

    for i in 0..3 {
        let test = from_str::<Test>(&text).unwrap();
        assert_eq!(test.values, expected);

        text.push_str(&format!(r#" (values "{}")"#, i));
        expected.push(format!("{}", i));
    }
}

#[test]
#[cfg(feature = "macros")]
pub fn resursive_field() {
    #[derive(FromParens, PartialEq, Eq, Debug)]
    struct Outer {
        #[sexpr(repeated)]
        inner: Vec<Inner>,
    }

    #[derive(FromParens, PartialEq, Eq, Debug)]
    struct Inner {
        positional: Symbol,
        #[sexpr(required)]
        field: String,
    }

    let text = r#"
        (inner first (field "first"))
        (inner second (field "second"))
    "#;

    let expected = Outer {
        inner: vec![
            Inner {
                positional: "first".into(),
                field: "first".into(),
            },
            Inner {
                positional: "second".into(),
                field: "second".into(),
            },
        ],
    };

    let test = from_str::<Outer>(text).unwrap();
    assert_eq!(test, expected);
}
