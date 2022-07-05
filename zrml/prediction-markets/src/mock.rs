#![allow(
    // Mocks are only used for fuzzing and unit tests
    clippy::integer_arithmetic
)]
#![cfg(feature = "mock")]

use crate as prediction_markets;
use frame_support::{
    construct_runtime, ord_parameter_types, parameter_types,
    traits::{Everything, OnFinalize, OnInitialize},
    PalletId,
};
use frame_system::EnsureSignedBy;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
};
use substrate_fixed::{types::extra::U33, FixedI128, FixedU128};
use zeitgeist_primitives::{
    constants::{
        AuthorizedPalletId, BalanceFractionalDecimals, BlockHashCount, CourtCaseDuration,
        CourtPalletId, DisputeFactor, ExistentialDeposit, ExistentialDeposits, ExitFee,
        GetNativeCurrencyId, GlobalDisputesPalletId, LiquidityMiningPalletId, LockPeriod,
        MaxAssets, MaxCategories, MaxDisputeLocks, MaxDisputes, MaxInRatio, MaxMarketPeriod,
        MaxOutRatio, MaxReserves, MaxSubsidyPeriod, MaxTotalWeight, MaxWeight, MinAssets,
        MinCategories, MinLiquidity, MinSubsidy, MinSubsidyPeriod, MinWeight, MinimumPeriod,
        PmPalletId, ReportingPeriod, SimpleDisputesPalletId, StakeWeight, SwapsPalletId,
        VoteLockIdentifier, BASE, CENT,
    },
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

ord_parameter_types! {
    pub const Sudo: AccountIdTest = SUDO;
}
parameter_types! {
    pub const DisputePeriod: BlockNumber = 10;
    pub const TreasuryPalletId: PalletId = PalletId(*b"3.141592");
    pub const MinSubsidyPerAccount: Balance = BASE;
    pub const AdvisoryBond: Balance = 11 * CENT;
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
        Currency: orml_currencies::{Call, Pallet, Storage},
        LiquidityMining: zrml_liquidity_mining::{Config<T>, Event<T>, Pallet},
        MarketCommons: zrml_market_commons::{Pallet, Storage},
        PredictionMarkets: prediction_markets::{Event<T>, Pallet, Storage},
        RandomnessCollectiveFlip: pallet_randomness_collective_flip::{Pallet, Storage},
        RikiddoSigmoidFeeMarketEma: zrml_rikiddo::{Pallet, Storage},
        SimpleDisputes: zrml_simple_disputes::{Event<T>, Pallet, Storage},
        GlobalDisputes: zrml_global_disputes::{Event<T>, Pallet, Storage},
        Swaps: zrml_swaps::{Call, Event<T>, Pallet},
        System: frame_system::{Config, Event<T>, Pallet, Storage},
        Timestamp: pallet_timestamp::{Pallet},
        Tokens: orml_tokens::{Config<T>, Event<T>, Pallet, Storage},
    }
);

impl crate::Config for Runtime {
    type AdvisoryBond = AdvisoryBond;
    type ApprovalOrigin = EnsureSignedBy<Sudo, AccountIdTest>;
    type Authorized = Authorized;
    type CloseOrigin = EnsureSignedBy<Sudo, AccountIdTest>;
    type Court = Court;
    type DestroyOrigin = EnsureSignedBy<Sudo, AccountIdTest>;
    type DisputeBond = DisputeBond;
    type DisputeFactor = DisputeFactor;
    type DisputePeriod = DisputePeriod;
    type Event = Event;
    type LiquidityMining = LiquidityMining;
    type MarketCommons = MarketCommons;
    type MaxCategories = MaxCategories;
    type MaxDisputes = MaxDisputes;
    type MaxSubsidyPeriod = MaxSubsidyPeriod;
    type MaxMarketPeriod = MaxMarketPeriod;
    type MinCategories = MinCategories;
    type MinSubsidyPeriod = MinSubsidyPeriod;
    type OracleBond = OracleBond;
    type PalletId = PmPalletId;
    type ResolveOrigin = EnsureSignedBy<Sudo, AccountIdTest>;
    type ReportingPeriod = ReportingPeriod;
    type Shares = Tokens;
    type SimpleDisputes = SimpleDisputes;
    type GlobalDisputes = GlobalDisputes;
    type Slash = ();
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
    type Call = Call;
    type DbWeight = ();
    type Event = Event;
    type Hash = Hash;
    type Hashing = BlakeTwo256;
    type Header = Header;
    type Index = Index;
    type Lookup = IdentityLookup<Self::AccountId>;
    type MaxConsumers = frame_support::traits::ConstU32<16>;
    type OnKilledAccount = ();
    type OnNewAccount = ();
    type OnSetCode = ();
    type Origin = Origin;
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
    type Event = Event;
    type ExistentialDeposits = ExistentialDeposits;
    type MaxLocks = ();
    type MaxReserves = MaxReserves;
    type OnDust = ();
    type ReserveIdentifier = [u8; 8];
    type WeightInfo = ();
}

