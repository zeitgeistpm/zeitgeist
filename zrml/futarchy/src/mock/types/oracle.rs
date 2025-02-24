// Copyright 2024-2025 Forecasting Technologies LTD.
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

use alloc::fmt::Debug;
use frame_support::pallet_prelude::Weight;
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::traits::Zero;
use zeitgeist_primitives::{traits::FutarchyOracle, types::BlockNumber};

#[cfg(feature = "fuzzing")]
use arbitrary::{Arbitrary, Result as ArbitraryResult, Unstructured};

#[derive(Clone, Debug, Decode, Encode, Eq, MaxEncodedLen, PartialEq, TypeInfo)]
pub struct MockOracle {
    weight: Weight,
    value: bool,
}

impl Default for MockOracle {
    fn default() -> Self {
        MockOracle { weight: Default::default(), value: true }
    }
}

impl MockOracle {
    pub fn new(weight: Weight, value: bool) -> Self {
        Self { weight, value }
    }
}

impl FutarchyOracle for MockOracle {
    type BlockNumber = BlockNumber;

    fn evaluate(&self) -> (Weight, bool) {
        (self.weight, self.value)
    }

    fn update(&mut self, _: Self::BlockNumber) -> Weight {
        Zero::zero()
    }
}

#[cfg(feature = "fuzzing")]
impl<'a> Arbitrary<'a> for MockOracle {
    fn arbitrary(u: &mut Unstructured<'a>) -> ArbitraryResult<Self> {
        let ref_time = u64::arbitrary(u)?;
        let proof_size = u64::arbitrary(u)?;
        let weight = Weight::from_parts(ref_time, proof_size);

        let value = bool::arbitrary(u)?;

        Ok(MockOracle::new(weight, value))
    }
}
