// Copyright 2023-2024 Forecasting Technologies LTD.
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

#![cfg(test)]

extern crate alloc;

use crate::{self as zrml_asset_router};
use alloc::{collections::BTreeMap, vec, vec::Vec};
use frame_support::{
    construct_runtime,
    pallet_prelude::{DispatchResult, Weight},
    traits::{AsEnsureOriginWithArg, Everything},
};
use frame_system::EnsureSigned;
use orml_traits::parameter_type_with_key;
use pallet_assets::ManagedDestroy;
use parity_scale_codec::Compact;
use sp_runtime::{
    testing::Header,
    traits::{parameter_types, BlakeTwo256, ConstU128, ConstU32, IdentityLookup},
};
use zeitgeist_primitives::{
    constants::mock::{BlockHashCount, ExistentialDeposit, MaxLocks, MaxReserves, BASE},
    types::{
        AccountIdTest, Amount, Assets, Balance, BlockNumber, BlockTest, CampaignAsset,
        CampaignAssetClass, CampaignAssetId, Currencies, CustomAsset, CustomAssetClass,
        CustomAssetId, Hash, Index, MarketAsset, UncheckedExtrinsicTest,
    },
};

pub(super) const ALICE: AccountIdTest = 0;
pub(super) const BOB: AccountIdTest = 1;
pub(super) const CHARLIE: AccountIdTest = 2;

pub(super) const CAMPAIGN_ASSET: Assets = Assets::CampaignAsset(0);
pub(super) const CAMPAIGN_ASSET_INTERNAL: CampaignAssetClass = CampaignAssetClass(0);
pub(super) const CUSTOM_ASSET: Assets = Assets::CustomAsset(0);
pub(super) const CUSTOM_ASSET_INTERNAL: CustomAssetClass = CustomAssetClass(0);
pub(super) const MARKET_ASSET: Assets = Assets::CategoricalOutcome(7, 8);
pub(super) const MARKET_ASSET_INTERNAL: MarketAsset = MarketAsset::CategoricalOutcome(7, 8);
pub(super) const CURRENCY: Assets = Assets::ForeignAsset(0);
pub(super) const CURRENCY_OLD_OUTCOME: Assets = Assets::CategoricalOutcome(7, 8);
pub(super) const CURRENCY_INTERNAL: Currencies = Currencies::ForeignAsset(0);

pub(super) const CAMPAIGN_ASSET_MIN_BALANCE: Balance = 2;
pub(super) const CUSTOM_ASSET_MIN_BALANCE: Balance = 3;
pub(super) const MARKET_ASSET_MIN_BALANCE: Balance = 4;
pub(super) const CURRENCY_MIN_BALANCE: Balance = 5;

pub(super) const CAMPAIGN_ASSET_INITIAL_AMOUNT: Balance = 10;
pub(super) const CUSTOM_ASSET_INITIAL_AMOUNT: Balance = 20;
pub(super) const MARKET_ASSET_INITIAL_AMOUNT: Balance = 30;
pub(super) const CURRENCY_INITIAL_AMOUNT: Balance = 40;

pub(super) const DESTROY_WEIGHT: Weight = Weight::from_parts(1000, 0);

pub(super) type AccountId = <Runtime as frame_system::Config>::AccountId;
pub(super) type CustomAssetsInstance = pallet_assets::Instance1;
pub(super) type CampaignAssetsInstance = pallet_assets::Instance2;
pub(super) type MarketAssetsInstance = pallet_assets::Instance3;

parameter_types! {
    pub const DestroyWeight: Weight = DESTROY_WEIGHT;
}

construct_runtime!(
    pub enum Runtime
    where
        Block = BlockTest<Runtime>,
        NodeBlock = BlockTest<Runtime>,
        UncheckedExtrinsic = UncheckedExtrinsicTest<Runtime>,
    {
        AssetRouter: zrml_asset_router::{Pallet},
        Balances: pallet_balances::{Call, Config<T>, Event<T>, Pallet, Storage},
        CustomAssets: pallet_assets::<Instance1>::{Call, Pallet, Storage, Event<T>},
        CampaignAssets: pallet_assets::<Instance2>::{Call, Pallet, Storage, Event<T>},
        MarketAssets: pallet_assets::<Instance3>::{Call, Pallet, Storage, Event<T>},
        System: frame_system::{Call, Config, Event<T>, Pallet, Storage},
        Tokens: orml_tokens::{Config<T>, Event<T>, Pallet, Storage},
    }
);

impl crate::Config for Runtime {
    type AssetType = Assets;
    type Balance = Balance;
    type CurrencyType = Currencies;
    type Currencies = Tokens;
    type CampaignAssetType = CampaignAsset;
    type CampaignAssets = CampaignAssets;
    type CustomAssetType = CustomAsset;
    type CustomAssets = CustomAssets;
    type DestroyAccountWeight = DestroyWeight;
    type DestroyApprovalWeight = DestroyWeight;
    type DestroyFinishWeight = DestroyWeight;
    type MarketAssetType = MarketAsset;
    type MarketAssets = MarketAssets;
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
    type RuntimeOrigin = RuntimeOrigin;
    type PalletInfo = PalletInfo;
    type SS58Prefix = ();
    type SystemWeightInfo = ();
    type Version = ();
    type OnSetCode = ();
}

