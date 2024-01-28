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
use frame_support::dispatch::Weight;
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;

#[derive(sp_runtime::RuntimeDebug, Clone, Decode, Encode, Eq, PartialEq, TypeInfo)]
pub struct ResultWithWeightInfo<R> {
    pub result: R,
    pub weight: Weight,
}
