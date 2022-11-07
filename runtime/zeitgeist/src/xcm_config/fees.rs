// Copyright 2022 Zeitgeist PM LLC.
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

use crate::{xcm_config::asset_registry::CustomMetadata, Balance, CurrencyId};
use core::marker::PhantomData;
use frame_support::weights::constants::{ExtrinsicBaseWeight, WEIGHT_PER_SECOND};
use xcm::latest::MultiLocation;
use zeitgeist_primitives::constants::BalanceFractionalDecimals;
use zrml_swaps::check_arithm_rslt::CheckArithmRslt;

/// The fee cost per second for transferring the native token in cents.
pub fn native_per_second() -> Balance {
    default_per_second(BalanceFractionalDecimals::get().into())
}

pub fn default_per_second(decimals: u32) -> Balance {
    let base_weight = Balance::from(ExtrinsicBaseWeight::get());
    let default_per_second = (WEIGHT_PER_SECOND as u128) / base_weight;
    default_per_second * base_fee(decimals)
}

fn base_fee(decimals: u32) -> Balance {
    cent(decimals).saturating_div(10)
}

/// 1 Asset in fixed point decimal representation
pub fn dollar(decimals: u32) -> Balance {
    10u128.saturating_pow(decimals)
}

/// 0.01 Asset in fixed point decimal presentation
pub fn cent(decimals: u32) -> Balance {
    dollar(decimals).saturating_div(100)
}

pub fn bmul(a: u128, b: u128, base: u128) -> Option<u128> {
    let c0 = a.check_mul_rslt(&b).ok()?;
    let c1 = c0.check_add_rslt(&base.check_div_rslt(&2).ok()?).ok()?;
    c1.check_div_rslt(&base).ok()
}

/// Our FixedConversionRateProvider, used to charge XCM-related fees for tokens registered in
/// the asset registry that were not already handled by native Trader rules.
pub struct FixedConversionRateProvider<AssetRegistry>(PhantomData<AssetRegistry>);

impl<
    AssetRegistry: orml_traits::asset_registry::Inspect<
        AssetId = CurrencyId,
        Balance = Balance,
        CustomMetadata = CustomMetadata,
    >,
> orml_traits::FixedConversionRateProvider for FixedConversionRateProvider<AssetRegistry>
{
    fn get_fee_per_second(location: &MultiLocation) -> Option<u128> {
        let metadata = AssetRegistry::metadata_by_location(&location)?;
        let default_per_second = default_per_second(metadata.decimals);

        if let Some(fee_factor) = metadata.additional.xcm.fee_factor {
            let base = 10u128.checked_pow(metadata.decimals.into())?;
            bmul(default_per_second, fee_factor, base)
        } else {
            Some(default_per_second)
        }
    }
}
