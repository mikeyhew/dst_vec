# dst_vec
Provides a contiguos list data structure for `?Sized` types in Rust. The name might be a bit misleading, but it's called that because it's contiguos, like `Vec`.

## Usage

Add this to your Cargo.toml:

```
dst_vec = { git = "https://github.com/mikeyhew/dst_vec" }
```

To use it with trait objects, derive the `Referent` trait with `derive_referent!` from the `referent` crate:

```
#[macro_use] extern crate referent;

// you need your own trait in order to implement `Referent` on it
trait MyTrait: Foo + Bar + Baz {}

derive_referent!(MyTrait);
```

## Docs

Until this gets published on cargo, you'll have to build the docs yourself and open them with

```
cargo doc --package dst_vec --open
```

## Examples

For an example using `FnOnce` closures, see [tests/callbacks.rs](./tests/callbacks.rs)

## Questions, Help

Post an issue here or ping `mikeyhew` on IRC

## Contributing

Some things that probably need to be done

- Creating a queue data structure would also be nice
- Figure out if there's a way to store everything in a single buffer, instead of using a separate `Vec` for pointer metadata and offsets
- There's probably quite a few traits that should be implemented
- Improve the `derive_referent!` macro (located at https://github.com/mikeyhew/referent_trait)
