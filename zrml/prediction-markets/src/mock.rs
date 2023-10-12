// Copyright 2022-2023 Forecasting Technologies LTD.
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
    traits::{Everything, NeverEnsureOrigin, OnFinalize, OnInitialize},
};
use frame_system::{EnsureRoot, EnsureSignedBy};
#[cfg(feature = "parachain")]
use orml_asset_registry::AssetMetadata;
use sp_arithmetic::per_things::Percent;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
    DispatchError, DispatchResult,
};
use std::cell::RefCell;
use substrate_fixed::{types::extra::U33, FixedI128, FixedU128};
use zeitgeist_primitives::{
    constants::mock::{
        AddOutcomePeriod, AggregationPeriod, AppealBond, AppealPeriod, AuthorizedPalletId,
        BalanceFractionalDecimals, BlockHashCount, BlocksPerYear, CorrectionPeriod, CourtPalletId,
        ExistentialDeposit, ExistentialDeposits, ExitFee, GdVotingPeriod, GetNativeCurrencyId,
        GlobalDisputeLockId, GlobalDisputesPalletId, InflationPeriod, LiquidityMiningPalletId,
        LockId, MaxAppeals, MaxApprovals, MaxAssets, MaxCategories, MaxCourtParticipants,
        MaxCreatorFee, MaxDelegations, MaxDisputeDuration, MaxDisputes, MaxEditReasonLen,
        MaxGlobalDisputeVotes, MaxGracePeriod, MaxInRatio, MaxMarketLifetime, MaxOracleDuration,
        MaxOutRatio, MaxOwners, MaxRejectReasonLen, MaxReserves, MaxSelectedDraws,
        MaxSubsidyPeriod, MaxSwapFee, MaxTotalWeight, MaxWeight, MinAssets, MinCategories,
        MinDisputeDuration, MinJurorStake, MinOracleDuration, MinOutcomeVoteAmount, MinSubsidy,
        MinSubsidyPeriod, MinWeight, MinimumPeriod, OutcomeBond, OutcomeFactor, OutsiderBond,
        PmPalletId, RemoveKeysLimit, RequestInterval, SimpleDisputesPalletId, SwapsPalletId,
        TreasuryPalletId, VotePeriod, VotingOutcomeFee, BASE, CENT, MILLISECS_PER_BLOCK,
    },
    traits::DeployPoolApi,
    types::{
        AccountIdTest, Amount, Asset, Balance, BasicCurrencyAdapter, BlockNumber, BlockTest,
        CurrencyId, Hash, Index, MarketId, Moment, PoolId, SerdeWrapper, UncheckedExtrinsicTest,
    },
};
use zrml_rikiddo::types::{EmaMarketVolume, FeeSigmoid, RikiddoSigmoidMV};

pub const ALICE: AccountIdTest = 0;
pub const BOB: AccountIdTest = 1;
pub const CHARLIE: AccountIdTest = 2;
pub const DAVE: AccountIdTest = 3;
pub const EVE: AccountIdTest = 4;
pub const FRED: AccountIdTest = 5;
pub const SUDO: AccountIdTest = 69;

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
    pub static DEPLOY_POOL_CALL_DATA: RefCell<Vec<DeployPoolArgs>> = RefCell::new(vec![]);
    pub static DEPLOY_POOL_RETURN_VALUE: RefCell<DispatchResult> = RefCell::new(Ok(()));
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
}
parameter_types! {
    pub const MinSubsidyPerAccount: Balance = BASE;
    pub const AdvisoryBond: Balance = 11 * CENT;
    pub const AdvisoryBondSlashPercentage: Percent = Percent::from_percent(10);
    pub const OracleBond: Balance = 25 * CENT;
    pub const ValidityBond: Balance = 53 * CENT;
    pub const DisputeBond: Balance = 109 * CENT;
}

