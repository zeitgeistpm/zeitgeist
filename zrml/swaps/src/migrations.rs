pub mod _0_1_1_add_pool_status {
    use crate::{BalanceOf, Config, Pallet, Pools};
    use alloc::{collections::BTreeMap, vec::Vec};
    use frame_support::{
        dispatch::Weight,
        traits::{Get, GetPalletVersion, PalletVersion},
    };
    use zeitgeist_primitives::types::{Asset, Pool, PoolStatus};

    type PreviousPoolTy<T> = PreviousPool<BalanceOf<T>, <T as crate::Config>::MarketId>;

    #[derive(
        Clone,
        Eq,
        PartialEq,
        parity_scale_codec::Decode,
        parity_scale_codec::Encode,
        sp_runtime::RuntimeDebug,
    )]
    struct PreviousPool<B, MI> {
        assets: Vec<Asset<MI>>,
        swap_fee: B,
        total_weight: u128,
        weights: BTreeMap<Asset<MI>, u128>,
    }

    pub fn migrate<T>() -> Weight
    where
        T: Config,
    {
        let mut weight: Weight = 0;
        let version_0_1_0 = PalletVersion {
            major: 0,
            minor: 1,
            patch: 0,
        };
        let storage_version = <Pallet<T>>::storage_version().unwrap_or(version_0_1_0);

        if storage_version == version_0_1_0 {
            let _ = <Pools<T>>::translate::<Option<PreviousPoolTy<T>>, _>(|_, pool_opt| {
                weight = weight.saturating_add(T::DbWeight::get().reads(1));
                if let Some(pool) = pool_opt {
                    weight = weight.saturating_add(T::DbWeight::get().writes(1));
                    Some(Some(Pool {
                        assets: pool.assets,
                        pool_status: PoolStatus::Active,
                        swap_fee: pool.swap_fee,
                        total_weight: pool.total_weight,
                        weights: pool.weights,
                    }))
                } else {
                    None
                }
            });
        }

        weight
    }

    #[cfg(test)]
    mod test {
        use super::*;
        use crate::mock::{ExtBuilder, Runtime, Swaps};
        use frame_support::{traits::OnRuntimeUpgrade, Hashable};
        use zeitgeist_primitives::types::{Balance, MarketId};

        #[test]
        fn _0_1_1_upgrade_should_include_pool_status() {
            ExtBuilder::default().build().execute_with(|| {
                frame_system::Pallet::<Runtime>::set_block_number(1);

                let previous_pool: PreviousPool<Balance, MarketId> = PreviousPool {
                    assets: Default::default(),
                    swap_fee: 0,
                    total_weight: 0,
                    weights: Default::default(),
                };

                let hash = 1u128.blake2_128_concat();
                frame_support::migration::put_storage_value(
                    b"Swaps",
                    b"Pools",
                    &hash,
                    Some(previous_pool),
                );

                Swaps::on_runtime_upgrade();

                let pool_opt = Pools::<Runtime>::iter()
                    .collect::<Vec<_>>()
                    .pop()
                    .unwrap()
                    .1;

                assert_eq!(
                    pool_opt,
                    Some(Pool {
                        assets: Default::default(),
                        pool_status: PoolStatus::Active,
                        swap_fee: 0,
                        total_weight: 0,
                        weights: Default::default(),
                    })
                )
            })
        }
    }
}
