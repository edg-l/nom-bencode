# nom_bencode

[![Version](https://img.shields.io/crates/v/nom_bencode)](https://crates.io/crates/nom_bencode)
[![Downloads](https://img.shields.io/crates/d/nom_bencode)](https://crates.io/crates/nom_bencode)
[![License](https://img.shields.io/crates/l/nom_bencode)](https://crates.io/crates/nom_bencode)
![Rust](https://github.com/edg-l/nom-bencode/workflows/Rust/badge.svg)
[![Docs](https://docs.rs/nom_bencode/badge.svg)](https://docs.rs/nom_bencode)

A bencode parser written with nom.
```rust
let data = parse(b"d3:cow3:moo4:spam4:eggse").unwrap();
let v = data.first().unwrap();
assert_matches!(v, Value::Dictionary(_));

if let Value::Dictionary(dict) = v {
    let v = dict.get("cow".as_bytes()).unwrap();
    assert_matches!(*v, Value::Bytes(b"moo"));

    let v = dict.get("spam".as_bytes()).unwrap();
    assert_matches!(*v, Value::Bytes(b"eggs"));
}

let (_, v) = Value::parse_dict(b"d4:spaml1:a1:bee").unwrap();
assert_matches!(v, Value::Dictionary(_));

if let Value::Dictionary(dict) = v {
    let v = dict.get("spam".as_bytes()).unwrap();
    assert_matches!(*v, Value::List(_));
}
```

License: MIT OR Apache-2.0
