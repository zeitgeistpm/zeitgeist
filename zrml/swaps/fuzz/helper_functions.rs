use zeitgeist_primitives::types::{Asset, ScalarPosition, SerdeWrapper};

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
