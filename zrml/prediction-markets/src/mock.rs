// Copyright 2022-2024 Forecasting Technologies LTD.
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
    construct_runtime, ord_parameter_types,
    pallet_prelude::Weight,
    parameter_types,
    storage::unhashed::put,
    traits::{
        AsEnsureOriginWithArg, Everything, NeverEnsureOrigin, OnFinalize, OnIdle, OnInitialize,
    },
};
use frame_system::{mocking::MockBlock, EnsureRoot, EnsureSigned, EnsureSignedBy};
use parity_scale_codec::Compact;
use sp_arithmetic::per_things::Percent;
use sp_runtime::{
    traits::{BlakeTwo256, ConstU32, IdentityLookup},
    BuildStorage, DispatchError, DispatchResult,
};
use std::cell::RefCell;
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
        GlobalDisputesPalletId, InflationPeriod, LiquidityMiningPalletId, LockId, MaxAppeals,
        MaxApprovals, MaxCategories, MaxCourtParticipants, MaxCreatorFee, MaxDelegations,
        MaxDisputeDuration, MaxDisputes, MaxEditReasonLen, MaxGlobalDisputeVotes, MaxGracePeriod,
        MaxLocks, MaxMarketLifetime, MaxOracleDuration, MaxOwners, MaxRejectReasonLen, MaxReserves,
        MaxSelectedDraws, MaxYearlyInflation, MinCategories, MinDisputeDuration, MinJurorStake,
        MinOracleDuration, MinOutcomeVoteAmount, MinimumPeriod, OutcomeBond, OutcomeFactor,
        OutsiderBond, PmPalletId, RemoveKeysLimit, RequestInterval, SimpleDisputesPalletId,
        TreasuryPalletId, VotePeriod, VotingOutcomeFee, BASE, CENT, MILLISECS_PER_BLOCK,
    },
    traits::{DeployPoolApi, MarketTransitionApi},
    types::{
        AccountIdTest, Amount, Assets, Balance, BasicCurrencyAdapter, BlockNumber, BlockTest,
        CampaignAsset, CampaignAssetClass, CampaignAssetId, Currencies, CustomAsset, CustomAssetId,
        Hash, MarketAsset, MarketId, Moment, ResultWithWeightInfo,
    },
};

#[cfg(feature = "parachain")]
use {
    orml_traits::asset_registry::AssetProcessor, parity_scale_codec::Encode,
    zeitgeist_primitives::types::CustomMetadata, zeitgeist_primitives::types::XcmAsset,
};

pub(super) const ON_PROPOSAL_STORAGE: [u8; 4] = [0x09, 0x09, 0x00, 0x00];
pub(super) const ON_ACTIVATION_STORAGE: [u8; 4] = [0x09, 0x09, 0x00, 0x01];
pub(super) const ON_CLOSURE_STORAGE: [u8; 4] = [0x09, 0x09, 0x00, 0x02];
pub(super) const ON_REPORT_STORAGE: [u8; 4] = [0x09, 0x09, 0x00, 0x03];
pub(super) const ON_DISPUTE_STORAGE: [u8; 4] = [0x09, 0x09, 0x00, 0x04];
pub(super) const ON_RESOLUTION_STORAGE: [u8; 4] = [0x09, 0x09, 0x00, 0x05];

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

// It just writes true to specific memory locations depending on the hook that's invoked.
pub struct StateTransitionMock;

impl MarketTransitionApi<MarketId> for StateTransitionMock {
    fn on_proposal(_market_id: &MarketId) -> ResultWithWeightInfo<DispatchResult> {
        put(&ON_PROPOSAL_STORAGE, &true);
        ResultWithWeightInfo::new(Ok(()), Weight::zero())
    }
    fn on_activation(_market_id: &MarketId) -> ResultWithWeightInfo<DispatchResult> {
        put(&ON_ACTIVATION_STORAGE, &true);
        ResultWithWeightInfo::new(Ok(()), Weight::zero())
    }
    fn on_closure(_market_id: &MarketId) -> ResultWithWeightInfo<DispatchResult> {
        put(&ON_CLOSURE_STORAGE, &true);
        ResultWithWeightInfo::new(Ok(()), Weight::zero())
    }
    fn on_report(_market_id: &MarketId) -> ResultWithWeightInfo<DispatchResult> {
        put(&ON_REPORT_STORAGE, &true);
        ResultWithWeightInfo::new(Ok(()), Weight::zero())
    }
    fn on_dispute(_market_id: &MarketId) -> ResultWithWeightInfo<DispatchResult> {
        put(&ON_DISPUTE_STORAGE, &true);
        ResultWithWeightInfo::new(Ok(()), Weight::zero())
    }
    fn on_resolution(_market_id: &MarketId) -> ResultWithWeightInfo<DispatchResult> {
        put(&ON_RESOLUTION_STORAGE, &true);
        ResultWithWeightInfo::new(Ok(()), Weight::zero())
    }
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
}

type CustomAssetsInstance = pallet_assets::Instance1;
type CampaignAssetsInstance = pallet_assets::Instance2;
type MarketAssetsInstance = pallet_assets::Instance3;

