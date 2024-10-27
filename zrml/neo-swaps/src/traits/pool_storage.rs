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
