// Copyright 2025 Forecasting Technologies LTD.

use arbitrary::{Arbitrary, Result as ArbitraryResult, Unstructured};
use frame_system::pallet_prelude::{BlockNumberFor, OriginFor};
use libfuzzer_sys::fuzz_target;
use zrml_futarchy::{
    mock::{
        ext_builder::ExtBuilder,
        runtime::{Futarchy, Runtime, RuntimeOrigin},
    },
    types::Proposal,
};

#[derive(Debug)]
struct SubmitProposalParams {
    origin: OriginFor<Runtime>,
    duration: BlockNumberFor<Runtime>,
    proposal: Proposal<Runtime>,
}

impl<'a> Arbitrary<'a> for SubmitProposalParams {
    fn arbitrary(u: &mut Unstructured<'a>) -> ArbitraryResult<Self> {
        let account_id = u128::arbitrary(u)?;
        let origin = RuntimeOrigin::signed(account_id);

        let duration = Arbitrary::arbitrary(u)?;

        let proposal = Arbitrary::arbitrary(u)?;

        let params = SubmitProposalParams { origin, duration, proposal };

        Ok(params)
    }
}

fuzz_target!(|params: SubmitProposalParams| {
    let mut ext = ExtBuilder::build();

    ext.execute_with(|| {
        let _ = Futarchy::submit_proposal(params.origin, params.duration, params.proposal);
    });

    let _ = ext.commit_all();
});
