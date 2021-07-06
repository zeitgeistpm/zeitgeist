use sp_std::marker::PhantomData;

use crate::{
    constants::{INITIAL_FEE, MINIMAL_REVENUE},
    traits::{MarketAverage, Sigmoid},
};
use frame_support::dispatch::{fmt::Debug, Decode, Encode};
use substrate_fixed::{
    traits::{Fixed, FixedSigned, FixedUnsigned, LossyFrom, LossyInto},
    types::extra::U32,
    FixedU32,
};

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
pub struct RikiddoSigmoidMV<FS: FixedSigned, FU: FixedUnsigned, FE: Sigmoid, MA: MarketAverage> {
    pub config: RikiddoConfig<FU>,
    pub fees: FE,
    pub ma_short: MA,
    pub ma_long: MA,
    pub _market: PhantomData<FS>,
}
