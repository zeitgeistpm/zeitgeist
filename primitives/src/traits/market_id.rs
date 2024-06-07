// Copyright 2024 Forecasting Technologies LTD.
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

use frame_support::{
    pallet_prelude::{MaybeSerializeDeserialize, Member},
    Parameter,
};
use parity_scale_codec::MaxEncodedLen;
use sp_runtime::traits::AtLeast32Bit;

pub trait MarketId:
    AtLeast32Bit + Copy + Default + MaxEncodedLen + MaybeSerializeDeserialize + Member + Parameter
{
}

impl<T> MarketId for T where
    T: AtLeast32Bit
        + Copy
        + Default
        + MaxEncodedLen
        + MaybeSerializeDeserialize
        + Member
        + Parameter
{
}
