// Copyright 2023 Forecasting Technologies LTD.
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

use crate::{
    types::{Order, OrderId},
    AccountIdOf, BalanceOf, Config, MarketIdOf, Pallet as OrderbookPallet,
};
#[cfg(feature = "try-runtime")]
use alloc::collections::BTreeMap;
#[cfg(feature = "try-runtime")]
use alloc::format;
#[cfg(feature = "try-runtime")]
use alloc::vec::Vec;
#[cfg(feature = "try-runtime")]
use frame_support::migration::storage_key_iter;
use frame_support::{
    dispatch::Weight,
    log,
    pallet_prelude::PhantomData,
    traits::{Get, OnRuntimeUpgrade, StorageVersion},
    RuntimeDebug,
};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::traits::Saturating;
use zeitgeist_primitives::types::Asset;

#[cfg(any(feature = "try-runtime", test))]
const ORDER_BOOK: &[u8] = b"Orderbook";
#[cfg(any(feature = "try-runtime", test))]
const ORDERS: &[u8] = b"Orders";

const ORDER_BOOK_REQUIRED_STORAGE_VERSION: u16 = 0;
const ORDER_BOOK_NEXT_STORAGE_VERSION: u16 = 1;

#[derive(Clone, Encode, Eq, Decode, MaxEncodedLen, PartialEq, RuntimeDebug, TypeInfo)]
pub enum OldOrderSide {
    Bid,
    Ask,
}

#[derive(Clone, Encode, Eq, Decode, MaxEncodedLen, PartialEq, RuntimeDebug, TypeInfo)]
pub struct OldOrder<AccountId, Balance, MarketId: MaxEncodedLen> {
    pub market_id: MarketId,
    pub side: OldOrderSide,
    pub maker: AccountId,
    pub outcome_asset: Asset<MarketId>,
    pub base_asset: Asset<MarketId>,
    pub outcome_asset_amount: Balance,
    pub base_asset_amount: Balance,
}

type OldOrderOf<T> = OldOrder<AccountIdOf<T>, BalanceOf<T>, MarketIdOf<T>>;

#[frame_support::storage_alias]
pub(crate) type Orders<T: Config> =
    StorageMap<OrderbookPallet<T>, frame_support::Twox64Concat, OrderId, OldOrderOf<T>>;

pub struct TranslateOrderStructure<T>(PhantomData<T>);

impl<T: Config> OnRuntimeUpgrade for TranslateOrderStructure<T> {
    fn on_runtime_upgrade() -> Weight {
        let mut total_weight = T::DbWeight::get().reads(1);
        let order_book_pallet_version = StorageVersion::get::<OrderbookPallet<T>>();
        if order_book_pallet_version != ORDER_BOOK_REQUIRED_STORAGE_VERSION {
            log::info!(
                "TranslateOrderStructure: order book pallet version is {:?}, but {:?} is required",
                order_book_pallet_version,
                ORDER_BOOK_REQUIRED_STORAGE_VERSION,
            );
            return total_weight;
        }
        log::info!("TranslateOrderStructure: Starting...");

        let mut translated = 0u64;
        crate::Orders::<T>::translate::<OldOrderOf<T>, _>(|_order_id, old_order| {
            translated.saturating_inc();

            let (maker_asset, maker_amount, taker_asset, taker_amount) = match old_order.side {
                // the maker reserved the base asset for bids
                OldOrderSide::Bid => (
                    old_order.base_asset,
                    old_order.base_asset_amount,
                    old_order.outcome_asset,
                    old_order.outcome_asset_amount,
                ),
                // the maker reserved the outcome asset for asks
                OldOrderSide::Ask => (
                    old_order.outcome_asset,
                    old_order.outcome_asset_amount,
                    old_order.base_asset,
                    old_order.base_asset_amount,
                ),
            };

            let new_order = Order {
                market_id: old_order.market_id,
                maker: old_order.maker,
                maker_asset,
                maker_amount,
                taker_asset,
                taker_amount,
            };

            Some(new_order)
        });
        log::info!("TranslateOrderStructure: Upgraded {} orders.", translated);
        total_weight =
            total_weight.saturating_add(T::DbWeight::get().reads_writes(translated, translated));

        StorageVersion::new(ORDER_BOOK_NEXT_STORAGE_VERSION).put::<OrderbookPallet<T>>();
        total_weight = total_weight.saturating_add(T::DbWeight::get().writes(1));
        log::info!("TranslateOrderStructure: Done!");
        total_weight
    }

    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
        use frame_support::pallet_prelude::Twox64Concat;

