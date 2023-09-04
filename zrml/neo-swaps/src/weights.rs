// Copyright 2023 Forecasting Technologies LTD.
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

use core::marker::PhantomData;
use frame_support::weights::Weight;

pub trait WeightInfoZeitgeist {
    fn buy(a: u32) -> Weight;
    fn sell(a: u32) -> Weight;
    fn join(a: u32) -> Weight;
    fn exit(a: u32) -> Weight;
    fn split() -> Weight;
    fn withdraw_fees() -> Weight;
    fn deploy_pool(a: u32) -> Weight;
}

pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfoZeitgeist for WeightInfo<T> {
    fn buy(_a: u32) -> Weight {
        Weight::from_ref_time(1u64)
    }
    fn sell(_a: u32) -> Weight {
        Weight::from_ref_time(1u64)
    }
    fn join(_a: u32) -> Weight {
        Weight::from_ref_time(1u64)
    }
    fn exit(_a: u32) -> Weight {
        Weight::from_ref_time(1u64)
    }
    fn split() -> Weight {
        Weight::from_ref_time(1u64)
    }
    fn withdraw_fees() -> Weight {
        Weight::from_ref_time(1u64)
    }
    fn deploy_pool(_a: u32) -> Weight {
        Weight::from_ref_time(1u64)
    }
}