impl pallet_balances::Config for Runtime {
    type AccountStore = System;
    type Balance = Balance;
    type DustRemoval = ();
    type Event = Event;
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

impl zrml_authorized::Config for Runtime {
    type Event = Event;
    type MarketCommons = MarketCommons;
    type PalletId = AuthorizedPalletId;
    type WeightInfo = zrml_authorized::weights::WeightInfo<Runtime>;
}

impl zrml_court::Config for Runtime {
    type CourtCaseDuration = CourtCaseDuration;
    type Event = Event;
    type MarketCommons = MarketCommons;
    type PalletId = CourtPalletId;
    type Random = RandomnessCollectiveFlip;
    type StakeWeight = StakeWeight;
    type TreasuryPalletId = TreasuryPalletId;
    type WeightInfo = zrml_court::weights::WeightInfo<Runtime>;
}

impl zrml_liquidity_mining::Config for Runtime {
    type Event = Event;
    type MarketCommons = MarketCommons;
    type MarketId = MarketId;
    type PalletId = LiquidityMiningPalletId;
    type WeightInfo = zrml_liquidity_mining::weights::WeightInfo<Runtime>;
}

impl zrml_market_commons::Config for Runtime {
    type Currency = Balances;
    type MarketId = MarketId;
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
    type Event = Event;
    type MarketCommons = MarketCommons;
    type PalletId = SimpleDisputesPalletId;
}

impl zrml_global_disputes::Config for Runtime {
    type Event = Event;
    type MarketCommons = MarketCommons;
    type PalletId = GlobalDisputesPalletId;
    type VoteLockIdentifier = VoteLockIdentifier;
    type MaxDisputeLocks = MaxDisputeLocks;
}

impl zrml_swaps::Config for Runtime {
    type Event = Event;
    type ExitFee = ExitFee;
    type FixedTypeU = <Runtime as zrml_rikiddo::Config>::FixedTypeU;
    type FixedTypeS = <Runtime as zrml_rikiddo::Config>::FixedTypeS;
    type LiquidityMining = LiquidityMining;
    type MarketCommons = MarketCommons;
    type MarketId = MarketId;
    type MaxAssets = MaxAssets;
    type MaxInRatio = MaxInRatio;
    type MaxOutRatio = MaxOutRatio;
    type MaxTotalWeight = MaxTotalWeight;
    type MaxWeight = MaxWeight;
    type MinAssets = MinAssets;
    type MinLiquidity = MinLiquidity;
    type MinSubsidy = MinSubsidy;
    type MinSubsidyPerAccount = MinSubsidyPerAccount;
    type MinWeight = MinWeight;
    type PalletId = SwapsPalletId;
    type RikiddoSigmoidFeeMarketEma = RikiddoSigmoidFeeMarketEma;
    type Shares = Currency;
    type WeightInfo = zrml_swaps::weights::WeightInfo<Runtime>;
}

pub struct ExtBuilder {
    balances: Vec<(AccountIdTest, Balance)>,
}

impl Default for ExtBuilder {
    fn default() -> Self {
        Self {
            balances: vec![
                (ALICE, 1_000 * BASE),
                (BOB, 1_000 * BASE),
                (CHARLIE, 1_000 * BASE),
                (DAVE, 1_000 * BASE),
                (EVE, 1_000 * BASE),
                (FRED, 1_000 * BASE),
                (SUDO, 1_000 * BASE),
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

        t.into()
    }
}

pub fn run_to_block(n: BlockNumber) {
    while System::block_number() < n {
        Balances::on_finalize(System::block_number());
        PredictionMarkets::on_finalize(System::block_number());
        System::on_finalize(System::block_number());
        System::set_block_number(System::block_number() + 1);
        System::on_initialize(System::block_number());
        PredictionMarkets::on_initialize(System::block_number());
        Balances::on_initialize(System::block_number());
    }
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
