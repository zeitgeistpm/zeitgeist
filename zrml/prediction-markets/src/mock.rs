// Copyright 2022-2025 Forecasting Technologies LTD.
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

#![allow(
    // Mocks are only used for fuzzing and unit tests
    clippy::arithmetic_side_effects
)]
#![cfg(feature = "mock")]

use crate as prediction_markets;
use frame_support::{
    construct_runtime, ord_parameter_types, parameter_types,
    traits::{
        tokens::{PayFromAccount, UnityAssetBalanceConversion},
        Everything, NeverEnsureOrigin, OnFinalize, OnInitialize,
    },
};
use frame_system::{mocking::MockBlock, EnsureRoot, EnsureSignedBy};
#[cfg(feature = "runtime-benchmarks")]
use pallet_treasury::ArgumentsFactory;
use sp_arithmetic::per_things::Percent;
#[cfg(feature = "runtime-benchmarks")]
use sp_core::{H160, H256};
use sp_runtime::{
    traits::{BlakeTwo256, ConstU32, IdentityLookup},
    BuildStorage, DispatchError, DispatchResult,
};
use std::cell::RefCell;
use zeitgeist_primitives::{
    constants::mock::{
        AddOutcomePeriod, AggregationPeriod, AppealBond, AppealPeriod, AuthorizedPalletId,
        BlockHashCount, BlocksPerYear, CloseEarlyBlockPeriod, CloseEarlyDisputeBond,
        CloseEarlyProtectionBlockPeriod, CloseEarlyProtectionTimeFramePeriod,
        CloseEarlyRequestBond, CloseEarlyTimeFramePeriod, CorrectionPeriod, CourtPalletId,
        ExistentialDeposit, ExistentialDeposits, GdVotingPeriod, GetNativeCurrencyId,
        GlobalDisputeLockId, GlobalDisputesPalletId, InflationPeriod, LockId, MaxAppeals,
        MaxApprovals, MaxCategories, MaxCourtParticipants, MaxCreatorFee, MaxDelegations,
        MaxDisputeDuration, MaxDisputes, MaxEditReasonLen, MaxGlobalDisputeVotes, MaxGracePeriod,
        MaxLocks, MaxMarketLifetime, MaxOracleDuration, MaxOwners, MaxRejectReasonLen, MaxReserves,
        MaxSelectedDraws, MaxYearlyInflation, MinCategories, MinDisputeDuration, MinJurorStake,
        MinOracleDuration, MinOutcomeVoteAmount, MinimumPeriod, OutsiderBond, PmPalletId,
        RemoveKeysLimit, RequestInterval, TreasuryPalletId, VotePeriod, VotingOutcomeFee, BASE,
        CENT, MILLISECS_PER_BLOCK,
    },
    traits::DeployPoolApi,
    types::{
        AccountIdTest, Amount, Asset, Balance, BasicCurrencyAdapter, BlockNumber, BlockTest,
        CurrencyId, Hash, MarketId, Moment,
    },
};
#[cfg(feature = "parachain")]
use {
    orml_traits::asset_registry::AssetProcessor, parity_scale_codec::Encode,
    zeitgeist_primitives::types::CustomMetadata,
};

pub const ALICE: AccountIdTest = 0;
pub const BOB: AccountIdTest = 1;
pub const CHARLIE: AccountIdTest = 2;
pub const DAVE: AccountIdTest = 3;
pub const EVE: AccountIdTest = 4;
pub const FRED: AccountIdTest = 5;
pub const SUDO: AccountIdTest = 69;
pub const APPROVE_ORIGIN: AccountIdTest = 70;
pub const REJECT_ORIGIN: AccountIdTest = 71;
pub const CLOSE_MARKET_EARLY_ORIGIN: AccountIdTest = 72;
pub const CLOSE_ORIGIN: AccountIdTest = 73;
pub const REQUEST_EDIT_ORIGIN: AccountIdTest = 74;
pub const RESOLVE_ORIGIN: AccountIdTest = 75;

pub const INITIAL_BALANCE: u128 = 1_000 * BASE;

#[allow(unused)]
pub struct DeployPoolMock;

