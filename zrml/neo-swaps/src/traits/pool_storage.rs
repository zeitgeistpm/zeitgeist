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

use sp_runtime::DispatchError;

pub(crate) trait PoolStorage {
    type PoolId;
    type Pool;

    fn next_pool_id() -> Self::PoolId;

    fn add(pool: Self::Pool) -> Result<Self::PoolId, DispatchError>;

    fn get(pool_id: Self::PoolId) -> Result<Self::Pool, DispatchError>;

    fn try_mutate_pool<R, F>(pool_id: &Self::PoolId, mutator: F) -> Result<R, DispatchError>
    where
        F: FnMut(&mut Self::Pool) -> Result<R, DispatchError>;
}