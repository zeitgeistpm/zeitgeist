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
    traits::{
        tokens::{PayFromAccount, UnityAssetBalanceConversion},
        Contains, Everything, NeverEnsureOrigin,
    },
    Blake2_256,
};
use frame_system::{mocking::MockBlock, EnsureRoot, EnsureSignedBy};
use orml_traits::MultiCurrency;
#[cfg(feature = "runtime-benchmarks")]
use pallet_treasury::ArgumentsFactory;
#[cfg(feature = "runtime-benchmarks")]
use sp_core::{H160, H256};
use sp_runtime::{
    traits::{BlakeTwo256, ConstU32, Get, IdentityLookup, Zero},
    BuildStorage, Perbill, Percent, SaturatedConversion,
};
use zeitgeist_primitives::{
    constants::mock::{
        AddOutcomePeriod, AggregationPeriod, AppealBond, AppealPeriod, AuthorizedPalletId,
        BlockHashCount, BlocksPerYear, CloseEarlyBlockPeriod, CloseEarlyDisputeBond,
        CloseEarlyProtectionBlockPeriod, CloseEarlyProtectionTimeFramePeriod,
        CloseEarlyRequestBond, CloseEarlyTimeFramePeriod, CombinatorialTokensPalletId,
        CorrectionPeriod, CourtPalletId, ExistentialDeposit, ExistentialDeposits, GdVotingPeriod,
        GetNativeCurrencyId, GlobalDisputeLockId, GlobalDisputesPalletId, HybridRouterPalletId,
        InflationPeriod, LockId, MaxAppeals, MaxApprovals, MaxCourtParticipants, MaxCreatorFee,
        MaxDelegations, MaxDisputeDuration, MaxDisputes, MaxEditReasonLen, MaxGlobalDisputeVotes,
        MaxGracePeriod, MaxLiquidityTreeDepth, MaxLocks, MaxMarketLifetime, MaxOracleDuration,
        MaxOrders, MaxOwners, MaxRejectReasonLen, MaxReserves, MaxSelectedDraws,
        MaxYearlyInflation, MinCategories, MinDisputeDuration, MinJurorStake, MinOracleDuration,
        MinOutcomeVoteAmount, MinimumPeriod, NeoMaxSwapFee, NeoSwapsPalletId, OrderbookPalletId,
        OutsiderBond, PmPalletId, RemoveKeysLimit, RequestInterval, TreasuryPalletId, VotePeriod,
        VotingOutcomeFee, BASE, CENT, MAX_ASSETS,
    },
    traits::DistributeFees,
    types::{
        AccountIdTest, Amount, Balance, BasicCurrencyAdapter, CombinatorialId, CurrencyId, Hash,
        MarketId, Moment,
    },
};
use zrml_combinatorial_tokens::types::{CryptographicIdManager, Fuel};

#[cfg(feature = "parachain")]
use {
    orml_traits::asset_registry::AssetProcessor, parity_scale_codec::Encode,
    sp_runtime::DispatchError, zeitgeist_primitives::types::Asset,
    zeitgeist_primitives::types::CustomMetadata,
};