#[allow(unused)]
#[derive(Clone)]
pub struct DeployPoolArgs {
    who: AccountIdTest,
    market_id: MarketId,
    amount: Balance,
    swap_prices: Vec<Balance>,
    swap_fee: Balance,
}

thread_local! {
    pub static DEPLOY_POOL_CALL_DATA: RefCell<Vec<DeployPoolArgs>> = const { RefCell::new(vec![]) };
    pub static DEPLOY_POOL_RETURN_VALUE: RefCell<DispatchResult> = const { RefCell::new(Ok(())) };
}

#[allow(unused)]
impl DeployPoolApi for DeployPoolMock {
    type AccountId = AccountIdTest;
    type Balance = Balance;
    type MarketId = MarketId;

    fn deploy_pool(
        who: Self::AccountId,
        market_id: Self::MarketId,
        amount: Self::Balance,
        swap_prices: Vec<Self::Balance>,
        swap_fee: Self::Balance,
    ) -> DispatchResult {
        DEPLOY_POOL_CALL_DATA.with(|value| {
            value.borrow_mut().push(DeployPoolArgs {
                who,
                market_id,
                amount,
                swap_prices,
                swap_fee,
            })
        });
        DEPLOY_POOL_RETURN_VALUE.with(|v| *v.borrow())
    }
}

#[allow(unused)]
impl DeployPoolMock {
    pub fn called_once_with(
        who: AccountIdTest,
        market_id: MarketId,
        amount: Balance,
        swap_prices: Vec<Balance>,
        swap_fee: Balance,
    ) -> bool {
        if DEPLOY_POOL_CALL_DATA.with(|value| value.borrow().len()) != 1 {
            return false;
        }
        let args = DEPLOY_POOL_CALL_DATA.with(|value| value.borrow()[0].clone());
        args.who == who
            && args.market_id == market_id
            && args.amount == amount
            && args.swap_prices == swap_prices
            && args.swap_fee == swap_fee
    }

    pub fn return_error() {
        DEPLOY_POOL_RETURN_VALUE
            .with(|value| *value.borrow_mut() = Err(DispatchError::Other("neo-swaps")));
    }
}

ord_parameter_types! {
    pub const Sudo: AccountIdTest = SUDO;
    pub const ApproveOrigin: AccountIdTest = APPROVE_ORIGIN;
    pub const RejectOrigin: AccountIdTest = REJECT_ORIGIN;
    pub const CloseMarketEarlyOrigin: AccountIdTest = CLOSE_MARKET_EARLY_ORIGIN;
    pub const CloseOrigin: AccountIdTest = CLOSE_ORIGIN;
    pub const RequestEditOrigin: AccountIdTest = REQUEST_EDIT_ORIGIN;
    pub const ResolveOrigin: AccountIdTest = RESOLVE_ORIGIN;
}
parameter_types! {
    pub const AdvisoryBond: Balance = 11 * CENT;
    pub const AdvisoryBondSlashPercentage: Percent = Percent::from_percent(10);
    pub const OracleBond: Balance = 25 * CENT;
    pub const ValidityBond: Balance = 53 * CENT;
    pub const DisputeBond: Balance = 109 * CENT;
    pub TreasuryAccount: AccountIdTest = Treasury::account_id();
}

construct_runtime!(
    pub enum Runtime {
        #[cfg(feature = "parachain")]
        AssetRegistry: orml_asset_registry::module,
        Authorized: zrml_authorized,
        Balances: pallet_balances,
        Court: zrml_court,
        AssetManager: orml_currencies,
        MarketCommons: zrml_market_commons,
        PredictionMarkets: prediction_markets,
        RandomnessCollectiveFlip: pallet_insecure_randomness_collective_flip,
        GlobalDisputes: zrml_global_disputes,
        System: frame_system,
        Timestamp: pallet_timestamp,
        Tokens: orml_tokens,
        Treasury: pallet_treasury,
    }
);

