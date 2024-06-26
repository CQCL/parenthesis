use parenthesis::{from_str, to_values, ToParens, Value};

#[test]
#[cfg(feature = "macros")]
pub fn positional() {
    #[derive(ToParens)]
    pub struct Test {
        first: String,
        second: String,
    }

    let expected = from_str::<Vec<Value>>(r#""a" "b""#).unwrap();

    let exported = to_values(Test {
        first: "a".into(),
        second: "b".into(),
    });

    assert_eq!(expected, exported);
}

#[test]
#[cfg(feature = "macros")]
pub fn required() {
    #[derive(ToParens)]
    pub struct Test {
        first: String,
        #[sexpr(required)]
        required: String,
    }

    let expected = from_str::<Vec<Value>>(r#""a" (required "b")"#).unwrap();

    let exported = to_values(Test {
        first: "a".into(),
        required: "b".into(),
    });

    assert_eq!(expected, exported);
}

#[test]
#[cfg(feature = "macros")]
pub fn optional_given() {
    #[derive(ToParens)]
    pub struct Test {
        first: String,
        #[sexpr(optional)]
        optional: Option<String>,
    }

    let expected = from_str::<Vec<Value>>(r#""a" (optional "b")"#).unwrap();

    let exported = to_values(Test {
        first: "a".into(),
        optional: Some("b".into()),
    });

    assert_eq!(expected, exported);
}

#[test]
#[cfg(feature = "macros")]
pub fn optional_absent() {
    #[derive(ToParens)]
    pub struct Test {
        first: String,
        #[sexpr(optional)]
        optional: Option<String>,
    }

    let expected = from_str::<Vec<Value>>(r#""a""#).unwrap();

    let exported = to_values(Test {
        first: "a".into(),
        optional: None,
    });

    assert_eq!(expected, exported);
}

#[test]
#[cfg(feature = "macros")]
pub fn repeated() {
    #[derive(ToParens)]
    struct Test {
        first: String,
        #[sexpr(repeated)]
        field: Vec<String>,
    }

    let mut test = Test {
        first: "a".into(),
        field: Vec::new(),
    };

    let mut expected_sexpr = r#""a""#.to_string();

    for i in 0..3 {
        let expected = from_str::<Vec<Value>>(&expected_sexpr).unwrap();
        let exported = to_values(&test);

        assert_eq!(expected, exported);

        test.field.push(format!("{}", i));
        expected_sexpr.push_str(&format!(r#" (field "{}")"#, i));
    }
}
