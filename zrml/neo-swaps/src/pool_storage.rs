use crate::{traits::PoolStorage, Config, Error, Pallet, PoolCount, PoolOf, Pools};
use sp_runtime::DispatchError;
use zeitgeist_primitives::math::checked_ops_res::CheckedAddRes;

impl<T> PoolStorage for Pallet<T>
where
    T: Config,
{
    type PoolId = T::PoolId;
    type Pool = PoolOf<T>;

    // TODO Make `PoolId` as u32.
    fn next_pool_id() -> T::PoolId {
        PoolCount::<T>::get()
    }

    fn add(pool: Self::Pool) -> Result<Self::PoolId, DispatchError> {
        let pool_id = Self::next_pool_id();
        Pools::<T>::insert(pool_id, pool);

        let pool_count = pool_id.checked_add_res(&1u8.into())?; // TODO Add CheckedInc.
        PoolCount::<T>::set(pool_count);

        Ok(pool_id)
    }

    fn get(pool_id: Self::PoolId) -> Result<Self::Pool, DispatchError> {
        Pools::<T>::get(pool_id).ok_or(Error::<T>::PoolNotFound.into())
    }

    fn try_mutate_pool<R, F>(pool_id: &Self::PoolId, mutator: F) -> Result<R, DispatchError>
    where
        F: FnMut(&mut PoolOf<T>) -> Result<R, DispatchError>,
    {
        Pools::<T>::try_mutate(pool_id, |maybe_pool| {
            maybe_pool.as_mut().ok_or(Error::<T>::PoolNotFound.into()).and_then(mutator)
        })
    }
}
