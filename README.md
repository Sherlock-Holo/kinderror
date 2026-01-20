# kinderror

An io::Error style kind Error derive macro

## Example

```rust
use kinderror::KindError;

#[derive(KindError, Debug)]
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
```

## License

MIT