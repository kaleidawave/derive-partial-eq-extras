use derive_partial_eq_extras::PartialEqExtras;

#[derive(PartialEqExtras, Debug)]
struct A {
    x: u32,
    #[partial_eq_ignore]
    y: String,
}

#[test]
fn ignore_field() {
    assert_eq!(
        A {
            x: 4,
            y: "Hello".into()
        },
        A {
            x: 4,
            y: "World".into()
        }
    );
    assert_ne!(
        A {
            x: 4,
            y: "Hello".into()
        },
        A {
            x: 7,
            y: "World".into()
        }
    );
}

#[derive(PartialEqExtras, Debug)]
#[partial_eq_ignore_types(String)]
struct B {
    x: u32,
    y: String,
}

#[test]
fn ignore_field_based_on_type() {
    assert_eq!(
        A {
            x: 4,
            y: "Hello".into()
        },
        A {
            x: 4,
            y: "World".into()
        }
    );
    assert_ne!(
        A {
            x: 4,
            y: "Hello".into()
        },
        A {
            x: 7,
            y: "World".into()
        }
    );
}

#[derive(PartialEqExtras, Debug)]
enum C {
    X,
    Y,
    Z,
}

#[test]
fn partial_eq_on_enum() {
    assert_eq!(C::X, C::X);
    assert_eq!(C::Y, C::Y);
    assert_ne!(C::Y, C::Z);
    assert_ne!(C::X, C::Z);
    assert_ne!(C::X, C::Y);
}

#[derive(PartialEqExtras, Debug)]
enum D {
    X,
    Y {
        a: i32,
        #[partial_eq_ignore]
        b: bool,
    },
}

#[test]
fn partial_eq_on_enum_ignored_field() {
    assert_eq!(D::X, D::X);
    assert_eq!(D::Y { a: 4, b: false }, D::Y { a: 4, b: true });
    assert_ne!(D::X, D::Y { a: 4, b: true });
    assert_ne!(D::Y { a: 6, b: false }, D::Y { a: 4, b: true });
}
