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

#![cfg(feature = "mock")]

use crate as orderbook;
use crate::{AssetOf, BalanceOf, MarketIdOf};
use core::marker::PhantomData;
use frame_support::{
    construct_runtime,
    pallet_prelude::Get,
    parameter_types,
    traits::{AsEnsureOriginWithArg, Everything},
};
use frame_system::{mocking::MockBlock, EnsureRoot, EnsureSigned};
use orml_traits::MultiCurrency;
use parity_scale_codec::Compact;
use sp_runtime::{
    traits::{BlakeTwo256, ConstU32, IdentityLookup, Zero},
    BuildStorage, Perbill, SaturatedConversion,
};
use zeitgeist_primitives::{
    constants::mock::{
        AssetsAccountDeposit, AssetsApprovalDeposit, AssetsDeposit, AssetsMetadataDepositBase,
        AssetsMetadataDepositPerByte, AssetsStringLimit, BlockHashCount, DestroyAccountWeight,
        DestroyApprovalWeight, DestroyFinishWeight, ExistentialDeposit, ExistentialDeposits,
        GetNativeCurrencyId, MaxLocks, MaxReserves, MinimumPeriod, OrderbookPalletId, BASE, CENT,
    },
    traits::DistributeFees,
    types::{
        AccountIdTest, Amount, Assets, Balance, BasicCurrencyAdapter, CampaignAsset,
        CampaignAssetId, Currencies, CustomAsset, CustomAssetId, Hash, MarketAsset, MarketId,
        Moment,
    },
};

pub const ALICE: AccountIdTest = 0;
pub const BOB: AccountIdTest = 1;
pub const MARKET_CREATOR: AccountIdTest = 42;
pub const INITIAL_BALANCE: Balance = 100 * BASE;
pub const EXTERNAL_FEES: Balance = CENT;

parameter_types! {
    pub const FeeAccount: AccountIdTest = MARKET_CREATOR;
}

type CustomAssetsInstance = pallet_assets::Instance1;
type CampaignAssetsInstance = pallet_assets::Instance2;
type MarketAssetsInstance = pallet_assets::Instance3;

pub fn fee_percentage<T: crate::Config>() -> Perbill {
    Perbill::from_rational(EXTERNAL_FEES, BASE)
}

pub fn calculate_fee<T: crate::Config>(amount: BalanceOf<T>) -> BalanceOf<T> {
    fee_percentage::<T>().mul_floor(amount.saturated_into::<BalanceOf<T>>())
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
        fee_percentage::<T>()
    }
}

construct_runtime!(
    pub enum Runtime {
        AssetManager: orml_currencies,
        AssetRouter: zrml_asset_router,
        Balances: pallet_balances,
        CampaignAssets: pallet_assets::<Instance2>,
        CustomAssets: pallet_assets::<Instance1>,
        MarketAssets: pallet_assets::<Instance3>,
        MarketCommons: zrml_market_commons,
        Orderbook: orderbook,
        System: frame_system,
        Timestamp: pallet_timestamp,
        Tokens: orml_tokens,
    }
);

impl crate::Config for Runtime {
    type AssetManager = AssetManager;
    type ExternalFees = ExternalFees<Runtime, FeeAccount>;
    type RuntimeEvent = RuntimeEvent;
    type MarketCommons = MarketCommons;
    type PalletId = OrderbookPalletId;
    type WeightInfo = orderbook::weights::WeightInfo<Runtime>;
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

impl pallet_timestamp::Config for Runtime {
    type MinimumPeriod = MinimumPeriod;
    type Moment = Moment;
    type OnTimestampSet = ();
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

impl zrml_market_commons::Config for Runtime {
    type Balance = Balance;
    type MarketId = MarketId;
    type Timestamp = Timestamp;
}

pub struct ExtBuilder {
    balances: Vec<(AccountIdTest, Balance)>,
}

impl Default for ExtBuilder {
    fn default() -> Self {
        Self { balances: vec![(ALICE, INITIAL_BALANCE), (BOB, INITIAL_BALANCE)] }
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

        let mut t: sp_io::TestExternalities = t.into();

        t.execute_with(|| System::set_block_number(1));

        t
    }
}
