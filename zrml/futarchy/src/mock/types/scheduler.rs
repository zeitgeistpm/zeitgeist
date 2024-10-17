use crate::{mock::runtime::Runtime, BoundedCallOf, CallOf};
use core::cell::RefCell;
use frame_support::traits::schedule::{v3::Anon as ScheduleAnon, DispatchTime, Period, Priority};
use frame_system::{
    pallet_prelude::{BlockNumberFor, OriginFor},
    Call, Origin,
};
use sp_runtime::{traits::Bounded, DispatchError, DispatchResult};

pub struct MockScheduler;

impl MockScheduler {
    fn set_return_value(value: DispatchResult) {
        SCHEDULER_RETURN_VALUE.with(|v| *v.borrow_mut() = Some(value));
    }

    fn called_once_with(
        when: DispatchTime<BlockNumberFor<Runtime>>,
        call: BoundedCallOf<Runtime>,
    ) -> bool {
        if SCHEDULER_CALL_DATA.with(|value| value.borrow().len()) != 1 {
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

impl ScheduleAnon<BlockNumberFor<Runtime>, CallOf<Runtime>, OriginFor<Runtime>> for MockScheduler {
    type Address = ();

    fn schedule(
        when: DispatchTime<BlockNumberFor<Runtime>>,
        _maybe_periodic: Option<Period<BlockNumberFor<Runtime>>>,
        _priority: Priority,
        _origin: OriginFor<Runtime>,
        call: BoundedCallOf<Runtime>,
    ) -> Result<Self::Address, DispatchError> {
        SCHEDULER_CALL_DATA
            .with(|values| values.borrow_mut().push(SchedulerCallData { when, call }));

        SCHEDULER_RETURN_VALUE
            .with(|value| *value.borrow())
            .expect("no return value configured for scheduler mock")
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
    pub static SCHEDULER_RETURN_VALUE: RefCell<Option<DispatchResult>> =
        const { RefCell::new(None) };
}
