// Copyright 2024 Forecasting Technologies LTD.
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

#![cfg(feature = "mock")]
#![allow(
    // Mocks are only used for fuzzing and unit tests
    clippy::arithmetic_side_effects,
    clippy::too_many_arguments,
)]

use crate as zrml_hybrid_router;
use crate::{AssetOf, BalanceOf, MarketIdOf};
use core::marker::PhantomData;
use frame_support::{
    construct_runtime, ord_parameter_types, parameter_types,
    traits::{AsEnsureOriginWithArg, Contains, Everything, NeverEnsureOrigin},
};
use frame_system::{EnsureRoot, EnsureSigned, EnsureSignedBy};
#[cfg(feature = "parachain")]
use orml_asset_registry::AssetMetadata;
use orml_traits::MultiCurrency;
use parity_scale_codec::Compact;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, ConstU32, Get, IdentityLookup, Zero},
    Perbill, Percent, SaturatedConversion,
};
#[cfg(feature = "parachain")]
use zeitgeist_primitives::types::Asset;
use zeitgeist_primitives::{
    constants::mock::{
        AddOutcomePeriod, AggregationPeriod, AppealBond, AppealPeriod, AssetsAccountDeposit,
        AssetsApprovalDeposit, AssetsDeposit, AssetsMetadataDepositBase,
        AssetsMetadataDepositPerByte, AssetsStringLimit, AuthorizedPalletId, BlockHashCount,
        BlocksPerYear, CloseEarlyBlockPeriod, CloseEarlyDisputeBond,
        CloseEarlyProtectionBlockPeriod, CloseEarlyProtectionTimeFramePeriod,
        CloseEarlyRequestBond, CloseEarlyTimeFramePeriod, CorrectionPeriod, CourtPalletId,
        DestroyAccountWeight, DestroyApprovalWeight, DestroyFinishWeight, ExistentialDeposit,
        ExistentialDeposits, GdVotingPeriod, GetNativeCurrencyId, GlobalDisputeLockId,
        GlobalDisputesPalletId, HybridRouterPalletId, InflationPeriod, LiquidityMiningPalletId,
        LockId, MaxAppeals, MaxApprovals, MaxCourtParticipants, MaxCreatorFee, MaxDelegations,
        MaxDisputeDuration, MaxDisputes, MaxEditReasonLen, MaxGlobalDisputeVotes, MaxGracePeriod,
        MaxLiquidityTreeDepth, MaxLocks, MaxMarketLifetime, MaxOracleDuration, MaxOrders,
        MaxOwners, MaxRejectReasonLen, MaxReserves, MaxSelectedDraws, MaxYearlyInflation,
        MinCategories, MinDisputeDuration, MinJurorStake, MinOracleDuration, MinOutcomeVoteAmount,
        MinimumPeriod, NeoMaxSwapFee, NeoSwapsPalletId, OrderbookPalletId, OutcomeBond,
        OutcomeFactor, OutsiderBond, PmPalletId, RemoveKeysLimit, RequestInterval,
        SimpleDisputesPalletId, TreasuryPalletId, VotePeriod, VotingOutcomeFee, BASE, CENT,
        MAX_ASSETS,
    },
    traits::DistributeFees,
    types::{
        AccountIdTest, Amount, Assets, Balance, BasicCurrencyAdapter, BlockNumber, BlockTest,
        CampaignAsset, CampaignAssetId, Currencies, CustomAsset, CustomAssetId, Hash, Index,
        MarketAsset, MarketId, Moment, UncheckedExtrinsicTest,
    },
};

pub const ALICE: AccountIdTest = 0;
#[allow(unused)]
pub const BOB: AccountIdTest = 1;
pub const CHARLIE: AccountIdTest = 2;
pub const DAVE: AccountIdTest = 3;
pub const EVE: AccountIdTest = 4;
pub const FEE_ACCOUNT: AccountIdTest = 5;
pub const SUDO: AccountIdTest = 123456;
pub const EXTERNAL_FEES: Balance = CENT;
pub const INITIAL_BALANCE: Balance = 100 * BASE;
pub const MARKET_CREATOR: AccountIdTest = ALICE;

