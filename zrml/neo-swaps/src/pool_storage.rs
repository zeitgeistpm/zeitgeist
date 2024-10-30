// Copyright 2024 Forecasting Technologies LTD.
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
