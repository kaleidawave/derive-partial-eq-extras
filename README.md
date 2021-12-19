# Derive partial eq extras

![](https://img.shields.io/crates/v/derive-partial-eq-extras)

More customisable `#[derive(PartialEq)]`

Adds ability to ignore/skip fields

### `#[partial_eq_ignore]`

```rust
use derive_partial_eq_extras::PartialEqExtras;

#[derive(PartialEqExtras)]
struct A {
    x: u32,
    #[partial_eq_ignore]
    y: String,
}
```

Here the `y` field is ignored when comparing `A`s. e.g ` A { x: 4, y: "Hello".into() } == A { x: 4, y: "World".into() }` is `true`

### `#[partial_eq_ignore_types]`

```rust
use derive_partial_eq_extras::PartialEqExtras;

#[derive(PartialEqExtras)]
#[partial_eq_ignore_types(u32)]
struct Numbers {
    name: String,
    x: u32,
    y: u32,
    z: u32,
}
```

Here the `x`, `y` and `z` fields are ignored because they have type `u32` which is marked on a top level attribute as something to ignore. This becomes a shorthand for defining `#[partial_eq_ignore]` on all fields with `u32` types