impl crate::Config for Runtime {
    type AdvisoryBond = AdvisoryBond;
    type AdvisoryBondSlashPercentage = AdvisoryBondSlashPercentage;
    type ApproveOrigin = EnsureSignedBy<ApproveOrigin, AccountIdTest>;
    #[cfg(feature = "parachain")]
    type AssetRegistry = AssetRegistry;
    type Authorized = Authorized;
    type CloseEarlyDisputeBond = CloseEarlyDisputeBond;
    type CloseMarketEarlyOrigin = EnsureSignedBy<CloseMarketEarlyOrigin, AccountIdTest>;
    type CloseEarlyProtectionTimeFramePeriod = CloseEarlyProtectionTimeFramePeriod;
    type CloseEarlyProtectionBlockPeriod = CloseEarlyProtectionBlockPeriod;
    type CloseEarlyRequestBond = CloseEarlyRequestBond;
    type CloseOrigin = EnsureSignedBy<CloseOrigin, AccountIdTest>;
    type Currency = Balances;
    type MaxCreatorFee = MaxCreatorFee;
    type Court = Court;
    type DeployPool = DeployPoolMock;
    type DisputeBond = DisputeBond;
    type RuntimeEvent = RuntimeEvent;
    type GlobalDisputes = GlobalDisputes;
    type MaxCategories = MaxCategories;
    type MaxDisputes = MaxDisputes;
    type MinDisputeDuration = MinDisputeDuration;
    type MinOracleDuration = MinOracleDuration;
    type MaxDisputeDuration = MaxDisputeDuration;
    type MaxGracePeriod = MaxGracePeriod;
    type MaxOracleDuration = MaxOracleDuration;
    type MaxMarketLifetime = MaxMarketLifetime;
    type MinCategories = MinCategories;
    type MaxEditReasonLen = MaxEditReasonLen;
    type MaxRejectReasonLen = MaxRejectReasonLen;
    type OracleBond = OracleBond;
    type OutsiderBond = OutsiderBond;
    type PalletId = PmPalletId;
    type CloseEarlyBlockPeriod = CloseEarlyBlockPeriod;
    type CloseEarlyTimeFramePeriod = CloseEarlyTimeFramePeriod;
    type RejectOrigin = EnsureSignedBy<RejectOrigin, AccountIdTest>;
    type RequestEditOrigin = EnsureSignedBy<RequestEditOrigin, AccountIdTest>;
    type ResolveOrigin = EnsureSignedBy<ResolveOrigin, AccountIdTest>;
    type AssetManager = AssetManager;
    type Slash = Treasury;
    type ValidityBond = ValidityBond;
    type WeightInfo = prediction_markets::weights::WeightInfo<Runtime>;
}

impl frame_system::Config for Runtime {
    type AccountData = pallet_balances::AccountData<Balance>;
    type AccountId = AccountIdTest;
    type BaseCallFilter = Everything;
    type Block = MockBlock<Runtime>;
    type BlockHashCount = BlockHashCount;
    type BlockLength = ();
    type BlockWeights = ();
    type RuntimeCall = RuntimeCall;
    type RuntimeTask = RuntimeTask;
    type DbWeight = ();
    type RuntimeEvent = RuntimeEvent;
    type Hash = Hash;
    type Hashing = BlakeTwo256;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Nonce = u64;
    type MaxConsumers = ConstU32<16>;
    type MultiBlockMigrator = ();
    type OnKilledAccount = ();
    type OnNewAccount = ();
    type RuntimeOrigin = RuntimeOrigin;
    type PalletInfo = PalletInfo;
    type PreInherents = ();
    type PostInherents = ();
    type PostTransactions = ();
    type SingleBlockMigrations = ();
    type SS58Prefix = ();
    type SystemWeightInfo = ();
    type Version = ();
    type OnSetCode = ();
}

impl orml_currencies::Config for Runtime {
    type GetNativeCurrencyId = GetNativeCurrencyId;
    type MultiCurrency = Tokens;
    type NativeCurrency = BasicCurrencyAdapter<Runtime, Balances>;
    type WeightInfo = ();
}

