# kinderror

An io::Error style kind Error derive macro

## Example

```rust
use kinderror::KindError;
use std::error::Error as _;

#[derive(KindError, Debug, Eq, PartialEq)]
#[kind_error(
    source = "std::io::Error",
    source_fn = true,
    new_vis = "pub",
    name = "Error",
    type_vis = "pub",
    kind_fn_vis = "pub",
    display = "hey, error kind: {kind:?}, source: {source}"
)]
enum ErrorKind {
    First,
    Second,
}

fn main() {
    let err = Error::new(ErrorKind::First, std::io::Error::other("first error"));
    assert_eq!(*err.kind(), ErrorKind::First);
    assert!(err.source().is_some());
}
```

## License

MIT