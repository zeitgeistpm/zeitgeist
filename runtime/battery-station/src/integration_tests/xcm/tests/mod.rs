// Copyright 2022-2025 Forecasting Technologies LTD.
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

use xcm_emulator::{Network, TestExt};

mod currency_id_convert;
mod transfers;

use once_cell::sync::Lazy;
use std::sync::{Mutex, MutexGuard};

static TEST_NET_MUTEX: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

pub(crate) fn reset_test_net() -> MutexGuard<'static, ()> {
    let guard = TEST_NET_MUTEX.lock().expect("Test net mutex poisoned");
    crate::integration_tests::xcm::test_net::TestNet::reset();
    guard
}

#[cfg(feature = "parachain")]
fn clear_async_backing_slot_info() {
    use pallet_async_backing::SlotInfo;
    SlotInfo::<crate::Runtime>::kill();
}

pub(crate) fn with_battery_station<R>(f: impl FnOnce() -> R) -> R {
    crate::integration_tests::xcm::test_net::BatteryStationPara::execute_with(|| {
        let res = f();
        #[cfg(feature = "parachain")]
        clear_async_backing_slot_info();
        res
    })
}

pub(crate) fn with_sibling<R>(f: impl FnOnce() -> R) -> R {
    crate::integration_tests::xcm::test_net::SiblingPara::execute_with(|| {
        let res = f();
        #[cfg(feature = "parachain")]
        clear_async_backing_slot_info();
        res
    })
}
