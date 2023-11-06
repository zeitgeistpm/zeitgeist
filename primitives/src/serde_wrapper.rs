// Copyright 2021-2022 Zeitgeist PM LLC.
//
// This file is part of Zeitgeist.
//
// Zeitgeist is free software: you can redistribute it and/or modify it
// under the terms of the GNU General Public License as published by the
// Free Software Foundation, either version 3 of the License, or (at
// your option) any later version.
//
// Zeitgeist is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Zeitgeist. If not, see <https://www.gnu.org/licenses/>.

use parity_scale_codec::{Decode, Encode, Error as ScaleError, Compact, CompactAs, HasCompact, MaxEncodedLen};
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
pub struct SerdeWrapper<B: MaxEncodedLen + HasCompact>(
    #[cfg_attr(feature = "std", serde(bound(serialize = "B: std::fmt::Display")))]
    #[cfg_attr(feature = "std", serde(serialize_with = "serialize_as_string"))]
    #[cfg_attr(feature = "std", serde(bound(deserialize = "B: std::str::FromStr")))]
    #[cfg_attr(feature = "std", serde(deserialize_with = "deserialize_from_string"))]
    pub B,
);

impl<B: MaxEncodedLen + HasCompact> CompactAs for SerdeWrapper<B> {
    type As = B;

    fn encode_as(&self) -> &Self::As {
        &self.0
    }

    fn decode_from(inner: Self::As) -> Result<Self, ScaleError> {
        Ok(SerdeWrapper(inner))
    }
}

impl<B: MaxEncodedLen + HasCompact> From<Compact<SerdeWrapper<B>>> for SerdeWrapper<B> {
    fn from(compact: Compact<SerdeWrapper<B>>) -> Self {
        compact.0
    }
}

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