cfg_if::cfg_if!(
    if #[cfg(feature = "parachain")] {
        type AssetMetadata = orml_traits::asset_registry::AssetMetadata<
            Balance,
            CustomMetadata,
            ConstU32<1024>
        >;
        pub struct NoopAssetProcessor {}

        impl AssetProcessor<CurrencyId, AssetMetadata> for NoopAssetProcessor {
            fn pre_register(id: Option<CurrencyId>, asset_metadata: AssetMetadata)
             -> Result<(CurrencyId, AssetMetadata), DispatchError> {
                Ok((id.unwrap(), asset_metadata))
            }
        }

        impl orml_asset_registry::module::Config for Runtime {
            type RuntimeEvent = RuntimeEvent;
            type CustomMetadata = CustomMetadata;
            type AssetId = CurrencyId;
            type AuthorityOrigin = EnsureRoot<AccountIdTest>;
            type AssetProcessor = NoopAssetProcessor;
            type Balance = Balance;
            type StringLimit = ConstU32<1024>;
            type WeightInfo = ();
        }
    }
);

impl orml_tokens::Config for Runtime {
    type Amount = Amount;
    type Balance = Balance;
    type CurrencyId = CurrencyId;
    type DustRemovalWhitelist = Everything;
    type RuntimeEvent = RuntimeEvent;
    type ExistentialDeposits = ExistentialDeposits;
    type MaxLocks = ();
    type MaxReserves = MaxReserves;
    type CurrencyHooks = ();
    type ReserveIdentifier = [u8; 8];
    type WeightInfo = ();
}

// #[cfg(feature = "parachain")]
// crate::orml_asset_registry::impl_mock_registry! {
//     MockRegistry,
//     CurrencyId,
//     Balance,
//     zeitgeist_primitives::types::CustomMetadata
// }

impl pallet_balances::Config for Runtime {
    type AccountStore = System;
    type Balance = Balance;
    type DustRemoval = ();
    type FreezeIdentifier = ();
    type RuntimeHoldReason = ();
    type RuntimeEvent = RuntimeEvent;
    type ExistentialDeposit = ExistentialDeposit;
    type MaxFreezes = ();
    type MaxLocks = MaxLocks;
    type MaxReserves = MaxReserves;
    type ReserveIdentifier = [u8; 8];
    type RuntimeFreezeReason = ();
    type WeightInfo = ();
}

impl pallet_insecure_randomness_collective_flip::Config for Runtime {}

impl pallet_timestamp::Config for Runtime {
    type MinimumPeriod = MinimumPeriod;
    type Moment = Moment;
    type OnTimestampSet = ();
    type WeightInfo = ();
}

ord_parameter_types! {
    pub const AuthorizedDisputeResolutionUser: AccountIdTest = ALICE;
}

impl zrml_authorized::Config for Runtime {
    type AuthorizedDisputeResolutionOrigin =
        EnsureSignedBy<AuthorizedDisputeResolutionUser, AccountIdTest>;
    type CorrectionPeriod = CorrectionPeriod;
    type Currency = Balances;
    type RuntimeEvent = RuntimeEvent;
    type DisputeResolution = prediction_markets::Pallet<Runtime>;
    type MarketCommons = MarketCommons;
    type PalletId = AuthorizedPalletId;
    type WeightInfo = zrml_authorized::weights::WeightInfo<Runtime>;
}

impl zrml_court::Config for Runtime {
    type AppealBond = AppealBond;
    type BlocksPerYear = BlocksPerYear;
    type DisputeResolution = prediction_markets::Pallet<Runtime>;
    type VotePeriod = VotePeriod;
    type AggregationPeriod = AggregationPeriod;
    type AppealPeriod = AppealPeriod;
    type LockId = LockId;
    type Currency = Balances;
    type RuntimeEvent = RuntimeEvent;
    type InflationPeriod = InflationPeriod;
    type MarketCommons = MarketCommons;
    type MaxAppeals = MaxAppeals;
    type MaxDelegations = MaxDelegations;
    type MaxSelectedDraws = MaxSelectedDraws;
    type MaxCourtParticipants = MaxCourtParticipants;
    type MaxYearlyInflation = MaxYearlyInflation;
    type MinJurorStake = MinJurorStake;
    type MonetaryGovernanceOrigin = EnsureRoot<AccountIdTest>;
    type PalletId = CourtPalletId;
    type Random = RandomnessCollectiveFlip;
    type RequestInterval = RequestInterval;
    type Slash = Treasury;
    type TreasuryPalletId = TreasuryPalletId;
    type WeightInfo = zrml_court::weights::WeightInfo<Runtime>;
}

