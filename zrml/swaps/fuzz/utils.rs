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
    clippy::integer_arithmetic,
    clippy::type_complexity,
)]

use zeitgeist_primitives::constants::{
    MaxAssets, MaxTotalWeight, MaxWeight, MinAssets, MinLiquidity, MinWeight, BASE,
};

use arbitrary::{Arbitrary, Result, Unstructured};

use rand::{rngs::ThreadRng, seq::SliceRandom, Rng};

use zeitgeist_primitives::{
    constants::MaxSwapFee,
    types::{Asset, ScalarPosition, SerdeWrapper},
};

use zeitgeist_primitives::{
    traits::Swaps as SwapsTrait,
    types::{PoolId, ScoringRule},
};
use zrml_swaps::mock::Swaps;

pub fn construct_asset(seed: (u8, u128, u16)) -> Asset<u128> {
    let (module, seed0, seed1) = seed;
    match module % 5 {
        0 => Asset::CategoricalOutcome(seed0, seed1),
        1 => {
            let scalar_position =
                if seed1 % 2 == 0 { ScalarPosition::Long } else { ScalarPosition::Short };
            Asset::ScalarOutcome(seed0, scalar_position)
        }
        2 => Asset::CombinatorialOutcome,
        3 => Asset::PoolShare(SerdeWrapper(seed0)),
        _ => Asset::Ztg,
    }
}

fn construct_swap_fee(swap_fee: u128) -> Option<u128> {
    Some(swap_fee % MaxSwapFee::get())
}

#[derive(Debug)]
pub struct ValidPoolData {
    pub origin: u128,
    pub assets: Vec<(u8, u128, u16)>,
    pub base_asset: (u8, u128, u16),
    pub market_id: u128,
    pub swap_fee: u128,
    pub amount: u128,
    pub weights: Vec<u128>,
}

impl ValidPoolData {
    // This function is called in the swap fuzz tests.
    #[allow(dead_code)]
    pub fn create_pool(self) -> PoolId {
        match Swaps::create_pool(
            self.origin,
            self.assets.into_iter().map(construct_asset).collect(),
            construct_asset(self.base_asset),
            self.market_id,
            ScoringRule::CPMM,
            construct_swap_fee(self.swap_fee),
            Some(self.amount),
            Some(self.weights),
        ) {
            Ok(pool_id) => pool_id,
            Err(e) => panic!("Pool creation failed unexpectedly. Error: {:?}", e),
        }
    }
}

impl<'a> arbitrary::Arbitrary<'a> for ValidPoolData {
    fn arbitrary(_: &mut Unstructured<'a>) -> Result<Self> {
        let mut rng = rand::thread_rng();

        // unstructured arbitrary_len function did always return zero, that's why using rand crate
        let assets_len: u16 = rng.gen_range(MinAssets::get()..=MaxAssets::get());

        let assets_len = assets_len as usize;

        // create assets and weights collections with assets length
        let (assets, weights) = create_random_assets_and_weights(assets_len, &mut rng)?;

        // the base_assets needs to be in the assets
        let base_asset = *assets
            .get(rng.gen::<usize>() % assets_len)
            .ok_or(<arbitrary::Error>::IncorrectFormat)?;

        let origin = rng.gen::<u128>();
        let market_id = rng.gen::<u128>();
        let swap_fee = rng.gen_range(0..BASE);
        let amount = rng.gen_range(MinLiquidity::get()..u128::MAX);

        Ok(ValidPoolData { origin, assets, base_asset, market_id, swap_fee, amount, weights })
    }
}

fn create_random_assets_and_weights(
    assets_len: usize,
    rng: &mut ThreadRng,
) -> Result<(Vec<(u8, u128, u16)>, Vec<u128>)> {
    let mut assets: Vec<(u8, u128, u16)> = Vec::with_capacity(assets_len);
    let mut weights: Vec<u128> = Vec::with_capacity(assets_len);

    assert!(MaxWeight::get() <= MaxTotalWeight::get());

    let mut weight_sum = 0;

    let assets_len = assets_len as u128;
    for i in 0..assets_len {
        // first iteration: (asset_len - 1) assets left
        let assets_left = assets_len - 1 - i;
        // reservation of multiple MinWeight for the future iterations
        let future_min_weight_reserve = assets_left * MinWeight::get();
        // take min_weight_reserve for future iterations in calculation
        // maximum value of weight without looking at the previous weight_sum
        let max_weight_limit = MaxTotalWeight::get() - future_min_weight_reserve;
        // previous weight sum substraction limits to exceed the MaxTotalWeight (otherwise no pool creation)
        let max_weight_limit = max_weight_limit - weight_sum;
        // each individual weight is at most MaxWeight
        let max = max_weight_limit.min(MaxWeight::get());
        let weight = rng.gen_range(MinWeight::get()..max);

        match weight_sum.checked_add(weight) {
            Some(sum) => weight_sum = sum,
            None => return Err(<arbitrary::Error>::IncorrectFormat),
        };
        weights.push(weight);

        let mut asset = (rng.gen::<u8>(), rng.gen::<u128>(), rng.gen::<u16>());
        while assets.clone().into_iter().map(construct_asset).any(|a| a == construct_asset(asset)) {
            // another try for finding a non-duplicated asset
            asset = (rng.gen::<u8>(), rng.gen::<u128>(), rng.gen::<u16>());
        }

        assets.push(asset);
    }
    // Need to shuffle the vector, because earlier numbers have a higher probability of being
    // large.
    weights.shuffle(rng);
    Ok((assets, weights))
}

#[derive(Debug, Arbitrary)]
pub struct ExactAmountData {
    pub origin: u128,
    pub pool_creation: ValidPoolData,
    pub asset: (u8, u128, u16),
    pub pool_amount: u128,
    pub asset_amount: u128,
}

#[derive(Debug, Arbitrary)]
pub struct ExactAssetAmountData {
    pub origin: u128,
    pub pool_creation: ValidPoolData,
    pub asset: (u8, u128, u16),
    pub asset_amount: u128,
    pub pool_amount: u128,
}

#[derive(Debug, Arbitrary)]
pub struct GeneralPoolData {
    pub origin: u128,
    pub pool_creation: ValidPoolData,
    pub pool_amount: u128,
    pub asset_bounds: Vec<u128>,
}

#[derive(Debug, Arbitrary)]
pub struct SwapExactAmountInData {
    pub origin: u128,
    pub pool_creation: ValidPoolData,
    pub asset_in: (u8, u128, u16),
    pub asset_amount_in: u128,
    pub asset_out: (u8, u128, u16),
    pub asset_amount_out: Option<u128>,
    pub max_price: Option<u128>,
}

#[derive(Debug, Arbitrary)]
pub struct SwapExactAmountOutData {
    pub origin: u128,
    pub pool_creation: ValidPoolData,
    pub asset_in: (u8, u128, u16),
    pub asset_amount_in: Option<u128>,
    pub asset_out: (u8, u128, u16),
    pub asset_amount_out: u128,
    pub max_price: Option<u128>,
}

#[derive(Debug, Arbitrary)]
pub struct PoolCreationData {
    pub origin: u128,
    pub assets: Vec<(u8, u128, u16)>,
    pub base_asset: (u8, u128, u16),
    pub market_id: u128,
    pub swap_fee: Option<u128>,
    pub amount: Option<u128>,
    pub weights: Option<Vec<u128>>,
}
