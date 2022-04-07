#![no_main]

use libfuzzer_sys::fuzz_target;

use sp_runtime::traits::CheckedAdd;
use zeitgeist_primitives::{
    constants::{MaxAssets, MaxTotalWeight, MaxWeight, MinAssets, MinWeight},
    traits::Swaps as SwapsTrait,
    types::{Asset, PoolId, ScalarPosition, ScoringRule, SerdeWrapper},
};

use zrml_swaps::mock::{ExtBuilder, Swaps};

use arbitrary::{Result, Unstructured};

fuzz_target!(|data: PoolCreation| {
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
pub struct PoolCreation {
    origin: u8,
    assets: Vec<(u128, u16)>,
    base_asset: Option<(u128, u16)>,
    market_id: u128,
    swap_fee: Option<u128>,
    weights: Option<Vec<u128>>,
}

#[derive(Debug)]
pub struct ValidPoolCreation {
    origin: u8,
    assets: Vec<(u128, u16)>,
    base_asset: (u128, u16),
    market_id: u128,
    swap_fee: u128,
    weights: Vec<u128>,
}

impl<'a> arbitrary::Arbitrary<'a> for ValidPoolCreation {
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
        let origin = u8::arbitrary(u)?;

        let asset_iter = u.arbitrary_iter::<(u128, u16)>()?;
        let mut assets: Vec<(u128, u16)> = Vec::new();
        for (index, elem_result) in asset_iter.enumerate() {
            let elem = elem_result?;
            if assets.len() <= usize::from(MaxAssets::get()) {
                // in the bound
                assets.insert(index, elem);
            }
        }
        if assets.len() < usize::from(MinAssets::get()) {
            // below bound
            return Err(<arbitrary::Error>::IncorrectFormat.into());
        }

        let base_asset = (u128::arbitrary(u)?, u16::arbitrary(u)?);
        let market_id = u128::arbitrary(u)?;
        let swap_fee = u128::arbitrary(u)?;

        let weight_iter = u.arbitrary_iter::<u128>()?;
        let mut weights: Vec<u128> = Vec::new();
        let mut weight_sum = 0;
        for elem_result in weight_iter {
            if weights.clone().len() == assets.len() {
                // weights length and assets length need to be equal to get a valid pool creation
                // enough weights generated
                break;
            }
            let elem = elem_result?;
            let weight_sum_opt = weight_sum.checked_add(&elem);
            if let Some(weight_sum_value) = weight_sum_opt {
                if elem <= MaxWeight::get()
                    && elem >= MinWeight::get()
                    && weight_sum_value <= MaxTotalWeight::get()
                {
                    weights.push(elem);
                    weight_sum = weight_sum_value;
                }
            } else {
                break;
            }
        }

        if assets.len() != weights.clone().len() {
            return Err(<arbitrary::Error>::IncorrectFormat.into());
        }

        // TODO add last checks to generate a valid pool
        for (asset, weight) in assets.iter().copied().zip(weights.clone()) {
            /*
            let free_balance = T::Shares::free_balance(asset, &data.origin);
            if free_balance < MinLiquidity::get() {
                // insufficient balance
                return Err(<arbitrary::Error>::IncorrectFormat.into());
            }
            */
        }

        Ok(ValidPoolCreation { origin, assets, base_asset, market_id, swap_fee, weights })
    }
}

pub fn get_valid_pool_id(data: ValidPoolCreation) -> Option<PoolId> {
    let pool_id_result = Swaps::create_pool(
        data.origin.into(),
        data.assets.into_iter().map(asset).collect(),
        Some(data.base_asset).map(asset),
        data.market_id.into(),
        ScoringRule::CPMM,
        Some(data.swap_fee).into(),
        Some(data.weights).into(),
    );

    if pool_id_result.is_err() {
        return None;
    }

    Some(pool_id_result.unwrap())
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
