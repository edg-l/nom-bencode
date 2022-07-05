//! [![Version](https://img.shields.io/crates/v/nom_bencode)](https://crates.io/crates/nom_bencode)
//! [![Downloads](https://img.shields.io/crates/d/nom_bencode)](https://crates.io/crates/nom_bencode)
//! [![License](https://img.shields.io/crates/l/nom_bencode)](https://crates.io/crates/nom_bencode)
//! ![Rust](https://github.com/edg-l/nom_bencode/workflows/Rust/badge.svg)
//! [![Docs](https://docs.rs/nom_bencode/badge.svg)](https://docs.rs/nom_bencode)
//!
//! A bencode parser written with nom.
//! ```rust
//! let data = parse(b"d3:cow3:moo4:spam4:eggse").unwrap();
//! let v = data.first().unwrap();
//! assert_matches!(v, Value::Dictionary(_));
//!
//! if let Value::Dictionary(dict) = v {
//!     let v = dict.get("cow".as_bytes()).unwrap();
//!     assert_matches!(*v, Value::Bytes(b"moo"));
//!
//!     let v = dict.get("spam".as_bytes()).unwrap();
//!     assert_matches!(*v, Value::Bytes(b"eggs"));
//! }
//!
//! let (_, v) = Value::parse_dict(b"d4:spaml1:a1:bee").unwrap();
//! assert_matches!(v, Value::Dictionary(_));
//!
//! if let Value::Dictionary(dict) = v {
//!     let v = dict.get("spam".as_bytes()).unwrap();
//!     assert_matches!(*v, Value::List(_));
//! }
//! ```

use nom::{
    branch::alt,
    bytes::complete::take,
    character::complete::{char, digit1},
    combinator::{eof, recognize},
    error::ParseError,
    multi::many_till,
    sequence::{delimited, pair, preceded},
    IResult,
};
use std::{collections::HashMap, fmt::Debug, num::ParseIntError, str::Utf8Error};

type BenResult<'a> = IResult<&'a [u8], Value<'a>, Error<&'a [u8]>>;

