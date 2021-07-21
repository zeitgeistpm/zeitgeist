use crate::{
    utils::{calculate_perthousand, calculate_perthousand_value},
    BlockSoldShares, OwnedValues,
};
use core::marker::PhantomData;
use sp_runtime::traits::Saturating;

pub struct TrackIncentivesBasedOnSoldShares<T>(PhantomData<T>);

// No-one can have more balance than `Balance::MAX` so most functions saturate rewards.
impl<T> TrackIncentivesBasedOnSoldShares<T>
where
    T: crate::Config,
{
    pub(crate) fn exec() {
        for (market_id, account_id, sold_shares) in <BlockSoldShares<T>>::iter() {
            let values = if let Ok(e) = <OwnedValues<T>>::try_get(market_id, account_id.clone()) {
                e
            } else {
                // Trying to retrieve an account that doesn't have any bought shares
                continue;
            };

            let ptd = if let Some(el) = calculate_perthousand(sold_shares, &values.total_shares) {
                el
            } else {
                // `total_shares` is zero
                continue;
            };

            let balance_to_subtract = calculate_perthousand_value(ptd, values.total_incentives);

            <OwnedValues<T>>::mutate(market_id, account_id, |values| {
                // `total_balance` or `total_shares` can be less than 0 but this possible
                // scenario is ignored here.
                values.total_incentives =
                    values.total_incentives.saturating_sub(balance_to_subtract);
                values.total_shares = values.total_shares.saturating_sub(sold_shares);
            });
        }
        <BlockSoldShares<T>>::remove_all(None);
    }
}