#[cfg(feature = "parachain")]
pub const FOREIGN_ASSET: Asset<MarketId> = Asset::ForeignAsset(1);

parameter_types! {
    pub const FeeAccount: AccountIdTest = FEE_ACCOUNT;
}
ord_parameter_types! {
    pub const AuthorizedDisputeResolutionUser: AccountIdTest = ALICE;
}
ord_parameter_types! {
    pub const Sudo: AccountIdTest = SUDO;
}
parameter_types! {
    pub storage NeoMinSwapFee: Balance = 0;
}
parameter_types! {
    pub const AdvisoryBond: Balance = 0;
    pub const AdvisoryBondSlashPercentage: Percent = Percent::from_percent(10);
    pub const OracleBond: Balance = 0;
    pub const ValidityBond: Balance = 0;
    pub const DisputeBond: Balance = 0;
    pub const MaxCategories: u16 = MAX_ASSETS + 1;
}

pub fn fee_percentage() -> Perbill {
    Perbill::from_rational(EXTERNAL_FEES, BASE)
}

pub fn calculate_fee<T: crate::Config>(amount: BalanceOf<T>) -> BalanceOf<T> {
    fee_percentage().mul_floor(amount.saturated_into::<BalanceOf<T>>())
}

pub struct ExternalFees<T, F>(PhantomData<T>, PhantomData<F>);

impl<T: crate::Config, F> DistributeFees for ExternalFees<T, F>
where
    F: Get<T::AccountId>,
{
    type Asset = AssetOf<T>;
    type AccountId = T::AccountId;
    type Balance = BalanceOf<T>;
    type MarketId = MarketIdOf<T>;

    fn distribute(
        _market_id: Self::MarketId,
        asset: Self::Asset,
        account: &Self::AccountId,
        amount: Self::Balance,
    ) -> Self::Balance {
        let fees = calculate_fee::<T>(amount);
        match T::AssetManager::transfer(asset, account, &F::get(), fees) {
            Ok(_) => fees,
            Err(_) => Zero::zero(),
        }
    }

    fn fee_percentage(_market_id: Self::MarketId) -> Perbill {
        fee_percentage()
    }
}

pub type UncheckedExtrinsic = UncheckedExtrinsicTest<Runtime>;

pub struct DustRemovalWhitelist;

impl Contains<AccountIdTest> for DustRemovalWhitelist {
    fn contains(account_id: &AccountIdTest) -> bool {
        *account_id == FEE_ACCOUNT
    }
}

pub(super) type CustomAssetsInstance = pallet_assets::Instance1;
pub(super) type CampaignAssetsInstance = pallet_assets::Instance2;
pub(super) type MarketAssetsInstance = pallet_assets::Instance3;

construct_runtime!(
    pub enum Runtime
    where
        Block = BlockTest<Runtime>,
        NodeBlock = BlockTest<Runtime>,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        HybridRouter: zrml_hybrid_router::{Pallet, Call, Storage, Event<T>},
        Orderbook: zrml_orderbook::{Call, Event<T>, Pallet, Storage},
        AssetRouter: zrml_asset_router::{Pallet},
        NeoSwaps: zrml_neo_swaps::{Call, Event<T>, Pallet},
        Authorized: zrml_authorized::{Event<T>, Pallet, Storage},
        Balances: pallet_balances::{Call, Config<T>, Event<T>, Pallet, Storage},
        Court: zrml_court::{Event<T>, Pallet, Storage},
        CampaignAssets: pallet_assets::<Instance2>::{Call, Pallet, Storage, Event<T>},
        CustomAssets: pallet_assets::<Instance1>::{Call, Pallet, Storage, Event<T>},
        MarketAssets: pallet_assets::<Instance3>::{Call, Pallet, Storage, Event<T>},
        AssetManager: orml_currencies::{Call, Pallet, Storage},
        LiquidityMining: zrml_liquidity_mining::{Config<T>, Event<T>, Pallet},
        MarketCommons: zrml_market_commons::{Pallet, Storage},
        PredictionMarkets: zrml_prediction_markets::{Event<T>, Pallet, Storage},
        RandomnessCollectiveFlip: pallet_randomness_collective_flip::{Pallet, Storage},
        SimpleDisputes: zrml_simple_disputes::{Event<T>, Pallet, Storage},
        GlobalDisputes: zrml_global_disputes::{Event<T>, Pallet, Storage},
        System: frame_system::{Call, Config, Event<T>, Pallet, Storage},
        Timestamp: pallet_timestamp::{Pallet},
        Tokens: orml_tokens::{Config<T>, Event<T>, Pallet, Storage},
        Treasury: pallet_treasury::{Call, Event<T>, Pallet, Storage},
    }
);