construct_runtime!(
    pub enum Runtime
    where
        Block = BlockTest<Runtime>,
        NodeBlock = BlockTest<Runtime>,
        UncheckedExtrinsic = UncheckedExtrinsicTest<Runtime>,
    {
        Authorized: zrml_authorized::{Event<T>, Pallet, Storage},
        Balances: pallet_balances::{Call, Config<T>, Event<T>, Pallet, Storage},
        Court: zrml_court::{Event<T>, Pallet, Storage},
        AssetManager: orml_currencies::{Call, Pallet, Storage},
        LiquidityMining: zrml_liquidity_mining::{Config<T>, Event<T>, Pallet},
        MarketCommons: zrml_market_commons::{Pallet, Storage},
        PredictionMarkets: prediction_markets::{Event<T>, Pallet, Storage},
        RandomnessCollectiveFlip: pallet_randomness_collective_flip::{Pallet, Storage},
        RikiddoSigmoidFeeMarketEma: zrml_rikiddo::{Pallet, Storage},
        SimpleDisputes: zrml_simple_disputes::{Event<T>, Pallet, Storage},
        GlobalDisputes: zrml_global_disputes::{Event<T>, Pallet, Storage},
        Swaps: zrml_swaps::{Call, Event<T>, Pallet},
        System: frame_system::{Call, Config, Event<T>, Pallet, Storage},
        Timestamp: pallet_timestamp::{Pallet},
        Tokens: orml_tokens::{Config<T>, Event<T>, Pallet, Storage},
        Treasury: pallet_treasury::{Call, Event<T>, Pallet, Storage},
    }
);

impl crate::Config for Runtime {
    type AdvisoryBond = AdvisoryBond;
    type AdvisoryBondSlashPercentage = AdvisoryBondSlashPercentage;
    type ApproveOrigin = EnsureSignedBy<Sudo, AccountIdTest>;
    #[cfg(feature = "parachain")]
    type AssetRegistry = MockRegistry;
    type Authorized = Authorized;
    type CloseOrigin = EnsureSignedBy<Sudo, AccountIdTest>;
    type Currency = Balances;
    type MaxCreatorFee = MaxCreatorFee;
    type Court = Court;
    type DestroyOrigin = EnsureSignedBy<Sudo, AccountIdTest>;
    type DeployPool = DeployPoolMock;
    type DisputeBond = DisputeBond;
    type RuntimeEvent = RuntimeEvent;
    type GlobalDisputes = GlobalDisputes;
    type LiquidityMining = LiquidityMining;
    type MaxCategories = MaxCategories;
    type MaxDisputes = MaxDisputes;
    type MinDisputeDuration = MinDisputeDuration;
    type MinOracleDuration = MinOracleDuration;
    type MaxDisputeDuration = MaxDisputeDuration;
    type MaxGracePeriod = MaxGracePeriod;
    type MaxOracleDuration = MaxOracleDuration;
    type MaxSubsidyPeriod = MaxSubsidyPeriod;
    type MaxMarketLifetime = MaxMarketLifetime;
    type MinCategories = MinCategories;
    type MinSubsidyPeriod = MinSubsidyPeriod;
    type MaxEditReasonLen = MaxEditReasonLen;
    type MaxRejectReasonLen = MaxRejectReasonLen;
    type OracleBond = OracleBond;
    type OutsiderBond = OutsiderBond;
    type PalletId = PmPalletId;
    type RejectOrigin = EnsureSignedBy<Sudo, AccountIdTest>;
    type RequestEditOrigin = EnsureSignedBy<Sudo, AccountIdTest>;
    type ResolveOrigin = EnsureSignedBy<Sudo, AccountIdTest>;
    type AssetManager = AssetManager;
    type SimpleDisputes = SimpleDisputes;
    type Slash = Treasury;
    type Swaps = Swaps;
    type ValidityBond = ValidityBond;
    type WeightInfo = prediction_markets::weights::WeightInfo<Runtime>;
}

