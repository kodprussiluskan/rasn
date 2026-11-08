//! OER: an extension addition whose value has extension additions of its own. The nested encode used to reuse the outer
//! encoder's buffers, so it counted and copied the bytes already encoded around it.

use rasn::prelude::*;

#[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq)]
#[rasn(automatic_tags)]
#[non_exhaustive]
pub struct Inner {
    pub a: u8,
    #[rasn(extension_addition)]
    pub b: Option<Utf8String>,
}

#[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq)]
#[rasn(automatic_tags)]
#[non_exhaustive]
pub struct Outer {
    pub name: Utf8String,
    #[rasn(extension_addition)]
    pub inner: Option<SequenceOf<Inner>>,
}

fn outer(b: Option<&str>) -> Outer {
    let inner = Inner { a: 2, b: b.map(Into::into) };
    Outer { name: "outer".into(), inner: Some(vec![inner.clone(), inner]) }
}

#[test]
fn nested_extension_additions_round_trip() {
    for codec in [rasn::Codec::Oer, rasn::Codec::Coer] {
        for value in [outer(None), outer(Some("nested"))] {
            let bytes = codec.encode_to_binary(&value).unwrap();
            assert_eq!(codec.decode_from_binary::<Outer>(&bytes).unwrap(), value, "{codec:?}");
        }
    }
}

#[derive(AsnType, Debug, Clone, Decode, Encode, PartialEq)]
#[rasn(automatic_tags)]
#[non_exhaustive]
pub struct Single {
    pub name: Utf8String,
    #[rasn(extension_addition)]
    pub inner: Option<Inner>,
}

#[test]
fn a_nested_extension_addition_is_encoded_exactly() {
    let value = Single { name: "o".into(), inner: Some(Inner { a: 2, b: Some("n".into()) }) };
    let inner = [0x80, 0x02, 0x02, 0x07, 0x80, 0x02, 0x01, 0x6e];
    assert_eq!(rasn::oer::encode(&value.inner.clone().unwrap()).unwrap(), inner);
    // Preamble (extension bit), "o", the extension bitmap (length 2, 7 unused bits, first present), then `inner` as an
    // open type: its length, then exactly its bytes.
    let mut expected = vec![0x80, 0x01, 0x6f, 0x02, 0x07, 0x80, 0x08];
    expected.extend(inner);
    assert_eq!(rasn::oer::encode(&value).unwrap(), expected);
}
