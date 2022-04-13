#![allow(
    // Mocks are only used for fuzzing and unit tests
    clippy::integer_arithmetic
)]

use zeitgeist_primitives::constants::{MaxAssets, MaxTotalWeight, MinAssets, MinWeight};

use arbitrary::{Arbitrary, Result, Unstructured};

use rand::Rng;

#[derive(Debug)]
pub struct ValidPoolData {
    pub origin: u128,
    pub assets: Vec<(u128, u16)>,
    pub base_asset: (u128, u16),
    pub market_id: u128,
    pub swap_fee: u128,
    pub weights: Vec<u128>,
}

impl<'a> arbitrary::Arbitrary<'a> for ValidPoolData {
    fn arbitrary(_: &mut Unstructured<'a>) -> Result<Self> {
        let mut rng = rand::thread_rng();

        let assets_len: u16 = rng.gen_range(MinAssets::get()..=MaxAssets::get());

        let assets_len = assets_len as usize;
        // unstructured arbitrary_len function did always return zero, that's why using rand crate
        // create a weight collection with the capacity of assets length
        let mut weights: Vec<u128> = Vec::with_capacity(assets_len);
        let mut weight_sum = 0u128;

        let mut assets: Vec<(u128, u16)> = Vec::with_capacity(assets_len);

        for _ in 0..assets_len {
            // use always the min weight to rarely reach the constraint of MaxTotalWeight
            let weight: u128 = MinWeight::get();
            match weight_sum.checked_add(weight) {
                // MaxWeight is 50 * BASE and MaxTotalWeight is also 50 * BASE
                // so it is very likely to reach the max total weight with two assets
                Some(sum) if sum <= MaxTotalWeight::get() => weight_sum = sum,
                // if sum > MaxTotalWeight or u128 Overflow (None case)
                _ => return Err(<arbitrary::Error>::IncorrectFormat),
            }
            let asset = (rng.gen::<u128>(), rng.gen::<u16>());

            weights.push(weight);
            assets.push(asset);
        }

        let origin = rng.gen::<u128>();
        // the base_assets needs to be in the assets
        let base_asset = *assets
            .get(rng.gen::<usize>() % assets_len)
            .ok_or(<arbitrary::Error>::IncorrectFormat)?;
        let market_id = rng.gen::<u128>();
        let swap_fee = rng.gen::<u128>();

        Ok(ValidPoolData { origin, assets, base_asset, market_id, swap_fee, weights })
    }
}

#[derive(Debug, Arbitrary)]
pub struct ExactAmountData {
    pub origin: u128,
    pub pool_creation: ValidPoolData,
    pub asset: (u128, u16),
    pub pool_amount: u128,
    pub asset_amount: u128,
}

#[derive(Debug, Arbitrary)]
pub struct ExactAssetAmountData {
    pub origin: u128,
    pub pool_creation: ValidPoolData,
    pub asset: (u128, u16),
    pub asset_amount: u128,
    pub pool_amount: u128,
}

#[derive(Debug, Arbitrary)]
pub struct GeneralPoolData {
    pub origin: u128,
    pub pool_creation: ValidPoolData,
    pub pool_amount: u128,
    pub assets: Vec<u128>,
}

#[derive(Debug, Arbitrary)]
pub struct SwapExactAmountData {
    pub origin: u128,
    pub pool_creation: ValidPoolData,
    pub asset_in: (u128, u16),
    pub asset_amount_in: u128,
    pub asset_out: (u128, u16),
    pub asset_amount_out: u128,
    pub max_price: u128,
}

#[derive(Debug, Arbitrary)]
pub struct PoolCreationData {
    pub origin: u128,
    pub assets: Vec<(u128, u16)>,
    pub base_asset: Option<(u128, u16)>,
    pub market_id: u128,
    pub swap_fee: Option<u128>,
    pub weights: Option<Vec<u128>>,
}