#[derive(Debug, thiserror::Error)]
pub enum Error<I> {
    #[error("invalid integer: {0:?}")]
    InvalidInteger(I),
    #[error("invalid bytes length: {0:?}")]
    InvalidBytesLength(I),
    #[error("parse int error: {0:?}")]
    ParseIntError(#[from] ParseIntError),
    #[error("utf8 error: {0:?}")]
    Utf8Error(#[from] Utf8Error),
    #[error("nom parsing error: {0:?}")]
    NomError(#[from] nom::error::Error<I>),
}

impl<I> From<Error<I>> for nom::Err<Error<I>> {
    fn from(e: Error<I>) -> Self {
        nom::Err::Error(e)
    }
}

impl<I> From<nom::Err<Error<I>>> for Error<I> {
    fn from(e: nom::Err<Error<I>>) -> Self {
        e.into()
    }
}

impl<I> ParseError<I> for Error<I> {
    fn from_error_kind(input: I, kind: nom::error::ErrorKind) -> Self {
        Self::NomError(nom::error::Error { input, code: kind })
    }

    fn append(_: I, _: nom::error::ErrorKind, other: Self) -> Self {
        other
    }
}

#[derive(Debug, Clone)]
pub enum Value<'a> {
    Bytes(&'a [u8]),
    Integer(i64),
    List(Vec<Self>),
    Dictionary(HashMap<&'a [u8], Self>),
}

impl<'a> Value<'a> {
    fn parse_integer(start_inp: &'a [u8]) -> BenResult<'a> {
        let (inp, value) = delimited(
            char('i'),
            alt((
                recognize(pair(char('+'), digit1)),
                recognize(pair(char('-'), digit1)),
                digit1,
            )),
            char('e'),
        )(start_inp)?;

        // SAFETY: Provided the combinators work correctly, this will always be a valid UTF-8 sequence.
        let value_str = unsafe { std::str::from_utf8_unchecked(value) };

        if value_str.starts_with("-0") || (value_str.starts_with("0") && value_str.len() > 1) {
            Err(Error::InvalidInteger(start_inp))?
        } else {
            let value_integer = i64::from_str_radix(value_str, 10).map_err(Error::ParseIntError)?;
            Ok((inp, Value::Integer(value_integer)))
        }
    }

    fn parse_bytes(start_inp: &'a [u8]) -> BenResult<'a> {
        let (inp, length) = digit1(start_inp)?;

        let (inp, _) = char(':')(inp)?;

        // SAFETY: Provided the combinators work correctly, this will always be a valid UTF-8 sequence.
        let length = unsafe { std::str::from_utf8_unchecked(length) };

        let length = u64::from_str_radix(length, 10).map_err(Error::ParseIntError)?;

        if length == 0 {
            Err(Error::InvalidBytesLength(start_inp))?
        }

        let (inp, characters) = take(length)(inp)?;

        Ok((inp, Value::Bytes(characters)))
    }

    fn parse_list(start_inp: &'a [u8]) -> BenResult<'a> {
        let (inp, value) = preceded(
            char('l'),
            many_till(
                alt((
                    Self::parse_bytes,
                    Self::parse_integer,
                    Self::parse_list,
                    Self::parse_dict,
                )),
                char('e'),
            ),
        )(start_inp)?;

        Ok((inp, Value::List(value.0)))
    }

    fn parse_dict(start_inp: &'a [u8]) -> BenResult<'a> {
        let (inp, value) = preceded(
            char('d'),
            many_till(
                pair(
                    Self::parse_bytes,
                    alt((
                        Self::parse_bytes,
                        Self::parse_integer,
                        Self::parse_list,
                        Self::parse_dict,
                    )),
                ),
                char('e'),
            ),
        )(start_inp)?;

        let data = value.0.into_iter().map(|x| {
            // Keys are always string
            if let Value::Bytes(key) = x.0 {
                (key, x.1)
            } else {
                unreachable!()
            }
        });

        let map = HashMap::from_iter(data);

        Ok((inp, Value::Dictionary(map)))
    }
}

pub fn parse<'a>(source: &'a [u8]) -> Result<Vec<Value<'a>>, Error<&'a [u8]>> {
    let (_, items) = many_till(
        alt((
            Value::parse_bytes,
            Value::parse_integer,
            Value::parse_list,
            Value::parse_dict,
        )),
        eof,
    )(source)?;

    Ok(items.0)
}

#[cfg(test)]
mod tests {
    use crate::{parse, Error, Value};
    use assert_matches::assert_matches;

    #[test]
    fn test_integer() {
        let (_, v) = Value::parse_integer(b"i3e").unwrap();
        assert_matches!(v, Value::Integer(3));

        let (_, v) = Value::parse_integer(b"i3e1:a").unwrap();
        assert_matches!(v, Value::Integer(3));

        let (_, v) = Value::parse_integer(b"i-3e").unwrap();
        assert_matches!(v, Value::Integer(-3));

        let (_, v) = Value::parse_integer(b"i333333e").unwrap();
        assert_matches!(v, Value::Integer(333333));

        let v = Value::parse_integer(b"i-0e").unwrap_err();
        assert_matches!(v, nom::Err::Error(Error::InvalidInteger(_)));

        let v = Value::parse_integer(b"i00e").unwrap_err();
        assert_matches!(v, nom::Err::Error(Error::InvalidInteger(_)));

        let v = Value::parse_integer(b"i-00e").unwrap_err();
        assert_matches!(v, nom::Err::Error(Error::InvalidInteger(_)));

        let v = Value::parse_integer(b"i03e").unwrap_err();
        assert_matches!(v, nom::Err::Error(Error::InvalidInteger(_)));

        let v = Value::parse_integer(b"i0040e").unwrap_err();
        assert_matches!(v, nom::Err::Error(Error::InvalidInteger(_)));

        let v = Value::parse_integer(b"li3ee").unwrap_err();
        assert_matches!(v, nom::Err::Error(Error::NomError(_)));
    }

    #[test]
    fn test_string() {
        let (_, v) = Value::parse_bytes(b"4:abcd").unwrap();
        assert_matches!(v, Value::Bytes(b"abcd"));

        let (_, v) = Value::parse_bytes(b"1:a").unwrap();
        assert_matches!(v, Value::Bytes(b"a"));

        let (_, v) = Value::parse_bytes(b"1:rock").unwrap();
        assert_matches!(v, Value::Bytes(b"r"));

        let v = Value::parse_bytes(b"0:a").unwrap_err();
        assert_matches!(v, nom::Err::Error(Error::InvalidBytesLength(_)));
    }

    #[test]
    fn test_list() {
        let (_, v) = Value::parse_list(b"l4:spam4:eggsi22eli1ei2eee").unwrap();
        assert_matches!(v, Value::List(_));

        if let Value::List(list) = v {
            let mut it = list.iter();

            let x = it.next().unwrap();
            assert_matches!(*x, Value::Bytes(b"spam"));

            let x = it.next().unwrap();
            assert_matches!(*x, Value::Bytes(b"eggs"));

            let x = it.next().unwrap();
            assert_matches!(*x, Value::Integer(22));

            let x = it.next().unwrap();
            assert_matches!(*x, Value::List(_));

            if let Value::List(list) = x {
                let mut it = list.iter();

                let x = it.next().unwrap();
                assert_matches!(*x, Value::Integer(1));

                let x = it.next().unwrap();
                assert_matches!(*x, Value::Integer(2));
            }
        }
    }

    #[test]
    fn test_list_empty() {
        let (_, v) = Value::parse_list(b"le").unwrap();
        assert_matches!(v, Value::List(_));
    }

    #[test]
    fn test_dict() {
        let (_, v) = Value::parse_dict(b"d3:cow3:moo4:spam4:eggse").unwrap();
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
    }

    #[test]
    fn test_parse() {
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
    }

    #[test]
    fn test_parse_torrent() {
        let data = parse(include_bytes!("../test-assets/big-buck-bunny.torrent")).unwrap();
        assert_eq!(data.len(), 1);

        let v = data.first().unwrap();
        assert_matches!(*v, Value::Dictionary(_));

        if let Value::Dictionary(dict) = v {
            let info = dict.get("info".as_bytes()).unwrap();
            assert_matches!(*info, Value::Dictionary(_));

            let announce = dict.get("announce".as_bytes()).unwrap();
            assert_matches!(*announce, Value::Bytes(_));

            if let Value::Bytes(announce) = *announce {
                let announce = std::str::from_utf8(announce).unwrap();
                assert_eq!(announce, "udp://tracker.leechers-paradise.org:6969");
            }

            let announce_list = dict.get("announce-list".as_bytes()).unwrap();
            assert_matches!(*announce_list, Value::List(_));
        }
    }
}