impl zrml_market_commons::Config for Runtime {
    type Balance = Balance;
    type MarketId = MarketId;
    type Timestamp = Timestamp;
}

impl zrml_global_disputes::Config for Runtime {
    type AddOutcomePeriod = AddOutcomePeriod;
    type RuntimeEvent = RuntimeEvent;
    type DisputeResolution = prediction_markets::Pallet<Runtime>;
    type MarketCommons = MarketCommons;
    type Currency = Balances;
    type GlobalDisputeLockId = GlobalDisputeLockId;
    type GlobalDisputesPalletId = GlobalDisputesPalletId;
    type MaxGlobalDisputeVotes = MaxGlobalDisputeVotes;
    type MaxOwners = MaxOwners;
    type MinOutcomeVoteAmount = MinOutcomeVoteAmount;
    type RemoveKeysLimit = RemoveKeysLimit;
    type GdVotingPeriod = GdVotingPeriod;
    type VotingOutcomeFee = VotingOutcomeFee;
    type WeightInfo = zrml_global_disputes::weights::WeightInfo<Runtime>;
}

#[cfg(feature = "runtime-benchmarks")]
pub struct BenchmarkHelper;
#[cfg(feature = "runtime-benchmarks")]
impl ArgumentsFactory<(), AccountIdTest> for BenchmarkHelper {
    fn create_asset_kind(_seed: u32) {
        // No-op
    }

    fn create_beneficiary(seed: [u8; 32]) -> AccountIdTest {
        let h160 = H160::from(H256::from(seed));
        let lower_128: u128 = u128::from_le_bytes(h160.as_bytes()[..16].try_into().unwrap());
        AccountIdTest::from(lower_128)
    }
}

impl pallet_treasury::Config for Runtime {
    type AssetKind = ();
    type BalanceConverter = UnityAssetBalanceConversion;
    type Beneficiary = AccountIdTest;
    type BeneficiaryLookup = IdentityLookup<AccountIdTest>;
    type Burn = ();
    type BurnDestination = ();
    type Currency = Balances;
    type RuntimeEvent = RuntimeEvent;
    type MaxApprovals = MaxApprovals;
    type PalletId = TreasuryPalletId;
    type Paymaster = PayFromAccount<Balances, TreasuryAccount>;
    type PayoutPeriod = ();
    type RejectOrigin = EnsureSignedBy<Sudo, AccountIdTest>;
    type SpendFunds = ();
    type SpendOrigin = NeverEnsureOrigin<Balance>;
    type SpendPeriod = ();
    type WeightInfo = ();
    #[cfg(feature = "runtime-benchmarks")]
    type BenchmarkHelper = BenchmarkHelper;
}

pub struct ExtBuilder {
    balances: Vec<(AccountIdTest, Balance)>,
}

impl Default for ExtBuilder {
    fn default() -> Self {
        DEPLOY_POOL_CALL_DATA.with(|value| value.borrow_mut().clear());
        Self {
            balances: vec![
                (ALICE, INITIAL_BALANCE),
                (BOB, INITIAL_BALANCE),
                (CHARLIE, INITIAL_BALANCE),
                (DAVE, INITIAL_BALANCE),
                (EVE, INITIAL_BALANCE),
                (FRED, INITIAL_BALANCE),
                (SUDO, INITIAL_BALANCE),
            ],
        }
    }
}

