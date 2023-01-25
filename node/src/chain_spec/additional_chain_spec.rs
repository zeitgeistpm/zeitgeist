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

#[cfg(feature = "parachain")]
use {
    cumulus_primitives_core::ParaId,
    nimbus_primitives::NimbusId,
    pallet_parachain_staking::InflationInfo,
    sp_runtime::{Perbill, Percent},
    zeitgeist_primitives::types::{AccountId, Balance},
};

#[cfg(feature = "parachain")]
pub struct AdditionalChainSpec {
    pub blocks_per_round: u32,
    pub candidates: Vec<(AccountId, NimbusId, Balance)>,
    pub collator_commission: Perbill,
    pub inflation_info: InflationInfo<Balance>,
    pub nominations: Vec<(AccountId, AccountId, Balance, Percent)>,
    pub parachain_bond_reserve_percent: Percent,
    pub parachain_id: ParaId,
}

#[cfg(not(feature = "parachain"))]
pub struct AdditionalChainSpec {
    pub initial_authorities:
        Vec<(sp_consensus_aura::sr25519::AuthorityId, sp_finality_grandpa::AuthorityId)>,
}
