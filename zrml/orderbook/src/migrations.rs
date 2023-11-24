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

use crate::{BalanceOf, MarketIdOf, Config, MomentOf};
#[cfg(feature = "try-runtime")]
use alloc::collections::BTreeMap;
#[cfg(feature = "try-runtime")]
use alloc::format;
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
use zeitgeist_primitives::types::{
    Asset, Bond, Deadlines, Market, MarketBonds, MarketCreation, MarketDisputeMechanism,
    MarketPeriod, MarketStatus, MarketType, OutcomeReport, Report, ScoringRule,
};

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

type OldOrderOf<T> = OldOrder<
    <T as frame_system::Config>::AccountId,
    BalanceOf<T>,
    MarketIdOf<T>,
>;

#[frame_support::storage_alias]
pub(crate) type Orders<T: Config> = StorageMap<
    OrderbookPallet<T>,
    frame_support::Twox64Concat,
    OrderId,
    OldOrderOf<T>,
>;

pub struct TranslateOrderStructure<T>(PhantomData<T>);

impl<T: Config> OnRuntimeUpgrade for TranslateOrderStructure<T> {
    fn on_runtime_upgrade() -> Weight {
        let mut total_weight = T::DbWeight::get().reads(1);
        let market_commons_version = StorageVersion::get::<OrderbookPallet<T>>();
        if market_commons_version != ORDER_BOOK_REQUIRED_STORAGE_VERSION {
            log::info!(
                "TranslateOrderStructure: market-commons version is {:?}, but {:?} is required",
                market_commons_version,
                ORDER_BOOK_REQUIRED_STORAGE_VERSION,
            );
            return total_weight;
        }
        log::info!("TranslateOrderStructure: Starting...");

        let mut translated = 0u64;
        crate::Orders::<T>::translate::<OldOrderOf<T>, _>(|_key, old_order| {
            translated.saturating_inc();

            let new_order = Order {
                market_id: old_order.market_id,
                maker: old_order.maker,
                maker_asset: old_order.outcome_asset,
                maker_amount: old_order.outcome_asset_amount,
                taker_asset: old_order.base_asset,
                taker_amount: old_order.base_asset_amount,
            };

            Some(new_order)
        });
        log::info!("TranslateOrderStructure: Upgraded {} orders.", translated);
        total_weight =
            total_weight.saturating_add(T::DbWeight::get().reads_writes(translated, translated));

        StorageVersion::new(ORDER_BOOK_NEXT_STORAGE_VERSION).put::<MarketCommonsPallet<T>>();
        total_weight = total_weight.saturating_add(T::DbWeight::get().writes(1));
        log::info!("TranslateOrderStructure: Done!");
        total_weight
    }

    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
        use frame_support::pallet_prelude::Twox64Concat;

        let old_markets = storage_key_iter::<MarketIdOf<T>, OldOrderOf<T>, Twox64Concat>(
            ORDER_BOOK,
            ORDERS,
        )
        .collect::<BTreeMap<_, _>>();

        let orders = Orders::<T>::iter_keys().count() as u32;
        let decodable_orders = Orders::<T>::iter_values().count() as u32;
        if orders != decodable_orders {
            log::error!(
                "Can only decode {} of {} orders - others will be dropped",
                decodable_orders,
                orders
            );
        } else {
            log::info!("orders: {}, Decodable orders: {}", orders, decodable_orders);
        }

        Ok(old_orders.encode())
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade(previous_state: Vec<u8>) -> Result<(), &'static str> {
        let old_orders: BTreeMap<MarketIdOf<T>, OldMarketOf<T>> =
            Decode::decode(&mut &previous_state[..])
                .expect("Failed to decode state: Invalid state");
        let new_order_count = <zrml_market_commons::Pallet<T>>::market_iter().count();
        assert_eq!(old_orders.len(), new_order_count);
        for (order_id, new_order) in crate::Orders::<T>::iter() {
            let old_order = old_orders
                .get(&market_id)
                .expect(&format!("Market {:?} not found", market_id)[..]);
            // assert old fields
            assert_eq!(new_order.base_asset, old_order.base_asset);
            // assert new fields
            assert_eq!(new_order.bonds.outsider, None);
        }
        log::info!("TranslateOrderStructure: Order Counter post-upgrade is {}!", new_order_count);
        // TODO maybe remove this since we have no orders in production at the moment
        assert!(new_order_count > 0);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        mock::{ExtBuilder, Runtime},
        MarketIdOf, MarketOf,
    };
    use frame_support::{
        dispatch::fmt::Debug, migration::put_storage_value, Twox64Concat, StorageHasher,
    };
    use zrml_market_commons::MarketCommonsPalletApi;

    #[test]
    fn on_runtime_upgrade_increments_the_storage_version() {
        ExtBuilder::default().build().execute_with(|| {
            set_up_version();
            TranslateOrderStructure::<Runtime>::on_runtime_upgrade();
            assert_eq!(
                StorageVersion::get::<MarketCommonsPallet<Runtime>>(),
                ORDER_BOOK_NEXT_STORAGE_VERSION
            );
        });
    }

    #[test]
    fn on_runtime_upgrade_is_noop_if_versions_are_not_correct() {
        ExtBuilder::default().build().execute_with(|| {
            // Don't set up chain to signal that storage is already up to date.
            let (_, new_orders) = construct_old_new_tuple();
            populate_test_data::<Twox64Concat, MarketIdOf<Runtime>, OrderOf<Runtime>>(
                ORDER_BOOK,
                ORDERS,
                new_orders.clone(),
            );
            TranslateOrderStructure::<Runtime>::on_runtime_upgrade();
            let actual = crate::Orders::<Runtime>::get(0).unwrap();
            assert_eq!(actual, new_orders[0]);
        });
    }

    fn set_up_version() {
        StorageVersion::new(ORDER_BOOK_REQUIRED_STORAGE_VERSION)
            .put::<MarketCommonsPallet<Runtime>>();
    }
}
