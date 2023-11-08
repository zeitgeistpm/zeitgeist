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

#![cfg(test)]

use super::*;
use test_case::test_case;

// Assets <> MarketAssetClass
#[test_case(
    Asset::<MarketId>::CategoricalOutcome(7, 7),
    MarketAssetClass::<MarketId>::OldCategoricalOutcome(7, 7);
    "categorical_outcome"
)]
#[test_case(
    Asset::<MarketId>::ScalarOutcome(7, ScalarPosition::Long),
    MarketAssetClass::<MarketId>::OldScalarOutcome(7, ScalarPosition::Long);
    "scalar_outcome"
)]
#[test_case(
    Asset::<MarketId>::CombinatorialOutcome,
    MarketAssetClass::<MarketId>::OldCombinatorialOutcome;
    "combinatorial_outcome"
)]
#[test_case(
    Asset::<MarketId>::PoolShare(7),
    MarketAssetClass::<MarketId>::OldPoolShare(7);
    "pool_share"
)]
#[test_case(
    Asset::<MarketId>::ParimutuelShare(7, 7),
    MarketAssetClass::<MarketId>::OldParimutuelShare(7, 7);
    "parimutuel_share"
)]
#[test_case(
    Asset::<MarketId>::NewCategoricalOutcome(7, 7),
    MarketAssetClass::<MarketId>::CategoricalOutcome(7, 7);
    "new_categorical_outcome"
)]
#[test_case(
    Asset::<MarketId>::NewScalarOutcome(7, ScalarPosition::Long),
    MarketAssetClass::<MarketId>::ScalarOutcome(7, ScalarPosition::Long);
    "new_calar_outcome"
)]
#[test_case(
    Asset::<MarketId>::NewCombinatorialOutcome,
    MarketAssetClass::<MarketId>::CombinatorialOutcome;
    "new_combinatorial_outcome"
)]
#[test_case(
    Asset::<MarketId>::NewPoolShare(7),
    MarketAssetClass::<MarketId>::PoolShare(7);
    "new_pool_share"
)]
#[test_case(
    Asset::<MarketId>::NewParimutuelShare(7, 7),
    MarketAssetClass::<MarketId>::ParimutuelShare(7, 7);
    "new_parimutuel_share"
)]
fn index_matching_works_for_market_assets(
    old_asset: Asset<MarketId>,
    new_asset: MarketAssetClass<MarketId>,
) {
    let old_asset_encoded: Vec<u8> = old_asset.encode();
    let new_asset_decoded =
        <MarketAssetClass<MarketId> as Decode>::decode(&mut old_asset_encoded.as_slice()).unwrap();
    assert_eq!(new_asset_decoded, new_asset);
}

// Assets <> CurrencyClass
#[test_case(
    Asset::<MarketId>::CategoricalOutcome(7, 7),
    CurrencyClass::<MarketId>::OldCategoricalOutcome(7, 7);
    "categorical_outcome"
)]
#[test_case(
    Asset::<MarketId>::ScalarOutcome(7, ScalarPosition::Long),
    CurrencyClass::<MarketId>::OldScalarOutcome(7, ScalarPosition::Long);
    "scalar_outcome"
)]
#[test_case(
    Asset::<MarketId>::CombinatorialOutcome,
    CurrencyClass::<MarketId>::OldCombinatorialOutcome;
    "combinatorial_outcome"
)]
#[test_case(
    Asset::<MarketId>::PoolShare(7),
    CurrencyClass::<MarketId>::OldPoolShare(7);
    "pool_share"
)]
#[test_case(
    Asset::<MarketId>::ParimutuelShare(7, 7),
    CurrencyClass::<MarketId>::OldParimutuelShare(7, 7);
    "parimutuel_share"
)]
#[test_case(
    Asset::<MarketId>::ForeignAsset(7),
    CurrencyClass::<MarketId>::ForeignAsset(7);
    "foreign_asset"
)]
fn index_matching_works_for_currencies(
    old_asset: Asset<MarketId>,
    new_asset: CurrencyClass<MarketId>,
) {
    let old_asset_encoded: Vec<u8> = old_asset.encode();
    let new_asset_decoded =
        <CurrencyClass<MarketId> as Decode>::decode(&mut old_asset_encoded.as_slice()).unwrap();
    assert_eq!(new_asset_decoded, new_asset);
}