construct_runtime!(
    pub enum Runtime {
        #[cfg(feature = "parachain")]
        AssetRegistry: orml_asset_registry,
        AssetRouter: zrml_asset_router,
        Authorized: zrml_authorized,
        Balances: pallet_balances,
        CampaignAssets: pallet_assets::<Instance2>,
        CustomAssets: pallet_assets::<Instance1>,
        Court: zrml_court,
        AssetManager: orml_currencies,
        LiquidityMining: zrml_liquidity_mining,
        MarketAssets: pallet_assets::<Instance3>,
        MarketCommons: zrml_market_commons,
        PredictionMarkets: prediction_markets,
        RandomnessCollectiveFlip: pallet_insecure_randomness_collective_flip,
        SimpleDisputes: zrml_simple_disputes,
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
    type AssetCreator = AssetRouter;
    type AssetDestroyer = AssetRouter;
    type AssetManager = AssetManager;
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
    type LiquidityMining = LiquidityMining;
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
    type OnStateTransition = (StateTransitionMock,);
    type OracleBond = OracleBond;
    type OutsiderBond = OutsiderBond;
    type PalletId = PmPalletId;
    type CloseEarlyBlockPeriod = CloseEarlyBlockPeriod;
    type CloseEarlyTimeFramePeriod = CloseEarlyTimeFramePeriod;
    type RejectOrigin = EnsureSignedBy<RejectOrigin, AccountIdTest>;
    type RequestEditOrigin = EnsureSignedBy<RequestEditOrigin, AccountIdTest>;
    type ResolveOrigin = EnsureSignedBy<ResolveOrigin, AccountIdTest>;
    type SimpleDisputes = SimpleDisputes;
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
    type DbWeight = ();
    type RuntimeEvent = RuntimeEvent;
    type Hash = Hash;
    type Hashing = BlakeTwo256;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Nonce = u64;
    type MaxConsumers = frame_support::traits::ConstU32<16>;
    type OnKilledAccount = ();
    type OnNewAccount = ();
    type RuntimeOrigin = RuntimeOrigin;
    type PalletInfo = PalletInfo;
    type SS58Prefix = ();
    type SystemWeightInfo = ();
    type Version = ();
    type OnSetCode = ();
}

impl orml_currencies::Config for Runtime {
    type GetNativeCurrencyId = GetNativeCurrencyId;
    type MultiCurrency = AssetRouter;
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

        impl AssetProcessor<XcmAsset, AssetMetadata> for NoopAssetProcessor {
            fn pre_register(id: Option<XcmAsset>, asset_metadata: AssetMetadata)
             -> Result<(XcmAsset, AssetMetadata), DispatchError> {
                Ok((id.unwrap(), asset_metadata))
            }
        }

        impl orml_asset_registry::Config for Runtime {
            type RuntimeEvent = RuntimeEvent;
            type CustomMetadata = CustomMetadata;
            type AssetId = XcmAsset;
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
    type CurrencyId = Currencies;
    type DustRemovalWhitelist = Everything;
    type RuntimeEvent = RuntimeEvent;
    type ExistentialDeposits = ExistentialDeposits;
    type MaxLocks = ();
    type MaxReserves = MaxReserves;
    type CurrencyHooks = ();
    type ReserveIdentifier = [u8; 8];
    type WeightInfo = ();
}

impl pallet_balances::Config for Runtime {
    type AccountStore = System;
    type Balance = Balance;
    type DustRemoval = ();
    type FreezeIdentifier = ();
    type RuntimeHoldReason = ();
    type RuntimeEvent = RuntimeEvent;
    type ExistentialDeposit = ExistentialDeposit;
    type MaxHolds = ();
    type MaxFreezes = ();
    type MaxLocks = MaxLocks;
    type MaxReserves = MaxReserves;
    type ReserveIdentifier = [u8; 8];
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
    type Timestamp = Timestamp;
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
        let mut t = frame_system::GenesisConfig::<Runtime>::default().build_storage().unwrap();

        // see the logs in tests when using `RUST_LOG=debug cargo test -- --nocapture`
        let _ = env_logger::builder().is_test(true).try_init();

        pallet_assets::GenesisConfig::<Runtime, CampaignAssetsInstance> {
            assets: vec![(CampaignAssetClass(100), ALICE, true, 1)],
            metadata: vec![],
            accounts: self
                .balances
                .iter()
                .map(|(account, balance)| (CampaignAssetClass(100), *account, *balance))
                .collect(),
        }
        .assimilate_storage(&mut t)
        .unwrap();

        pallet_balances::GenesisConfig::<Runtime> { balances: self.balances }
            .assimilate_storage(&mut t)
            .unwrap();

        #[cfg(feature = "parachain")]
        {
            orml_tokens::GenesisConfig::<Runtime> {
                balances: (0..69)
                    .map(|idx| (idx, Currencies::ForeignAsset(100), INITIAL_BALANCE))
                    .collect(),
            }
            .assimilate_storage(&mut t)
            .unwrap();

            let custom_metadata = zeitgeist_primitives::types::CustomMetadata {
                allow_as_base_asset: true,
                ..Default::default()
            };

            orml_asset_registry::GenesisConfig::<Runtime> {
                assets: vec![
                    (
                        XcmAsset::ForeignAsset(100),
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
                        XcmAsset::ForeignAsset(420),
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
                last_asset_id: XcmAsset::ForeignAsset(420),
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
        AssetRouter::on_idle(System::block_number(), Weight::MAX);
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
        fn market_outcome_share_id(_: MarketId, _: u16) -> Assets {
            Assets::PoolShare(1)
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
