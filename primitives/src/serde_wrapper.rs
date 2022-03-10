use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
#[cfg(feature = "std")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Used to workaround serde serialization/deserialization problems involving `u128`.
///
/// # Types
///
/// * `B`: Balance
#[derive(
    scale_info::TypeInfo,
    Clone,
    Copy,
    Debug,
    Decode,
    Default,
    Encode,
    Eq,
    MaxEncodedLen,
    Ord,
    PartialEq,
    PartialOrd,
)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct SerdeWrapper<B: MaxEncodedLen>(
    #[cfg_attr(feature = "std", serde(bound(serialize = "B: std::fmt::Display")))]
    #[cfg_attr(feature = "std", serde(serialize_with = "serialize_as_string"))]
    #[cfg_attr(feature = "std", serde(bound(deserialize = "B: std::str::FromStr")))]
    #[cfg_attr(feature = "std", serde(deserialize_with = "deserialize_from_string"))]
    pub B,
);

#[cfg(feature = "std")]
fn serialize_as_string<S: Serializer, T: std::fmt::Display>(
    t: &T,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&t.to_string())
}

#[cfg(feature = "std")]
fn deserialize_from_string<'de, D: Deserializer<'de>, T: std::str::FromStr>(
    deserializer: D,
) -> Result<T, D::Error> {
    let s = String::deserialize(deserializer)?;
    s.parse::<T>().map_err(|_| serde::de::Error::custom("Parse from string failed"))
}
