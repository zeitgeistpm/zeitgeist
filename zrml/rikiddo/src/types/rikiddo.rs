use crate::{
    constants::{INITIAL_FEE, MINIMAL_REVENUE},
    traits::{Lmsr, MarketAverage, RikiddoMV, Sigmoid},
};
use frame_support::dispatch::{fmt::Debug, Decode, Encode};
use substrate_fixed::{
    traits::{Fixed, FixedSigned, FixedUnsigned, LossyFrom, LossyInto},
    types::extra::U32,
    FixedU32,
};

use super::TimestampedVolume;

#[derive(Clone, Debug, Decode, Encode, Eq, PartialEq)]
pub struct RikiddoConfig<F: Fixed> {
    pub initial_fee: F,
    pub minimal_revenue_coefficient: F,
}

impl<F: FixedUnsigned + LossyFrom<FixedU32<U32>>> RikiddoConfig<F> {
    pub fn new(initial_fee: F, minimal_revenue_coefficient: F) -> Self {
        Self { initial_fee, minimal_revenue_coefficient }
    }
}

impl<F: FixedUnsigned + LossyFrom<FixedU32<U32>>> Default for RikiddoConfig<F> {
    fn default() -> Self {
        Self::new(INITIAL_FEE.lossy_into(), MINIMAL_REVENUE.lossy_into())
    }
}
#[derive(Clone, Debug, Decode, Encode, Eq, PartialEq)]
pub struct RikiddoSigmoidMV<FU: FixedUnsigned, FS: FixedSigned, FE: Sigmoid, MA: MarketAverage>
where
    FU: FixedUnsigned,
    FS: FixedSigned,
    FE: Sigmoid<FIN = FS, FOUT = FU>,
    MA: MarketAverage<F = FU>,
{
    pub config: RikiddoConfig<FU>,
    pub fees: FE,
    pub ma_short: MA,
    pub ma_long: MA,
}

impl<FU, FS, FE, MA> Lmsr for RikiddoSigmoidMV<FU, FS, FE, MA>
where
    FU: FixedUnsigned,
    FS: FixedSigned,
    FE: Sigmoid<FIN = FS, FOUT = FU>,
    MA: MarketAverage<F = FU>,
{
    type F = FU;

    /// Return price P_i(q) for all assets in q
    fn all_prices(asset_balances: Vec<Self::F>) -> Result<Vec<Self::F>, &'static str> {
        Err("Unimplemented")
    }

    /// Return cost C(q) for all assets in q
    fn cost(asset_balances: Vec<Self::F>) -> Result<Self::F, &'static str> {
        Err("Unimplemented")
    }

    /// Return price P_i(q) for asset q_i in q
    fn price(
        asset_in_question_balance: Self::F,
        asset_balances: Vec<Self::F>,
    ) -> Result<Self::F, &'static str> {
        Err("Unimplemented")
    }
}

impl<FU, FS, FE, MA> RikiddoMV for RikiddoSigmoidMV<FU, FS, FE, MA>
where
    FU: FixedUnsigned,
    FS: FixedSigned,
    FE: Sigmoid<FIN = FS, FOUT = FU>,
    MA: MarketAverage<F = FU>,
{
    /// Clear market data
    fn clear(&mut self) {
        // TODO
    }

    /// Update market data
    fn update(
        &mut self,
        volume: TimestampedVolume<Self::F>,
    ) -> Result<Option<Self::F>, &'static str> {
        Err("Unimplemented")
    }
}