impl frame_system::Config for Runtime {
    type AccountData = pallet_balances::AccountData<Balance>;
    type AccountId = AccountIdTest;
    type BaseCallFilter = Everything;
    type BlockHashCount = BlockHashCount;
    type BlockLength = ();
    type BlockNumber = BlockNumber;
    type BlockWeights = ();
    type RuntimeCall = RuntimeCall;
    type DbWeight = ();
    type RuntimeEvent = RuntimeEvent;
    type Hash = Hash;
    type Hashing = BlakeTwo256;
    type Header = Header;
    type Index = Index;
    type Lookup = IdentityLookup<Self::AccountId>;
    type MaxConsumers = frame_support::traits::ConstU32<16>;
    type OnKilledAccount = ();
    type OnNewAccount = ();
    type OnSetCode = ();
    type RuntimeOrigin = RuntimeOrigin;
    type PalletInfo = PalletInfo;
    type SS58Prefix = ();
    type SystemWeightInfo = ();
    type Version = ();
}

impl orml_currencies::Config for Runtime {
    type GetNativeCurrencyId = GetNativeCurrencyId;
    type MultiCurrency = Tokens;
    type NativeCurrency = BasicCurrencyAdapter<Runtime, Balances>;
    type WeightInfo = ();
}

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

#[cfg(feature = "parachain")]
crate::orml_asset_registry::impl_mock_registry! {
    MockRegistry,
    CurrencyId,
    Balance,
    zeitgeist_primitives::types::CustomMetadata
}

impl pallet_balances::Config for Runtime {
    type AccountStore = System;
    type Balance = Balance;
    type DustRemoval = ();
    type RuntimeEvent = RuntimeEvent;
    type ExistentialDeposit = ExistentialDeposit;
    type MaxLocks = ();
    type MaxReserves = MaxReserves;
    type ReserveIdentifier = [u8; 8];
    type WeightInfo = ();
}

impl pallet_randomness_collective_flip::Config for Runtime {}

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
    type MinJurorStake = MinJurorStake;
    type MonetaryGovernanceOrigin = EnsureRoot<AccountIdTest>;
    type PalletId = CourtPalletId;
    type Random = RandomnessCollectiveFlip;
    type RequestInterval = RequestInterval;
    type Slash = Treasury;
    type TreasuryPalletId = TreasuryPalletId;
    type WeightInfo = zrml_court::weights::WeightInfo<Runtime>;
}

impl zrml_liquidity_mining::Config for Runtime {
    type Currency = Balances;
    type RuntimeEvent = RuntimeEvent;
    type MarketCommons = MarketCommons;
    type MarketId = MarketId;
    type PalletId = LiquidityMiningPalletId;
    type WeightInfo = zrml_liquidity_mining::weights::WeightInfo<Runtime>;
}

impl zrml_market_commons::Config for Runtime {
    type Balance = Balance;
    type MarketId = MarketId;
    type PredictionMarketsPalletId = PmPalletId;
    type Timestamp = Timestamp;
}

impl zrml_rikiddo::Config for Runtime {
    type Timestamp = Timestamp;
    type Balance = Balance;
    type FixedTypeU = FixedU128<U33>;
    type FixedTypeS = FixedI128<U33>;
    type BalanceFractionalDecimals = BalanceFractionalDecimals;
    type PoolId = PoolId;
    type Rikiddo = RikiddoSigmoidMV<
        Self::FixedTypeU,
        Self::FixedTypeS,
        FeeSigmoid<Self::FixedTypeS>,
        EmaMarketVolume<Self::FixedTypeU>,
    >;
}

