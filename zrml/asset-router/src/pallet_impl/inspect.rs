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

use crate::pallet::*;

// Supertrait of Create and Destroy
impl<T: Config> Inspect<T::AccountId> for Pallet<T> {
    type AssetId = T::AssetType;
    type Balance = T::Balance;

    fn total_issuance(asset: Self::AssetId) -> Self::Balance {
        route_call!(asset, total_issuance, total_issuance,).unwrap_or(Zero::zero())
    }

    fn active_issuance(asset: Self::AssetId) -> Self::Balance {
        route_call!(asset, total_issuance, active_issuance,).unwrap_or(Zero::zero())
    }

    fn minimum_balance(asset: Self::AssetId) -> Self::Balance {
        route_call!(asset, minimum_balance, minimum_balance,).unwrap_or(Zero::zero())
    }

    fn balance(asset: Self::AssetId, who: &T::AccountId) -> Self::Balance {
        route_call!(asset, free_balance, balance, who).unwrap_or(Zero::zero())
    }

    fn total_balance(asset: Self::AssetId, who: &T::AccountId) -> Self::Balance {
        route_call!(asset, total_balance, total_balance, who).unwrap_or(Zero::zero())
    }

    fn reducible_balance(
        asset: Self::AssetId,
        who: &T::AccountId,
        preservation: Preservation,
        force: Fortitude,
    ) -> Self::Balance {
        if let Ok(asset_inner) = T::MarketAssetType::try_from(asset) {
            if T::MarketAssets::asset_exists(asset_inner) {
                return only_asset!(
                    asset,
                    Zero::zero(),
                    Inspect,
                    reducible_balance,
                    who,
                    preservation,
                    force
                );
            }
        }

        if let Ok(_) = T::CurrencyType::try_from(asset) {
            let reducible = <Self as MultiCurrency<T::AccountId>>::free_balance(asset, who);

            match preservation {
                Preservation::Expendable => reducible,
                Preservation::Protect | Preservation::Preserve => {
                    let min_balance = <Self as MultiCurrency<T::AccountId>>::minimum_balance(asset);
                    reducible.saturating_sub(min_balance)
                }
            }
        } else {
            only_asset!(asset, Zero::zero(), Inspect, reducible_balance, who, preservation, force)
        }
    }

    fn can_deposit(
        asset: Self::AssetId,
        who: &T::AccountId,
        amount: Self::Balance,
        provenance: Provenance,
    ) -> DepositConsequence {
        if let Ok(asset_inner) = T::MarketAssetType::try_from(asset) {
            if T::MarketAssets::asset_exists(asset_inner) {
                return only_asset!(
                    asset,
                    DepositConsequence::UnknownAsset,
                    Inspect,
                    can_deposit,
                    who,
                    amount,
                    provenance
                );
            }
        }

        if let Ok(_) = T::CurrencyType::try_from(asset) {
            let total_balance = <Self as MultiCurrency<T::AccountId>>::total_balance(asset, who);
            let min_balance = <Self as MultiCurrency<T::AccountId>>::minimum_balance(asset);

            if total_balance.saturating_add(amount) < min_balance {
                DepositConsequence::BelowMinimum
            } else {
                DepositConsequence::Success
            }
        } else {
            only_asset!(
                asset,
                DepositConsequence::UnknownAsset,
                Inspect,
                can_deposit,
                who,
                amount,
                provenance
            )
        }
    }

    fn can_withdraw(
        asset: Self::AssetId,
        who: &T::AccountId,
        amount: Self::Balance,
    ) -> WithdrawConsequence<Self::Balance> {
        if let Ok(asset_inner) = T::MarketAssetType::try_from(asset) {
            if T::MarketAssets::asset_exists(asset_inner) {
                return only_asset!(
                    asset,
                    WithdrawConsequence::UnknownAsset,
                    Inspect,
                    can_withdraw,
                    who,
                    amount
                );
            }
        }

        if let Ok(_) = T::CurrencyType::try_from(asset) {
            let can_withdraw =
                <Self as MultiCurrency<T::AccountId>>::ensure_can_withdraw(asset, who, amount);

            if let Err(_e) = can_withdraw {
                return WithdrawConsequence::BalanceLow;
            }

            let total_balance = <Self as MultiCurrency<T::AccountId>>::total_balance(asset, who);
            let min_balance = <Self as MultiCurrency<T::AccountId>>::minimum_balance(asset);
            let remainder = total_balance.saturating_sub(amount);

            if remainder < min_balance {
                WithdrawConsequence::ReducedToZero(remainder)
            } else {
                WithdrawConsequence::Success
            }
        } else {
            only_asset!(
                asset,
                WithdrawConsequence::UnknownAsset,
                Inspect,
                can_withdraw,
                who,
                amount
            )
        }
    }

    fn asset_exists(asset: Self::AssetId) -> bool {
        if let Ok(currency) = T::CurrencyType::try_from(asset) {
            if <T::Currencies as MultiCurrency<T::AccountId>>::total_issuance(currency)
                > Zero::zero()
            {
                true
            } else {
                only_asset!(asset, false, Inspect, asset_exists,)
            }
        } else {
            only_asset!(asset, false, Inspect, asset_exists,)
        }
    }
}