impl crate::Config for Runtime {
    type AssetManager = AssetManager;
    #[cfg(feature = "runtime-benchmarks")]
    type AmmPoolDeployer = NeoSwaps;
    type Amm = NeoSwaps;
    #[cfg(feature = "runtime-benchmarks")]
    type CompleteSetOperations = PredictionMarkets;
    type MarketCommons = MarketCommons;
    type Orderbook = Orderbook;
    type RuntimeEvent = RuntimeEvent;
    type MaxOrders = MaxOrders;
    type PalletId = HybridRouterPalletId;
    type WeightInfo = zrml_hybrid_router::weights::WeightInfo<Runtime>;
}

pallet_assets::runtime_benchmarks_enabled! {
    pub struct AssetsBenchmarkHelper;

    impl<AssetIdParameter> pallet_assets::BenchmarkHelper<AssetIdParameter>
        for AssetsBenchmarkHelper
    where
        AssetIdParameter: From<u128>,
    {
        fn create_asset_id_parameter(id: u32) -> AssetIdParameter {
            (id as u128).into()
        }
    }
}

pallet_assets::runtime_benchmarks_enabled! {
    use zeitgeist_primitives::types::CategoryIndex;

    pub struct MarketAssetsBenchmarkHelper;

    impl pallet_assets::BenchmarkHelper<MarketAsset>
        for MarketAssetsBenchmarkHelper
    {
        fn create_asset_id_parameter(id: u32) -> MarketAsset {
            MarketAsset::CategoricalOutcome(0, id as CategoryIndex)
        }
    }
}

impl pallet_assets::Config<CampaignAssetsInstance> for Runtime {
    type ApprovalDeposit = AssetsApprovalDeposit;
    type AssetAccountDeposit = AssetsAccountDeposit;
    type AssetDeposit = AssetsDeposit;
    type AssetId = CampaignAsset;
    type AssetIdParameter = Compact<CampaignAssetId>;
    type Balance = Balance;
    #[cfg(feature = "runtime-benchmarks")]
    type BenchmarkHelper = AssetsBenchmarkHelper;
    type CallbackHandle = ();
    type CreateOrigin = AsEnsureOriginWithArg<EnsureSigned<Self::AccountId>>;
    type Currency = Balances;
    type Extra = ();
    type ForceOrigin = EnsureRoot<AccountIdTest>;
    type Freezer = ();
    type Destroyer = AssetRouter;
    type MetadataDepositBase = AssetsMetadataDepositBase;
    type MetadataDepositPerByte = AssetsMetadataDepositPerByte;
    type RemoveItemsLimit = ConstU32<50>;
    type RuntimeEvent = RuntimeEvent;
    type StringLimit = AssetsStringLimit;
    type WeightInfo = ();
}

impl pallet_assets::Config<CustomAssetsInstance> for Runtime {
    type ApprovalDeposit = AssetsApprovalDeposit;
    type AssetAccountDeposit = AssetsAccountDeposit;
    type AssetDeposit = AssetsDeposit;
    type AssetId = CustomAsset;
    type AssetIdParameter = Compact<CustomAssetId>;
    type Balance = Balance;
    #[cfg(feature = "runtime-benchmarks")]
    type BenchmarkHelper = AssetsBenchmarkHelper;
    type CallbackHandle = ();
    type CreateOrigin = AsEnsureOriginWithArg<EnsureSigned<Self::AccountId>>;
    type Currency = Balances;
    type Extra = ();
    type ForceOrigin = EnsureRoot<AccountIdTest>;
    type Freezer = ();
    type Destroyer = AssetRouter;
    type MetadataDepositBase = AssetsMetadataDepositBase;
    type MetadataDepositPerByte = AssetsMetadataDepositPerByte;
    type RemoveItemsLimit = ConstU32<50>;
    type RuntimeEvent = RuntimeEvent;
    type StringLimit = AssetsStringLimit;
    type WeightInfo = ();
}

