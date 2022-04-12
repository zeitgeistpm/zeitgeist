#![allow(
    // Mocks are only used for fuzzing and unit tests
    clippy::integer_arithmetic
)]

use zeitgeist_primitives::constants::{MaxAssets, MaxTotalWeight, MaxWeight, MinAssets, MinWeight};

use sp_runtime::traits::One;

use arbitrary::{Arbitrary, Result, Unstructured};

#[derive(Debug, Arbitrary)]
pub struct ExactAmountData {
    pub origin: u8,
    pub pool_creation: ValidPoolData,
    pub asset: (u128, u16),
    pub pool_amount: u128,
    pub asset_amount: u128,
}

#[derive(Debug, Arbitrary)]
pub struct ExactAssetAmountData {
    pub origin: u8,
    pub pool_creation: ValidPoolData,
    pub asset: (u128, u16),
    pub asset_amount: u128,
    pub pool_amount: u128,
}

#[derive(Debug, Arbitrary)]
pub struct GeneralPoolData {
    pub origin: u8,
    pub pool_creation: ValidPoolData,
    pub pool_amount: u128,
    pub assets: Vec<u128>,
}

#[derive(Debug, Arbitrary)]
pub struct SwapExactAmountData {
    pub origin: u8,
    pub pool_creation: ValidPoolData,
    pub asset_in: (u128, u16),
    pub asset_amount_in: u128,
    pub asset_out: (u128, u16),
    pub asset_amount_out: u128,
    pub max_price: u128,
}

#[derive(Debug, Arbitrary)]
pub struct PoolCreationData {
    pub origin: u8,
    pub assets: Vec<(u128, u16)>,
    pub base_asset: Option<(u128, u16)>,
    pub market_id: u128,
    pub swap_fee: Option<u128>,
    pub weights: Option<Vec<u128>>,
}

#[derive(Debug)]
pub struct ValidPoolData {
    pub origin: u8,
    pub assets: Vec<(u128, u16)>,
    pub base_asset: (u128, u16),
    pub market_id: u128,
    pub swap_fee: u128,
    pub weights: Vec<u128>,
}

impl<'a> arbitrary::Arbitrary<'a> for ValidPoolData {
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
        // if assets_len == MaxAssets then search for random usize modulo (MaxAssets + 1)
        // MaxAssets modulo MaxAssets = 0 => therefore MaxAssets modulo (MaxAssets + 1) = MaxAssets
        // upper bound is a possibility now
        let max_assets = usize::from(MaxAssets::get()).saturating_add(One::one());

        let mut assets_len: Option<usize> = None;
        let assets_len: usize = loop {
            match assets_len {
                // assets length is above or equal lower bound
                Some(len) if len >= usize::from(MinAssets::get()) => break len,
                _ => {
                    // as long as under lower bound find another assets_len
                    let assets_len_raw = u.arbitrary_len::<(u128, u16)>()?;
                    assets_len = Some(assets_len_raw % max_assets);
                }
            }
        };

        // create a weight collection with the capacity of assets length
        let mut weights: Vec<u128> = Vec::with_capacity(assets_len);
        let mut weight_sum = 0u128;

        let mut assets: Vec<(u128, u16)> = Vec::with_capacity(assets_len);
        for _ in 0..assets_len {
            // create inclusive range for the u128 weight
            // assume, that MinWeight < MaxWeight, if not then panic!
            let weight: u128 = u
                .int_in_range(MinWeight::get()..=MaxWeight::get())
                .expect("MinWeight should be smaller than MaxWeight");
            match weight_sum.checked_add(weight) {
                Some(sum) if sum <= MaxTotalWeight::get() => weight_sum = sum,
                // if sum > MaxTotalWeight or u128 Overflow (None case)
                _ => return Err(<arbitrary::Error>::IncorrectFormat),
            }
            let asset = <(u128, u16)>::arbitrary(u)?;

            weights.push(weight);
            assets.push(asset);
        }

        // the base_assets needs to be in the assets
        let base_asset: (u128, u16) = {
            // minus one, because of the inclusive length to get a valid index
            let asset_index: usize = u.int_in_range(0..=(assets_len - 1)).expect("First in range should be smaller than second.");
            match assets.get(asset_index) {
                Some(a) => *a,
                None => return Err(<arbitrary::Error>::IncorrectFormat),
            }
        };

        /*
        Mock default balances in the range [0, 1, 2, 3, 4] are defined and greater than zero.
        This is required to reach the MinLiquidity check in create_pool,
        where the balance needs to be greater than MinLiquidity.
        For example: if MinLiquidity = 100 * BASE,
        then balance of 0 (or 1, 2, 3, 4) needs to be greater than 100 * BASE.
        */
        // u8 number modulo 5 is in the range [0, 1, 2, 3, 4]
        let origin: u8 = u8::arbitrary(u)? % 5;
        let market_id = u128::arbitrary(u)?;
        let swap_fee = u128::arbitrary(u)?;

        Ok(ValidPoolData { origin, assets, base_asset, market_id, swap_fee, weights })
    }
}
