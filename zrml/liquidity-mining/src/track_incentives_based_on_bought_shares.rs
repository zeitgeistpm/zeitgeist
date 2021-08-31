use crate::{
    utils::{calculate_perthousand, calculate_perthousand_value},
    BalanceOf, BlockBoughtShares, MomentOf, OwnedValues, PerBlockIncentive,
};
use alloc::{collections::BTreeSet, vec::Vec};
use core::marker::PhantomData;
use sp_runtime::traits::{CheckedDiv, Saturating};
use zeitgeist_primitives::types::MarketPeriod;
use zrml_market_commons::MarketCommonsPalletApi;

// Per-thousand that every incentive should transfer to the perpetual balance.
// Currently is 0.1%
const PERPETUAL_PTD: u8 = 1;

pub struct TrackIncentivesBasedOnBoughtShares<T>(PhantomData<T>);

// No-one can have more balance than `Balance::MAX` so most functions saturate rewards.
impl<T> TrackIncentivesBasedOnBoughtShares<T>
where
    T: crate::Config,
{
    pub(crate) fn exec(curr_block: T::BlockNumber) -> Option<usize> {
        let per_block_incentives = <PerBlockIncentive<T>>::get();
        let market_incentives = Self::markets_incentives(per_block_incentives, curr_block)?;
        let market_incentives_len = market_incentives.len();
        let ppb = BalanceOf::<T>::from(PERPETUAL_PTD);

        for (market_id, incentive) in market_incentives {
            let mut total_bought_shares = BalanceOf::<T>::from(0u8);
            for (_, shares) in <BlockBoughtShares<T>>::iter_prefix(market_id) {
                total_bought_shares = total_bought_shares.saturating_add(shares);
            }
            let opt = Self::bought_share_value(&incentive, &total_bought_shares);
            let share_value = if let Some(el) = opt {
                el
            } else {
                continue;
            };
            for (account_id, bought_shares) in <BlockBoughtShares<T>>::iter_prefix(market_id) {
                let raw_incentives = share_value.saturating_mul(bought_shares);
                let perpetual_incentives = calculate_perthousand_value(ppb, raw_incentives);
                let incentives = raw_incentives.saturating_sub(perpetual_incentives);
                <OwnedValues<T>>::mutate(market_id, account_id, |values| {
                    let one = T::BlockNumber::from(1u8);

                    values.perpetual_incentives =
                        values.perpetual_incentives.saturating_add(perpetual_incentives);
                    values.total_incentives = values.total_incentives.saturating_add(incentives);
                    values.total_shares = values.total_shares.saturating_add(bought_shares);
                    values.participated_blocks = values.participated_blocks.saturating_add(one);
                });
            }
        }

        <BlockBoughtShares<T>>::remove_all(None);
        Some(market_incentives_len)
    }

    // ZTG value of one bought share for the current block being produced. Or in other words:
    // Determines how much a share will be worth given the amount of ZTG for liquidity
    // mining and the total number of bought shares for the current block.
    //
    // `None` result means no-one purchased a share.
    #[inline]
    fn bought_share_value(
        incentive: &BalanceOf<T>,
        total_bought_shares: &BalanceOf<T>,
    ) -> Option<BalanceOf<T>> {
        incentive.checked_div(total_bought_shares)
    }

    // How much incentive each market will receive
    #[allow(
        // `normalized_total` variable needs to be evaluated first
        clippy::needless_collect
    )]
    fn markets_incentives(
        per_block_incentives: BalanceOf<T>,
        curr_block: T::BlockNumber,
    ) -> Option<Vec<(T::MarketId, BalanceOf<T>)>> {
        let mut normalized_total = BalanceOf::<T>::from(0u8);
        let markets_periods: BTreeSet<_> = <BlockBoughtShares<T>>::iter().map(|el| el.0).collect();
        let now = T::MarketCommons::now();

        let normalized_values: Vec<_> = markets_periods
            .into_iter()
            .filter_map(|market_id| {
                let period = T::MarketCommons::market(&market_id).ok()?.period;
                let normalized_value = Self::normalize_market(curr_block, now, &period);
                normalized_total = normalized_total.saturating_add(normalized_value);
                Some((market_id, normalized_value))
            })
            .collect();
        normalized_values
            .into_iter()
            .map(|(market_id, normalized_value)| {
                let ptd = calculate_perthousand(normalized_value, &normalized_total)?.into();
                let incentive = calculate_perthousand_value(ptd, per_block_incentives);
                Some((market_id, incentive))
            })
            .collect()
    }

    // Takes any market parameter and outputs a number that will be used as a percentage
    // to calculate how much incentives each individual market will receive.
    //
    // In this case, the output is the percentage of the remaining number of blocks to stimulate
    // early liquidity providers. For example, a market starts at 0 and ends at 10. If the current
    // block is at 7, then the output is 30.
    //
    // Another example: A market starts at 500, ends at 700 and the current block is 510. In other
    // words, the normalized output will be near 100 (or 100%) because the current block is near
    // the start of the market.
    //
    // ```rust
    // let market_period = 500..=700
    //
    // let market_total_blocks = 700 - 500 = 200;
    // let market_remaining_blocks = 200 - 10 = 190
    //
    // let _market_normalized_value = 190 * 100 / 200 = 95
    // ```
    //
    // The greater the output, the more incentives the market will receive
    fn normalize_market(
        curr_block: T::BlockNumber,
        now: MomentOf<T>,
        period: &MarketPeriod<T::BlockNumber, MomentOf<T>>,
    ) -> BalanceOf<T> {
        let opt = match period {
            MarketPeriod::Block(range) => {
                let total_value = range.end.saturating_sub(range.start);
                calculate_perthousand(range.end.saturating_sub(curr_block), &total_value)
            }
            MarketPeriod::Timestamp(range) => {
                let total_value = range.end.saturating_sub(range.start);
                let value = now.saturating_sub(range.start);
                calculate_perthousand(value, &total_value)
            }
        };
        opt.map(|ptd| ptd.into()).unwrap_or_default()
    }
}