impl pallet_assets::Config<MarketAssetsInstance> for Runtime {
    type ApprovalDeposit = AssetsApprovalDeposit;
    type AssetAccountDeposit = AssetsAccountDeposit;
    type AssetDeposit = AssetsDeposit;
    type AssetId = MarketAsset;
    type AssetIdParameter = MarketAsset;
    type Balance = Balance;
    #[cfg(feature = "runtime-benchmarks")]
    type BenchmarkHelper = MarketAssetsBenchmarkHelper;
    type CallbackHandle = ();
    type CreateOrigin = AsEnsureOriginWithArg<EnsureSigned<Self::AccountId>>;
    type Currency = Balances;
    type Extra = ();
    type ForceOrigin = EnsureRoot<AccountIdTest>;
    type Freezer = ();
    type Destroyer = AssetRouter;
    type MetadataDepositBase = AssetsMetadataDepositBase;
    type MetadataDepositPerByte = AssetsMetadataDepositPerByte;
    type RemoveItemsLimit = ConstU32<50>;
    type RuntimeEvent = RuntimeEvent;
    type StringLimit = AssetsStringLimit;
    type WeightInfo = ();
}

impl zrml_asset_router::Config for Runtime {
    type AssetType = Assets;
    type Balance = Balance;
    type CurrencyType = Currencies;
    type Currencies = Tokens;
    type CampaignAssetType = CampaignAsset;
    type CampaignAssets = CampaignAssets;
    type CustomAssetType = CustomAsset;
    type CustomAssets = CustomAssets;
    type DestroyAccountWeight = DestroyAccountWeight;
    type DestroyApprovalWeight = DestroyApprovalWeight;
    type DestroyFinishWeight = DestroyFinishWeight;
    type MarketAssetType = MarketAsset;
    type MarketAssets = MarketAssets;
}

impl zrml_orderbook::Config for Runtime {
    type AssetManager = AssetManager;
    type ExternalFees = ExternalFees<Runtime, FeeAccount>;
    type RuntimeEvent = RuntimeEvent;
    type MarketCommons = MarketCommons;
    type PalletId = OrderbookPalletId;
    type WeightInfo = zrml_orderbook::weights::WeightInfo<Runtime>;
}

impl zrml_neo_swaps::Config for Runtime {
    type MultiCurrency = AssetManager;
    type CompleteSetOperations = PredictionMarkets;
    type ExternalFees = ExternalFees<Runtime, FeeAccount>;
    type MarketCommons = MarketCommons;
    type RuntimeEvent = RuntimeEvent;
    type MaxLiquidityTreeDepth = MaxLiquidityTreeDepth;
    type MaxSwapFee = NeoMaxSwapFee;
    type PalletId = NeoSwapsPalletId;
    type WeightInfo = zrml_neo_swaps::weights::WeightInfo<Runtime>;
}

impl pallet_randomness_collective_flip::Config for Runtime {}