#[cfg(feature = "runtime-benchmarks")]
use zeitgeist_primitives::types::NoopCombinatorialTokensBenchmarkHelper;

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
#[allow(unused)]
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
    pub storage MaxSplits: u16 = 128;
}
parameter_types! {
    pub const AdvisoryBond: Balance = 0;
    pub const AdvisoryBondSlashPercentage: Percent = Percent::from_percent(10);
    pub const OracleBond: Balance = 0;
    pub const ValidityBond: Balance = 0;
    pub const DisputeBond: Balance = 0;
    pub const MaxCategories: u16 = MAX_ASSETS + 1;
    pub TreasuryAccount: AccountIdTest = Treasury::account_id();
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

pub struct DustRemovalWhitelist;

impl Contains<AccountIdTest> for DustRemovalWhitelist {
    fn contains(account_id: &AccountIdTest) -> bool {
        *account_id == FEE_ACCOUNT
    }
}

construct_runtime!(
    pub enum Runtime {
        HybridRouter: zrml_hybrid_router,
        Orderbook: zrml_orderbook,
        NeoSwaps: zrml_neo_swaps,
        #[cfg(feature = "parachain")]
        AssetRegistry: orml_asset_registry::module,
        Authorized: zrml_authorized,
        Balances: pallet_balances,
        CombinatorialTokens: zrml_combinatorial_tokens,
        Court: zrml_court,
        AssetManager: orml_currencies,
        MarketCommons: zrml_market_commons,
        PredictionMarkets: zrml_prediction_markets,
        RandomnessCollectiveFlip: pallet_insecure_randomness_collective_flip,
        GlobalDisputes: zrml_global_disputes,
        System: frame_system,
        Timestamp: pallet_timestamp,
        Tokens: orml_tokens,
        Treasury: pallet_treasury,
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

impl zrml_orderbook::Config for Runtime {
    type AssetManager = AssetManager;
    type ExternalFees = ExternalFees<Runtime, FeeAccount>;
    type RuntimeEvent = RuntimeEvent;
    type MarketCommons = MarketCommons;
    type PalletId = OrderbookPalletId;
    type WeightInfo = zrml_orderbook::weights::WeightInfo<Runtime>;
}

impl zrml_neo_swaps::Config for Runtime {
    type CombinatorialId = CombinatorialId;
    type CombinatorialTokens = CombinatorialTokens;
    type CombinatorialTokensUnsafe = CombinatorialTokens;
    type CompleteSetOperations = PredictionMarkets;
    type ExternalFees = ExternalFees<Runtime, FeeAccount>;
    type MarketCommons = MarketCommons;
    type MultiCurrency = AssetManager;
    type PoolId = MarketId;
    type RuntimeEvent = RuntimeEvent;
    type MaxLiquidityTreeDepth = MaxLiquidityTreeDepth;
    type MaxSplits = MaxSplits;
    type MaxSwapFee = NeoMaxSwapFee;
    type PalletId = NeoSwapsPalletId;
    type WeightInfo = zrml_neo_swaps::weights::WeightInfo<Runtime>;
}

impl pallet_insecure_randomness_collective_flip::Config for Runtime {}

impl zrml_prediction_markets::Config for Runtime {
    type AdvisoryBond = AdvisoryBond;
    type AdvisoryBondSlashPercentage = AdvisoryBondSlashPercentage;
    type ApproveOrigin = EnsureSignedBy<Sudo, AccountIdTest>;
    #[cfg(feature = "parachain")]
    type AssetRegistry = AssetRegistry;
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
    type OracleBond = OracleBond;
    type OutsiderBond = OutsiderBond;
    type PalletId = PmPalletId;
    type RejectOrigin = EnsureSignedBy<Sudo, AccountIdTest>;
    type RequestEditOrigin = EnsureSignedBy<Sudo, AccountIdTest>;
    type ResolveOrigin = EnsureSignedBy<Sudo, AccountIdTest>;
    type AssetManager = AssetManager;
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

impl zrml_combinatorial_tokens::Config for Runtime {
    #[cfg(feature = "runtime-benchmarks")]
    type BenchmarkHelper = NoopCombinatorialTokensBenchmarkHelper<Balance, MarketId>;
    type CombinatorialIdManager = CryptographicIdManager<MarketId, Blake2_256>;
    type Fuel = Fuel;
    type MarketCommons = MarketCommons;
    type MultiCurrency = AssetManager;
    type Payout = PredictionMarkets;
    type RuntimeEvent = RuntimeEvent;
    type PalletId = CombinatorialTokensPalletId;
    type WeightInfo = zrml_combinatorial_tokens::weights::WeightInfo<Runtime>;
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
        let mut t = frame_system::GenesisConfig::<Runtime>::default().build_storage().unwrap();
        // see the logs in tests when using `RUST_LOG=debug cargo test -- --nocapture`
        let _ = env_logger::builder().is_test(true).try_init();
        pallet_balances::GenesisConfig::<Runtime> { balances: self.balances }
            .assimilate_storage(&mut t)
            .unwrap();
        #[cfg(feature = "parachain")]
        {
            orml_tokens::GenesisConfig::<Runtime> {
                balances: vec![
                    (ALICE, FOREIGN_ASSET, INITIAL_BALANCE),
                    (FEE_ACCOUNT, FOREIGN_ASSET, INITIAL_BALANCE),
                ],
            }
            .assimilate_storage(&mut t)
            .unwrap();
            let custom_metadata = zeitgeist_primitives::types::CustomMetadata {
                allow_as_base_asset: true,
                ..Default::default()
            };
            orml_asset_registry::module::GenesisConfig::<Runtime> {
                assets: vec![(
                    FOREIGN_ASSET,
                    AssetMetadata {
                        decimals: 18,
                        name: "MKL".as_bytes().to_vec().try_into().unwrap(),
                        symbol: "MKL".as_bytes().to_vec().try_into().unwrap(),
                        existential_deposit: 0,
                        location: None,
                        additional: custom_metadata,
                    }
                    .encode(),
                )],
                last_asset_id: FOREIGN_ASSET,
            }
            .assimilate_storage(&mut t)
            .unwrap();
        }
        let mut test_ext: sp_io::TestExternalities = t.into();
        test_ext.execute_with(|| System::set_block_number(1));
        test_ext
    }
}
