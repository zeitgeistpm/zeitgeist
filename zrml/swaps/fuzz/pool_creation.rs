#![no_main]

use libfuzzer_sys::fuzz_target;

use zeitgeist_primitives::types::{Asset, PoolId, ScalarPosition, ScoringRule, SerdeWrapper};

use zrml_swaps::mock::{ExtBuilder, Swaps};

use zeitgeist_primitives::traits::Swaps as SwapsTrait;

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
struct PoolCreation {
    origin: u8,
    assets: Vec<(u128, u16)>,
    base_asset: Option<(u128, u16)>,
    market_id: u128,
    swap_fee: Option<u128>,
    weights: Option<Vec<u128>>,
}

pub fn get_sample_pool_id() -> PoolId {
    0
    /* 
    fuzz_target!(|data: PoolCreation| {
        let mut ext = ExtBuilder::default().build();
        let pool_id_opt = ext.execute_with(|| {
            let pool_id_result = Swaps::create_pool(
                data.origin.into(),
                data.assets.into_iter().map(asset).collect(),
                data.base_asset.map(asset),
                data.market_id.into(),
                scoring_rule(data.scoring_rule),
                data.swap_fee.into(),
                data.weights.into(),
            );
            match pool_id_result {
                Ok(pool_id) => {
                    return Some(pool_id);
                }
                Err(_) => None,
            }
        });
        match pool_id_opt {
            Some(pool_id) => return pool_id,
            None =>
            /*repeat the fuzz_target with other data*/
            {
                continue;
            }
        }
        let _ = ext.commit_all();
    });
    */

    /*
    specific self selected pool id

    use sp_runtime::DispatchError;
    use zeitgeist_primitives::{constants::BASE};
    use zeitgeist_primitives::types::{MarketId};
    use zrml_swaps::mock::{BOB};

    const _2: u128 = 2 * BASE;

    pub const ASSET_A: Asset<MarketId> = Asset::CategoricalOutcome(0, 65);
    pub const ASSET_B: Asset<MarketId> = Asset::CategoricalOutcome(0, 66);
    pub const ASSET_C: Asset<MarketId> = Asset::CategoricalOutcome(0, 67);
    pub const ASSET_D: Asset<MarketId> = Asset::CategoricalOutcome(0, 68);
    pub const ASSET_E: Asset<MarketId> = Asset::CategoricalOutcome(0, 69);

    pub const ASSETS: [Asset<MarketId>; 4] = [ASSET_A, ASSET_B, ASSET_C, ASSET_D];

    ExtBuilder::default().build().execute_with(|| {
        // TODO this should be the first random (with fuzz target), but valid and created pool
        let pool_id_result: Result<PoolId, DispatchError> = Swaps::create_pool(
            BOB,
            ASSETS.iter().cloned().collect(),
            Some(ASSETS.last().unwrap().clone()),
            0,
            ScoringRule::CPMM,
            Some(0),
            Some(vec![_2, _2, _2, _2]),
        );
        // TODO hand over the valid pool id to the other fuzz_target tests (dispatch calls)
        match pool_id_result {
            Ok(pool_id) => {
                return pool_id;
            }
            Err(_) => panic!("Failed Swaps::create_pool"),
        }
    });
    panic!("Pool id not generated!");
    */
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