impl zrml_prediction_markets::Config for Runtime {
    type AdvisoryBond = AdvisoryBond;
    type AdvisoryBondSlashPercentage = AdvisoryBondSlashPercentage;
    type ApproveOrigin = EnsureSignedBy<Sudo, AccountIdTest>;
    type AssetCreator = AssetRouter;
    type AssetDestroyer = AssetRouter;
    #[cfg(feature = "parachain")]
    type AssetRegistry = MockRegistry;
    type Authorized = Authorized;
    type CloseEarlyBlockPeriod = CloseEarlyBlockPeriod;
    type CloseEarlyDisputeBond = CloseEarlyDisputeBond;
    type CloseEarlyTimeFramePeriod = CloseEarlyTimeFramePeriod;
    type CloseEarlyProtectionBlockPeriod = CloseEarlyProtectionBlockPeriod;
    type CloseEarlyProtectionTimeFramePeriod = CloseEarlyProtectionTimeFramePeriod;
    type CloseEarlyRequestBond = CloseEarlyRequestBond;
    type CloseMarketEarlyOrigin = EnsureSignedBy<Sudo, AccountIdTest>;
    type CloseOrigin = EnsureSignedBy<Sudo, AccountIdTest>;
    type Court = Court;
    type Currency = Balances;
    type DeployPool = NeoSwaps;
    type DisputeBond = DisputeBond;
    type RuntimeEvent = RuntimeEvent;
    type GlobalDisputes = GlobalDisputes;
    type LiquidityMining = LiquidityMining;
    type MaxCategories = MaxCategories;
    type MaxDisputes = MaxDisputes;
    type MinDisputeDuration = MinDisputeDuration;
    type MinOracleDuration = MinOracleDuration;
    type MaxCreatorFee = MaxCreatorFee;
    type MaxDisputeDuration = MaxDisputeDuration;
    type MaxGracePeriod = MaxGracePeriod;
    type MaxOracleDuration = MaxOracleDuration;
    type MaxMarketLifetime = MaxMarketLifetime;
    type MinCategories = MinCategories;
    type MaxEditReasonLen = MaxEditReasonLen;
    type MaxRejectReasonLen = MaxRejectReasonLen;
    type OnStateTransition = ();
    type OracleBond = OracleBond;
    type OutsiderBond = OutsiderBond;
    type PalletId = PmPalletId;
    type RejectOrigin = EnsureSignedBy<Sudo, AccountIdTest>;
    type RequestEditOrigin = EnsureSignedBy<Sudo, AccountIdTest>;
    type ResolveOrigin = EnsureSignedBy<Sudo, AccountIdTest>;
    type AssetManager = AssetManager;
    type SimpleDisputes = SimpleDisputes;
    type Slash = Treasury;
    type ValidityBond = ValidityBond;
    type WeightInfo = zrml_prediction_markets::weights::WeightInfo<Runtime>;
}

impl zrml_authorized::Config for Runtime {
    type AuthorizedDisputeResolutionOrigin =
        EnsureSignedBy<AuthorizedDisputeResolutionUser, AccountIdTest>;
    type CorrectionPeriod = CorrectionPeriod;
    type Currency = Balances;
    type RuntimeEvent = RuntimeEvent;
    type DisputeResolution = zrml_prediction_markets::Pallet<Runtime>;
    type MarketCommons = MarketCommons;
    type PalletId = AuthorizedPalletId;
    type WeightInfo = zrml_authorized::weights::WeightInfo<Runtime>;
}

