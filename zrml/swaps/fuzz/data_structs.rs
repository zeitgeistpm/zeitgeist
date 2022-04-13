#![allow(
    // Mocks are only used for fuzzing and unit tests
    clippy::integer_arithmetic
)]

use zeitgeist_primitives::constants::{
    MaxAssets, MaxTotalWeight, MinAssets, MinLiquidity, MinWeight,
};

use zrml_swaps::mock::ExtBuilder;

use arbitrary::{Arbitrary, Result, Unstructured};

use zrml_swaps::mock::Shares;

use orml_traits::MultiCurrency;

use rand::Rng;

use zeitgeist_primitives::types::{Asset, ScalarPosition, SerdeWrapper};

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
        let get_asset = |seed: (u128, u16)| {
            let (seed0, seed1) = seed;
            let module = seed0 % 4;
            match module {
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
        };

        let mut rng = rand::thread_rng();

        /*
        Mock default balances in the range [0, 1, 2, 3, 4] are defined and greater than zero.
        This is required to reach the MinLiquidity check in create_pool,
        where the balance needs to be greater than MinLiquidity.
        For example: if MinLiquidity = 100 * BASE,
        then balance of 0 (or 1, 2, 3, 4) needs to be greater than 100 * BASE.
        */
        // u8 number modulo 5 is in the range [0, 1, 2, 3, 4]
        let origin = rng.gen::<u128>() % 5;

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

            let mut ext = ExtBuilder::default().build();
            let _ = ext.execute_with(|| {
                // ensure that the account origin has a sufficient balance
                // use orml_traits::MultiCurrency; required for this
                let a = get_asset(asset);
                let _ = Shares::deposit(a, &origin, MinLiquidity::get());
            });
            let _ = ext.commit_all();

            weights.push(weight);
            assets.push(asset);
        }

        // the base_assets needs to be in the assets
        let base_asset = *assets
            .get(rng.gen::<usize>() % assets_len)
            .ok_or(<arbitrary::Error>::IncorrectFormat)?;
        let market_id = rng.gen::<u128>();
        let swap_fee = rng.gen::<u128>();

        println!(
            "ValidPoolData={:#?}",
            ValidPoolData {
                origin,
                assets: assets.clone(),
                base_asset,
                market_id,
                swap_fee,
                weights: weights.clone()
            }
        );

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