parameter_type_with_key! {
    pub ExistentialDeposits: |_currency_id: Currencies| -> Balance {
        CURRENCY_MIN_BALANCE
    };
}

impl orml_tokens::Config for Runtime {
    type Amount = Amount;
    type Balance = Balance;
    type CurrencyId = Currencies;
    type DustRemovalWhitelist = ();
    type RuntimeEvent = RuntimeEvent;
    type ExistentialDeposits = ExistentialDeposits;
    type MaxLocks = MaxLocks;
    type MaxReserves = MaxReserves;
    type CurrencyHooks = ();
    type ReserveIdentifier = [u8; 8];
    type WeightInfo = ();
}

// Required for runtime benchmarks
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

impl pallet_assets::Config<CustomAssetsInstance> for Runtime {
    type ApprovalDeposit = ConstU128<0>;
    type AssetAccountDeposit = ConstU128<0>;
    type AssetDeposit = ConstU128<0>;
    type AssetId = CustomAsset;
    type AssetIdParameter = Compact<CustomAssetId>;
    type Balance = Balance;
    #[cfg(feature = "runtime-benchmarks")]
    type BenchmarkHelper = AssetsBenchmarkHelper;
    type CallbackHandle = ();
    type CreateOrigin = AsEnsureOriginWithArg<EnsureSigned<AccountIdTest>>;
    type Currency = Balances;
    type Extra = ();
    type ForceOrigin = EnsureSigned<AccountIdTest>;
    type Destroyer = AssetRouter;
    type Freezer = ();
    type MetadataDepositBase = ConstU128<0>;
    type MetadataDepositPerByte = ConstU128<0>;
    type RemoveItemsLimit = ConstU32<50>;
    type RuntimeEvent = RuntimeEvent;
    type StringLimit = ConstU32<255>;
    type WeightInfo = ();
}

impl pallet_assets::Config<CampaignAssetsInstance> for Runtime {
    type ApprovalDeposit = ConstU128<0>;
    type AssetAccountDeposit = ConstU128<0>;
    type AssetDeposit = ConstU128<0>;
    type AssetId = CampaignAsset;
    type AssetIdParameter = Compact<CampaignAssetId>;
    type Balance = Balance;
    #[cfg(feature = "runtime-benchmarks")]
    type BenchmarkHelper = AssetsBenchmarkHelper;
    type CallbackHandle = ();
    type CreateOrigin = AsEnsureOriginWithArg<EnsureSigned<AccountIdTest>>;
    type Currency = Balances;
    type Extra = ();
    type ForceOrigin = EnsureSigned<AccountIdTest>;
    type Destroyer = AssetRouter;
    type Freezer = ();
    type MetadataDepositBase = ConstU128<0>;
    type MetadataDepositPerByte = ConstU128<0>;
    type RemoveItemsLimit = ConstU32<50>;
    type RuntimeEvent = RuntimeEvent;
    type StringLimit = ConstU32<255>;
    type WeightInfo = ();
}

// Required for runtime benchmarks
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

impl pallet_assets::Config<MarketAssetsInstance> for Runtime {
    type ApprovalDeposit = ConstU128<0>;
    type AssetAccountDeposit = ConstU128<0>;
    type AssetDeposit = ConstU128<0>;
    type AssetId = MarketAsset;
    type AssetIdParameter = MarketAsset;
    type Balance = Balance;
    #[cfg(feature = "runtime-benchmarks")]
    type BenchmarkHelper = MarketAssetsBenchmarkHelper;
    type CallbackHandle = ();
    type CreateOrigin = AsEnsureOriginWithArg<EnsureSigned<AccountIdTest>>;
    type Currency = Balances;
    type Extra = ();
    type ForceOrigin = EnsureSigned<AccountIdTest>;
    type Destroyer = AssetRouter;
    type Freezer = ();
    type MetadataDepositBase = ConstU128<0>;
    type MetadataDepositPerByte = ConstU128<0>;
    type RemoveItemsLimit = ConstU32<50>;
    type RuntimeEvent = RuntimeEvent;
    type StringLimit = ConstU32<255>;
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

pub(super) struct ExtBuilder {
    balances: Vec<(AccountIdTest, Balance)>,
}

impl Default for ExtBuilder {
    fn default() -> Self {
        Self { balances: vec![(ALICE, 1_000 * BASE), (BOB, 1_000 * BASE), (CHARLIE, 1_000 * BASE)] }
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

#[frame_support::transactional]
pub(super) fn managed_destroy_multi_transactional(
    assets: BTreeMap<Assets, Option<AccountIdTest>>,
) -> DispatchResult {
    AssetRouter::managed_destroy_multi(assets)
}
