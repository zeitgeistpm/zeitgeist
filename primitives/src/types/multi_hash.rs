// Copyright 2022-2024 Forecasting Technologies LTD.
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

pub use crate::{
    asset::*, market::*, max_runtime_usize::*, outcome_report::OutcomeReport, proxy_type::*,
    serde_wrapper::*,
};
#[cfg(feature = "arbitrary")]
use arbitrary::{Arbitrary, Result, Unstructured};
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;

/// Multihash for digest sizes up to 384 bit.
/// The multicodec encoding the hash algorithm uses only 1 byte,
/// effecitvely limiting the number of available hash types.
/// HashType (1B) + DigestSize (1B) + Hash (48B).
#[derive(TypeInfo, Clone, Debug, Decode, Encode, Eq, PartialEq)]
pub enum MultiHash {
    Sha3_384([u8; 50]),
}

// Implementation for the fuzzer
#[cfg(feature = "arbitrary")]
impl<'a> Arbitrary<'a> for MultiHash {
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
        let mut rand_bytes = <[u8; 50] as Arbitrary<'a>>::arbitrary(u)?;
        rand_bytes[0] = 0x15;
        rand_bytes[1] = 0x30;
        Ok(MultiHash::Sha3_384(rand_bytes))
    }

    fn size_hint(_depth: usize) -> (usize, Option<usize>) {
        (50, Some(50))
    }
}
