// Copyright 2024-2025 Forecasting Technologies LTD.
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
use frame_support::require_transactional;
use sp_runtime::DispatchError;
use zeitgeist_primitives::math::checked_ops_res::CheckedIncRes;

impl<T> PoolStorage for Pallet<T>
where
    T: Config,
{
    type PoolId = T::PoolId;
    type Pool = PoolOf<T>;

    fn next_pool_id() -> T::PoolId {
        PoolCount::<T>::get()
    }

    #[require_transactional]
    fn add(pool: Self::Pool) -> Result<Self::PoolId, DispatchError> {
        let pool_id = Self::next_pool_id();
        Pools::<T>::insert(pool_id, pool);

        let pool_count = pool_id.checked_inc_res()?;
        PoolCount::<T>::set(pool_count);

        Ok(pool_id)
    }

    fn get(pool_id: Self::PoolId) -> Result<Self::Pool, DispatchError> {
        Pools::<T>::get(pool_id).ok_or(Error::<T>::PoolNotFound.into())
    }

    fn try_mutate_pool<R, F>(pool_id: &Self::PoolId, mutator: F) -> Result<R, DispatchError>
    where
        F: FnMut(&mut Self::Pool) -> Result<R, DispatchError>,
    {
        Pools::<T>::try_mutate(pool_id, |maybe_pool| {
            maybe_pool.as_mut().ok_or(Error::<T>::PoolNotFound.into()).and_then(mutator)
        })
    }

    fn try_mutate_exists<R, F>(pool_id: &Self::PoolId, mutator: F) -> Result<R, DispatchError>
    where
        F: FnMut(&mut Self::Pool) -> Result<(R, bool), DispatchError>,
    {
        Pools::<T>::try_mutate_exists(pool_id, |maybe_pool| {
            let (result, delete) =
                maybe_pool.as_mut().ok_or(Error::<T>::PoolNotFound.into()).and_then(mutator)?;

            if delete {
                *maybe_pool = None;
            }

            Ok(result)
        })
    }
}