impl zrml_simple_disputes::Config for Runtime {
    type Currency = Balances;
    type RuntimeEvent = RuntimeEvent;
    type OutcomeBond = OutcomeBond;
    type OutcomeFactor = OutcomeFactor;
    type DisputeResolution = prediction_markets::Pallet<Runtime>;
    type MarketCommons = MarketCommons;
    type MaxDisputes = MaxDisputes;
    type PalletId = SimpleDisputesPalletId;
    type WeightInfo = zrml_simple_disputes::weights::WeightInfo<Runtime>;
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

impl zrml_swaps::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type ExitFee = ExitFee;
    type FixedTypeU = <Runtime as zrml_rikiddo::Config>::FixedTypeU;
    type FixedTypeS = <Runtime as zrml_rikiddo::Config>::FixedTypeS;
    type LiquidityMining = LiquidityMining;
    type MarketCommons = MarketCommons;
    type MaxAssets = MaxAssets;
    type MaxInRatio = MaxInRatio;
    type MaxOutRatio = MaxOutRatio;
    type MaxSwapFee = MaxSwapFee;
    type MaxTotalWeight = MaxTotalWeight;
    type MaxWeight = MaxWeight;
    type MinAssets = MinAssets;
    type MinSubsidy = MinSubsidy;
    type MinSubsidyPerAccount = MinSubsidyPerAccount;
    type MinWeight = MinWeight;
    type PalletId = SwapsPalletId;
    type RikiddoSigmoidFeeMarketEma = RikiddoSigmoidFeeMarketEma;
    type AssetManager = AssetManager;
    type WeightInfo = zrml_swaps::weights::WeightInfo<Runtime>;
}

impl pallet_treasury::Config for Runtime {
    type ApproveOrigin = EnsureSignedBy<Sudo, AccountIdTest>;
    type Burn = ();
    type BurnDestination = ();
    type Currency = Balances;
    type RuntimeEvent = RuntimeEvent;
    type MaxApprovals = MaxApprovals;
    type OnSlash = ();
    type PalletId = TreasuryPalletId;
    type ProposalBond = ();
    type ProposalBondMinimum = ();
    type ProposalBondMaximum = ();
    type RejectOrigin = EnsureSignedBy<Sudo, AccountIdTest>;
    type SpendFunds = ();
    type SpendOrigin = NeverEnsureOrigin<Balance>;
    type SpendPeriod = ();
    type WeightInfo = ();
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
        let mut t = frame_system::GenesisConfig::default().build_storage::<Runtime>().unwrap();

        pallet_balances::GenesisConfig::<Runtime> { balances: self.balances }
            .assimilate_storage(&mut t)
            .unwrap();
        #[cfg(feature = "parachain")]
        use frame_support::traits::GenesisBuild;
        #[cfg(feature = "parachain")]
        orml_tokens::GenesisConfig::<Runtime> {
            balances: (0..69)
                .map(|idx| (idx, CurrencyId::ForeignAsset(100), INITIAL_BALANCE))
                .collect(),
        }
        .assimilate_storage(&mut t)
        .unwrap();
        #[cfg(feature = "parachain")]
        let custom_metadata = zeitgeist_primitives::types::CustomMetadata {
            allow_as_base_asset: true,
            ..Default::default()
        };
        #[cfg(feature = "parachain")]
        orml_asset_registry_mock::GenesisConfig {
            metadata: vec![
                (
                    CurrencyId::ForeignAsset(100),
                    AssetMetadata {
                        decimals: 18,
                        name: "ACALA USD".as_bytes().to_vec(),
                        symbol: "AUSD".as_bytes().to_vec(),
                        existential_deposit: 0,
                        location: None,
                        additional: custom_metadata,
                    },
                ),
                (
                    CurrencyId::ForeignAsset(420),
                    AssetMetadata {
                        decimals: 18,
                        name: "FANCY_TOKEN".as_bytes().to_vec(),
                        symbol: "FTK".as_bytes().to_vec(),
                        existential_deposit: 0,
                        location: None,
                        additional: zeitgeist_primitives::types::CustomMetadata::default(),
                    },
                ),
            ],
        }
        .assimilate_storage(&mut t)
        .unwrap();
        t.into()
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

sp_api::mock_impl_runtime_apis! {
    impl zrml_prediction_markets_runtime_api::PredictionMarketsApi<BlockTest<Runtime>, MarketId, Hash> for Runtime {
        fn market_outcome_share_id(_: MarketId, _: u16) -> Asset<MarketId> {
            Asset::PoolShare(SerdeWrapper(1))
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