impl ExtBuilder {
    pub fn build(self) -> sp_io::TestExternalities {
        let mut t = frame_system::GenesisConfig::<Runtime>::default().build_storage().unwrap();

        // see the logs in tests when using `RUST_LOG=debug cargo test -- --nocapture`
        let _ = env_logger::builder().is_test(true).try_init();

        pallet_balances::GenesisConfig::<Runtime> { balances: self.balances }
            .assimilate_storage(&mut t)
            .unwrap();

        #[cfg(feature = "parachain")]
        {
            orml_tokens::GenesisConfig::<Runtime> {
                balances: (0..69)
                    .map(|idx| (idx, Asset::ForeignAsset(100), INITIAL_BALANCE))
                    .collect(),
            }
            .assimilate_storage(&mut t)
            .unwrap();

            let custom_metadata = zeitgeist_primitives::types::CustomMetadata {
                allow_as_base_asset: true,
                ..Default::default()
            };

            orml_asset_registry::module::GenesisConfig::<Runtime> {
                assets: vec![
                    (
                        Asset::ForeignAsset(100),
                        AssetMetadata {
                            decimals: 18,
                            name: "ACALA USD".as_bytes().to_vec().try_into().unwrap(),
                            symbol: "AUSD".as_bytes().to_vec().try_into().unwrap(),
                            existential_deposit: 0,
                            location: None,
                            additional: custom_metadata,
                        }
                        .encode(),
                    ),
                    (
                        Asset::ForeignAsset(420),
                        AssetMetadata {
                            decimals: 18,
                            name: "FANCY_TOKEN".as_bytes().to_vec().try_into().unwrap(),
                            symbol: "FTK".as_bytes().to_vec().try_into().unwrap(),
                            existential_deposit: 0,
                            location: None,
                            additional: zeitgeist_primitives::types::CustomMetadata::default(),
                        }
                        .encode(),
                    ),
                ],
                last_asset_id: Asset::ForeignAsset(420),
            }
            .assimilate_storage(&mut t)
            .unwrap();
        }

        let mut test_ext: sp_io::TestExternalities = t.into();
        test_ext.execute_with(|| System::set_block_number(1));
        test_ext
    }
}

pub fn run_to_block(n: BlockNumber) {
    while System::block_number() < n {
        Balances::on_finalize(System::block_number());
        Court::on_finalize(System::block_number());
        PredictionMarkets::on_finalize(System::block_number());
        System::on_finalize(System::block_number());
        System::set_block_number(System::block_number() + 1);
        System::on_initialize(System::block_number());
        PredictionMarkets::on_initialize(System::block_number());
        Court::on_initialize(System::block_number());
        Balances::on_initialize(System::block_number());
    }
}

pub fn run_blocks(n: BlockNumber) {
    run_to_block(System::block_number() + n);
}

// Our `on_initialize` compensates for the fact that `on_initialize` takes the timestamp from the
// previous block. Therefore, manually setting timestamp during tests becomes cumbersome without a
// utility function like this.
pub fn set_timestamp_for_on_initialize(time: Moment) {
    Timestamp::set_timestamp(time - MILLISECS_PER_BLOCK as u64);
}

type Block = MockBlock<Runtime>;

sp_api::mock_impl_runtime_apis! {
    impl zrml_prediction_markets_runtime_api::PredictionMarketsApi<BlockTest<Runtime>, MarketId, Hash> for Runtime {
        fn market_outcome_share_id(_: MarketId, _: u16) -> Asset<MarketId> {
            Asset::PoolShare(1)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Config;

    // We run this test to ensure that bonds are mutually non-equal (some of the tests in
    // `tests.rs` require this to be true).
    #[test]
    fn test_bonds_are_pairwise_non_equal() {
        assert_ne!(
            <Runtime as Config>::AdvisoryBond::get(),
            <Runtime as Config>::OracleBond::get()
        );
        assert_ne!(
            <Runtime as Config>::AdvisoryBond::get(),
            <Runtime as Config>::ValidityBond::get()
        );
        assert_ne!(
            <Runtime as Config>::AdvisoryBond::get(),
            <Runtime as Config>::DisputeBond::get()
        );
        assert_ne!(
            <Runtime as Config>::OracleBond::get(),
            <Runtime as Config>::ValidityBond::get()
        );
        assert_ne!(<Runtime as Config>::OracleBond::get(), <Runtime as Config>::DisputeBond::get());
        assert_ne!(
            <Runtime as Config>::ValidityBond::get(),
            <Runtime as Config>::DisputeBond::get()
        );
    }
}
