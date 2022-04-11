#![no_main]

use libfuzzer_sys::fuzz_target;

use zeitgeist_primitives::{
    constants::{MaxAssets, MaxTotalWeight, MaxWeight, MinAssets, MinWeight},
    traits::Swaps as SwapsTrait,
    types::{Asset, PoolId, ScalarPosition, ScoringRule, SerdeWrapper},
};

use frame_support::ensure;

use zrml_swaps::mock::{ExtBuilder, Swaps};

use arbitrary::{Result, Unstructured};

fuzz_target!(|data: PoolCreationData| {
    let mut ext = ExtBuilder::default().build();
    let _ = ext.execute_with(|| {
        let _ = Swaps::create_pool(
            data.origin.into(),
            data.assets.into_iter().map(asset).collect(),
            data.base_asset.map(asset),
            data.market_id.into(),
            ScoringRule::CPMM,
            data.swap_fee.into(),
            data.weights.into(),
        );
    });
    let _ = ext.commit_all();
});

#[derive(Debug, arbitrary::Arbitrary)]
pub struct PoolCreationData {
    origin: u8,
    assets: Vec<(u128, u16)>,
    base_asset: Option<(u128, u16)>,
    market_id: u128,
    swap_fee: Option<u128>,
    weights: Option<Vec<u128>>,
}

impl<'a> arbitrary::Arbitrary<'a> for ValidPoolData {
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
        let assets_len = u.arbitrary_len::<(u128, u16)>()?;
         // not in bounds checks
        ensure!(assets_len <= usize::from(MaxAssets::get()), <arbitrary::Error>::IncorrectFormat);
        ensure!(assets_len >= usize::from(MinAssets::get()), <arbitrary::Error>::IncorrectFormat);

        // create a weight collection with the capacity of assets length
        let mut weights: Vec<u128> = Vec::with_capacity(assets_len);
        let mut weight_sum = 0u128;
        while weights.len() != assets_len {
            // create inclusive range for the u128 weight
            // assume, that MinWeight < MaxWeight, if not then panic!
            let elem: u128 = u
                .int_in_range(MinWeight::get()..=MaxWeight::get())
                .expect("MinWeight should be smaller than MaxWeight");
            match weight_sum.checked_add(elem) {
                Some(sum) if sum <= MaxTotalWeight::get() => weight_sum = sum,
                // if sum > MaxTotalWeight or u128 Overflow (None case)
                _ => return Err(<arbitrary::Error>::IncorrectFormat.into()),
            }
            weights.push(elem);
        }

        /*
        Mock default balances in the range [0, 1, 2, 3, 4] are defined and greater than zero.
        This is required to reach the MinLiquidity check in create_pool,
        where the balance needs to be greater than MinLiquidity.
        For example: if MinLiquidity = 100 * BASE,
        then balance of 0 (or 1, 2, 3, 4) needs to be greater than 100 * BASE.
        */
        let origin: u8 =
            u.int_in_range(0..=4).expect("First should be smaller than second of range.");

        let mut assets: Vec<(u128, u16)> = Vec::with_capacity(assets_len);
        for _ in 0..assets_len {
            let elem = <(u128, u16)>::arbitrary(u)?;
            assets.push(elem);
        }

        let base_asset = <(u128, u16)>::arbitrary(u)?;
        let market_id = u128::arbitrary(u)?;
        let swap_fee = u128::arbitrary(u)?;

        Ok(ValidPoolData { origin, assets, base_asset, market_id, swap_fee, weights })
    }
}

pub fn get_valid_pool_id(
    data: ValidPoolData,
) -> core::result::Result<PoolId, sp_runtime::DispatchError> {
    Swaps::create_pool(
        data.origin.into(),
        data.assets.into_iter().map(asset).collect(),
        Some(data.base_asset).map(asset),
        data.market_id.into(),
        ScoringRule::CPMM,
        Some(data.swap_fee).into(),
        Some(data.weights).into(),
    )
}

#[derive(Debug)]
pub struct ValidPoolData {
    origin: u8,
    assets: Vec<(u128, u16)>,
    base_asset: (u128, u16),
    market_id: u128,
    swap_fee: u128,
    weights: Vec<u128>,
}

pub fn asset(seed: (u128, u16)) -> Asset<u128> {
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
}
