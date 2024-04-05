// Copyright 2023-2024 Forecasting Technologies LTD.
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

#![cfg(test)]

use super::{mock::*, Error};
use alloc::collections::BTreeMap;
use frame_support::{
    assert_noop, assert_ok,
    dispatch::RawOrigin::Signed,
    traits::{
        tokens::{
            fungibles::{Create, Destroy},
            DepositConsequence, WithdrawConsequence,
        },
        OnIdle, UnfilteredDispatchable,
    },
};
use orml_traits::{
    BalanceStatus, MultiCurrencyExtended, MultiLockableCurrency, MultiReservableCurrency,
    NamedMultiReservableCurrency,
};
use zeitgeist_primitives::types::Assets;

mod create;
mod custom_types;
mod destroy;
mod inspect;
mod managed_destroy;
mod multi_currency;
mod multi_lockable_currency;
mod multi_reservable_currency;
mod named_multi_reservable_currency;
mod unbalanced;
