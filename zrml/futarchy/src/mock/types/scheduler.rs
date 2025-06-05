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

use crate::{mock::runtime::Runtime, BoundedCallOf, CallOf, PalletsOriginOf};
use core::cell::RefCell;
use frame_support::traits::schedule::{v3::Anon as ScheduleAnon, DispatchTime, Period, Priority};
use frame_system::pallet_prelude::BlockNumberFor;
use sp_runtime::{traits::BlakeTwo256, DispatchError, DispatchResult};

pub struct MockScheduler;

impl MockScheduler {
    pub fn set_return_value(value: DispatchResult) {
        SCHEDULER_RETURN_VALUE.with(|v| *v.borrow_mut() = value);
    }

    pub fn not_called() -> bool {
        SCHEDULER_CALL_DATA.with(|values| values.borrow().is_empty())
    }

    pub fn called_once_with(
        when: DispatchTime<BlockNumberFor<Runtime>>,
        call: BoundedCallOf<Runtime>,
    ) -> bool {
        if SCHEDULER_CALL_DATA.with(|values| values.borrow().len()) != 1 {
            return false;
        }

        let args = SCHEDULER_CALL_DATA
            .with(|value| value.borrow().first().expect("can't be empty").clone());

        args == SchedulerCallData { when, call }
    }
}

#[derive(Clone, PartialEq)]
struct SchedulerCallData {
    when: DispatchTime<BlockNumberFor<Runtime>>,
    call: BoundedCallOf<Runtime>,
}

impl ScheduleAnon<BlockNumberFor<Runtime>, CallOf<Runtime>, PalletsOriginOf<Runtime>>
    for MockScheduler
{
    type Address = ();
    type Hasher = BlakeTwo256;

    fn schedule(
        when: DispatchTime<BlockNumberFor<Runtime>>,
        _maybe_periodic: Option<Period<BlockNumberFor<Runtime>>>,
        _priority: Priority,
        _origin: PalletsOriginOf<Runtime>,
        call: BoundedCallOf<Runtime>,
    ) -> Result<Self::Address, DispatchError> {
        SCHEDULER_CALL_DATA
            .with(|values| values.borrow_mut().push(SchedulerCallData { when, call }));

        SCHEDULER_RETURN_VALUE.with(|value| *value.borrow())
    }

    fn cancel(_address: Self::Address) -> Result<(), DispatchError> {
        unimplemented!();
    }

    fn reschedule(
        _address: Self::Address,
        _when: DispatchTime<BlockNumberFor<Runtime>>,
    ) -> Result<Self::Address, DispatchError> {
        unimplemented!();
    }

    fn next_dispatch_time(
        _address: Self::Address,
    ) -> Result<BlockNumberFor<Runtime>, DispatchError> {
        unimplemented!();
    }
}

thread_local! {
    pub static SCHEDULER_CALL_DATA: RefCell<Vec<SchedulerCallData>> =
        const { RefCell::new(vec![]) };
    pub static SCHEDULER_RETURN_VALUE: RefCell<DispatchResult> = const { RefCell::new(Ok(())) };
}
