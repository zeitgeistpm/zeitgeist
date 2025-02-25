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

#![no_main]

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