        let old_orders =
            storage_key_iter::<OrderId, OldOrderOf<T>, Twox64Concat>(ORDER_BOOK, ORDERS)
                .collect::<BTreeMap<_, _>>();

        let orders = Orders::<T>::iter_keys().count() as u32;
        let decodable_orders = Orders::<T>::iter_values().count() as u32;
        if orders == decodable_orders {
            log::info!("All orders could successfully be decoded, order_count: {}.", orders);
        } else {
            log::error!(
                "Can only decode {} of {} orders - others will be dropped",
                decodable_orders,
                orders
            );
        }

        Ok(old_orders.encode())
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade(previous_state: Vec<u8>) -> Result<(), &'static str> {
        use orml_traits::NamedMultiReservableCurrency;
        use zeitgeist_primitives::traits::MarketCommonsPalletApi;

        let old_orders: BTreeMap<OrderId, OldOrderOf<T>> = Decode::decode(&mut &previous_state[..])
            .expect("Failed to decode state: Invalid state");
        let mut new_order_count = 0usize;
        for (order_id, new_order) in crate::Orders::<T>::iter() {
            let old_order =
                old_orders.get(&order_id).expect(&format!("Order {:?} not found", order_id)[..]);
            // assert old fields
            assert_eq!(old_order.market_id, new_order.market_id);
            assert_eq!(old_order.maker, new_order.maker);
            // assert new fields
            let reserved = T::AssetManager::reserved_balance_named(
                &OrderbookPallet::<T>::reserve_id(),
                new_order.maker_asset,
                &new_order.maker,
            );
            // one reserve_id is for all orders for this maker_asset
            assert!(reserved >= new_order.maker_amount);

            if let Ok(market) = T::MarketCommons::market(&new_order.market_id) {
                let base_asset = market.base_asset;
                if new_order.maker_asset == base_asset {
                    assert_eq!(new_order.maker_asset, old_order.base_asset);
                    assert_eq!(new_order.maker_amount, old_order.base_asset_amount);
                    assert_eq!(new_order.taker_amount, old_order.outcome_asset_amount);
                    assert_eq!(new_order.taker_asset, old_order.outcome_asset);
                } else {
                    assert_eq!(new_order.taker_asset, base_asset);
                    assert_eq!(new_order.taker_asset, old_order.base_asset);
                    assert_eq!(new_order.taker_amount, old_order.base_asset_amount);
                    assert_eq!(new_order.maker_amount, old_order.outcome_asset_amount);
                    assert_eq!(new_order.maker_asset, old_order.outcome_asset);
                }
            } else {
                log::error!(
                    "The market should be present for the order market id {:?}!",
                    new_order.market_id
                );
            }

            new_order_count.saturating_inc();
        }
        assert_eq!(old_orders.len(), new_order_count);
        log::info!("TranslateOrderStructure: Order Counter post-upgrade is {}!", new_order_count);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        mock::{ExtBuilder, Runtime},
        OrderOf,
    };
    use alloc::vec::Vec;
    use frame_support::{
        dispatch::fmt::Debug, migration::put_storage_value, storage_root, StorageHasher,
        Twox64Concat,
    };
    use sp_runtime::StateVersion;
    use zeitgeist_primitives::types::ScalarPosition;

    #[test]
    fn on_runtime_upgrade_increments_the_storage_version() {
        ExtBuilder::default().build().execute_with(|| {
            set_up_version();
            TranslateOrderStructure::<Runtime>::on_runtime_upgrade();
            assert_eq!(
                StorageVersion::get::<OrderbookPallet<Runtime>>(),
                ORDER_BOOK_NEXT_STORAGE_VERSION
            );
        });
    }

    #[test]
    fn on_runtime_upgrade_works_as_expected() {
        ExtBuilder::default().build().execute_with(|| {
            set_up_version();
            let (old_orders, new_orders) = construct_old_new_tuple();
            populate_test_data::<Twox64Concat, OrderId, OldOrderOf<Runtime>>(
                ORDER_BOOK,
                ORDERS,
                old_orders.clone(),
            );
            TranslateOrderStructure::<Runtime>::on_runtime_upgrade();

            let actual = crate::Orders::<Runtime>::get(0).unwrap();
            assert_eq!(actual, new_orders[0]);

            let actual = crate::Orders::<Runtime>::get(1).unwrap();
            assert_eq!(actual, new_orders[1]);
        });
    }

    #[test]
    fn on_runtime_upgrade_is_noop_if_versions_are_not_correct() {
        ExtBuilder::default().build().execute_with(|| {
            // ensure we migrated already
            StorageVersion::new(ORDER_BOOK_NEXT_STORAGE_VERSION).put::<OrderbookPallet<Runtime>>();

            // save current storage root
            let tmp = storage_root(StateVersion::V1);
            TranslateOrderStructure::<Runtime>::on_runtime_upgrade();
            // ensure we did not change any storage with the migration
            assert_eq!(tmp, storage_root(StateVersion::V1));
        });
    }

    fn set_up_version() {
        StorageVersion::new(ORDER_BOOK_REQUIRED_STORAGE_VERSION).put::<OrderbookPallet<Runtime>>();
    }

    fn construct_old_new_tuple() -> (Vec<OldOrderOf<Runtime>>, Vec<OrderOf<Runtime>>) {
        let market_id_0 = 0;
        let outcome_asset_amount_0 = 42000;
        let base_asset_amount_0 = 69000;
        let old_order_0 = OldOrder {
            market_id: market_id_0,
            side: OldOrderSide::Bid,
            maker: 1,
            outcome_asset: Asset::CategoricalOutcome(market_id_0, 0u16),
            base_asset: Asset::Ztg,
            outcome_asset_amount: outcome_asset_amount_0,
            base_asset_amount: base_asset_amount_0,
        };
        let new_order_0 = Order {
            market_id: market_id_0,
            maker: 1,
            // the maker reserved the base asset for order side bid
            maker_asset: Asset::Ztg,
            maker_amount: base_asset_amount_0,
            taker_asset: Asset::CategoricalOutcome(market_id_0, 0u16),
            taker_amount: outcome_asset_amount_0,
        };

        let market_id_1 = 1;
        let outcome_asset_amount_1 = 42000;
        let base_asset_amount_1 = 69000;
        let old_order_1 = OldOrder {
            market_id: market_id_1,
            side: OldOrderSide::Ask,
            maker: 1,
            outcome_asset: Asset::ScalarOutcome(market_id_1, ScalarPosition::Long),
            base_asset: Asset::Ztg,
            outcome_asset_amount: outcome_asset_amount_1,
            base_asset_amount: base_asset_amount_1,
        };
        let new_order_1 = Order {
            market_id: market_id_1,
            maker: 1,
            // the maker reserved the outcome asset for order side ask
            maker_asset: Asset::ScalarOutcome(market_id_1, ScalarPosition::Long),
            maker_amount: outcome_asset_amount_1,
            taker_asset: Asset::Ztg,
            taker_amount: base_asset_amount_1,
        };
        (vec![old_order_0, old_order_1], vec![new_order_0, new_order_1])
    }

    #[allow(unused)]
    fn populate_test_data<H, K, V>(pallet: &[u8], prefix: &[u8], data: Vec<V>)
    where
        H: StorageHasher,
        K: TryFrom<usize> + Encode,
        V: Encode + Clone,
        <K as TryFrom<usize>>::Error: Debug,
    {
        for (key, value) in data.iter().enumerate() {
            let storage_hash = K::try_from(key).unwrap().using_encoded(H::hash).as_ref().to_vec();
            put_storage_value::<V>(pallet, prefix, &storage_hash, (*value).clone());
        }
    }
}
