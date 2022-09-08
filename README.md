# nom_bencode

[![Version](https://img.shields.io/crates/v/nom_bencode)](https://crates.io/crates/nom_bencode)
[![Downloads](https://img.shields.io/crates/d/nom_bencode)](https://crates.io/crates/nom_bencode)
[![License](https://img.shields.io/crates/l/nom_bencode)](https://crates.io/crates/nom_bencode)
![Rust](https://github.com/edg-l/nom-bencode/workflows/Rust/badge.svg)
[![Docs](https://docs.rs/nom_bencode/badge.svg)](https://docs.rs/nom_bencode)

A bencode parser written with nom.
```rust
use nom_bencode::Value;

let data = nom_bencode::parse(b"d3:cow3:moo4:spam4:eggse").unwrap();
let v = data.first().unwrap();

if let Value::Dictionary(dict) = v {
    let v = dict.get(b"cow").unwrap();

    if let Value::Bytes(data) = v {
        assert_eq!(data, b"moo");
    }

    let v = dict.get(b"spam").unwrap();
    if let Value::Bytes(data) = v {
        assert_eq!(data, b"eggs");
    }
}
```

License: MIT OR Apache-2.0