impl zrml_court::Config for Runtime {
    type AppealBond = AppealBond;
    type BlocksPerYear = BlocksPerYear;
    type DisputeResolution = zrml_prediction_markets::Pallet<Runtime>;
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

impl zrml_liquidity_mining::Config for Runtime {
    type Currency = Balances;
    type RuntimeEvent = RuntimeEvent;
    type MarketCommons = MarketCommons;
    type MarketId = MarketId;
    type PalletId = LiquidityMiningPalletId;
    type WeightInfo = zrml_liquidity_mining::weights::WeightInfo<Runtime>;
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
    type MultiCurrency = AssetRouter;
    type NativeCurrency = BasicCurrencyAdapter<Runtime, Balances>;
    type WeightInfo = ();
}

impl orml_tokens::Config for Runtime {
    type Amount = Amount;
    type Balance = Balance;
    type CurrencyId = Currencies;
    type DustRemovalWhitelist = DustRemovalWhitelist;
    type RuntimeEvent = RuntimeEvent;
    type ExistentialDeposits = ExistentialDeposits;
    type MaxLocks = MaxLocks;
    type MaxReserves = MaxReserves;
    type CurrencyHooks = ();
    type ReserveIdentifier = [u8; 8];
    type WeightInfo = ();
}

impl pallet_balances::Config for Runtime {
    type AccountStore = System;
    type Balance = Balance;
    type DustRemoval = ();
    type RuntimeEvent = RuntimeEvent;
    type ExistentialDeposit = ExistentialDeposit;
    type MaxLocks = MaxLocks;
    type MaxReserves = MaxReserves;
    type ReserveIdentifier = [u8; 8];
    type WeightInfo = ();
}

impl zrml_market_commons::Config for Runtime {
    type Balance = Balance;
    type MarketId = MarketId;
    type Timestamp = Timestamp;
}

impl pallet_timestamp::Config for Runtime {
    type MinimumPeriod = MinimumPeriod;
    type Moment = Moment;
    type OnTimestampSet = ();
    type WeightInfo = ();
}

impl zrml_simple_disputes::Config for Runtime {
    type Currency = Balances;
    type RuntimeEvent = RuntimeEvent;
    type OutcomeBond = OutcomeBond;
    type OutcomeFactor = OutcomeFactor;
    type DisputeResolution = zrml_prediction_markets::Pallet<Runtime>;
    type MarketCommons = MarketCommons;
    type MaxDisputes = MaxDisputes;
    type PalletId = SimpleDisputesPalletId;
    type WeightInfo = zrml_simple_disputes::weights::WeightInfo<Runtime>;
}

impl zrml_global_disputes::Config for Runtime {
    type AddOutcomePeriod = AddOutcomePeriod;
    type RuntimeEvent = RuntimeEvent;
    type DisputeResolution = zrml_prediction_markets::Pallet<Runtime>;
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

#[cfg(feature = "parachain")]
zrml_prediction_markets::impl_mock_registry! {
    MockRegistry,
    zeitgeist_primitives::types::XcmAsset,
    Balance,
    zeitgeist_primitives::types::CustomMetadata
}

#[allow(unused)]
pub struct ExtBuilder {
    balances: Vec<(AccountIdTest, Balance)>,
}

// TODO(#1222): Remove this in favor of adding whatever the account need in the individual tests.
#[allow(unused)]
impl Default for ExtBuilder {
    fn default() -> Self {
        Self {
            balances: vec![
                (ALICE, INITIAL_BALANCE),
                (CHARLIE, INITIAL_BALANCE),
                (DAVE, INITIAL_BALANCE),
                (EVE, INITIAL_BALANCE),
                (FEE_ACCOUNT, INITIAL_BALANCE),
            ],
        }
    }
}

#[allow(unused)]
impl ExtBuilder {
    pub fn build(self) -> sp_io::TestExternalities {
        let mut t = frame_system::GenesisConfig::default().build_storage::<Runtime>().unwrap();
        // see the logs in tests when using `RUST_LOG=debug cargo test -- --nocapture`
        let _ = env_logger::builder().is_test(true).try_init();
        pallet_balances::GenesisConfig::<Runtime> { balances: self.balances }
            .assimilate_storage(&mut t)
            .unwrap();
        #[cfg(feature = "parachain")]
        {
            use frame_support::traits::GenesisBuild;
            orml_tokens::GenesisConfig::<Runtime> {
                balances: vec![
                    (ALICE, FOREIGN_ASSET.try_into().unwrap(), INITIAL_BALANCE),
                    (FEE_ACCOUNT, FOREIGN_ASSET.try_into().unwrap(), INITIAL_BALANCE),
                ],
            }
            .assimilate_storage(&mut t)
            .unwrap();
            let custom_metadata = zeitgeist_primitives::types::CustomMetadata {
                allow_as_base_asset: true,
                ..Default::default()
            };
            orml_asset_registry_mock::GenesisConfig {
                metadata: vec![(
                    FOREIGN_ASSET.try_into().unwrap(),
                    AssetMetadata {
                        decimals: 18,
                        name: "MKL".as_bytes().to_vec(),
                        symbol: "MKL".as_bytes().to_vec(),
                        existential_deposit: 0,
                        location: None,
                        additional: custom_metadata,
                    },
                )],
            }
            .assimilate_storage(&mut t)
            .unwrap();
        }
        let mut test_ext: sp_io::TestExternalities = t.into();
        test_ext.execute_with(|| System::set_block_number(1));
        test_ext
    }
}
