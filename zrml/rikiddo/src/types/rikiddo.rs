use crate::{
    constants::INITIAL_FEE,
    traits::{Lmsr, MarketAverage, RikiddoMV, Sigmoid},
};
use frame_support::dispatch::{fmt::Debug, Decode, Encode};
use substrate_fixed::{FixedU32, traits::{Fixed, FixedSigned, FixedUnsigned, LossyFrom, LossyInto, ToFixed}, types::extra::U32};

use super::TimestampedVolume;

#[derive(Clone, Debug, Decode, Encode, Eq, PartialEq)]
pub struct RikiddoConfig<FI: Fixed> {
    pub initial_fee: FI,
}

impl<FU: FixedUnsigned + LossyFrom<FixedU32<U32>>> RikiddoConfig<FU> {
    pub fn new(initial_fee: FU) -> Self {
        Self { initial_fee }
    }
}

impl<FU: FixedUnsigned + LossyFrom<FixedU32<U32>>> Default for RikiddoConfig<FU> {
    fn default() -> Self {
        Self::new(INITIAL_FEE.lossy_into())
    }
}
#[derive(Clone, Debug, Decode, Default, Encode, Eq, PartialEq)]
pub struct RikiddoSigmoidMV<FU, FS, FE, MA>
where
    FU: FixedUnsigned + LossyFrom<FixedU32<U32>>,
    FS: FixedSigned,
    FE: Sigmoid<FIN = FS, FOUT = FU>,
    MA: MarketAverage<FU = FU>,
{
    pub config: RikiddoConfig<FU>,
    pub fees: FE,
    pub ma_short: MA,
    pub ma_long: MA,
}

impl<FU, FS, FE, MA> RikiddoSigmoidMV<FU, FS, FE, MA>
where
    FU: FixedUnsigned + LossyFrom<FixedU32<U32>>,
    FS: FixedSigned,
    FE: Sigmoid<FIN = FS, FOUT = FU>,
    MA: MarketAverage<FU = FU>,
{
    pub fn new(config: RikiddoConfig<FU>, fees: FE, ma_short: MA, ma_long: MA) -> Self {
        Self { config, fees, ma_short, ma_long }
    }
}

impl<FU, FS, FE, MA> Lmsr for RikiddoSigmoidMV<FU, FS, FE, MA>
where
    FU: FixedUnsigned + LossyFrom<FixedU32<U32>>,
    FS: FixedSigned,
    FE: Sigmoid<FIN = FS, FOUT = FU>,
    MA: MarketAverage<FU = FU>,
{
    type FU = FU;

    /// Return price P_i(q) for all assets in q
    fn all_prices(asset_balances: Vec<Self::FU>) -> Result<Vec<Self::FU>, &'static str> {
        Err("Unimplemented")
    }

    /// Return cost C(q) for all assets in q
    fn cost(asset_balances: Vec<Self::FU>) -> Result<Self::FU, &'static str> {
        Err("Unimplemented")
    }

    /// Return price P_i(q) for asset q_i in q
    fn price(
        asset_in_question_balance: Self::FU,
        asset_balances: Vec<Self::FU>,
    ) -> Result<Self::FU, &'static str> {
        Err("Unimplemented")
    }
}

impl<FU, FS, FE, MA> RikiddoMV for RikiddoSigmoidMV<FU, FS, FE, MA>
where
    FU: FixedUnsigned + LossyFrom<FixedU32<U32>>,
    FS: FixedSigned,
    FE: Sigmoid<FIN = FS, FOUT = FU>,
    MA: MarketAverage<FU = FU>,
{
    /// Clear market data
    fn clear(&mut self) {
        self.ma_short.clear();
        self.ma_long.clear();
    }

    /// Update market data
    /// Returns volume ratio short / long or None
    fn update(
        &mut self,
        volume: &TimestampedVolume<Self::FU>,
    ) -> Result<Option<Self::FU>, &'static str> {
        let mas = self.ma_short.update(volume)?;
        let mal = self.ma_long.update(volume)?;
        
        if let Some(mas_unwrapped) = mas {
            if let Some(mal_unwrapped) = mal {
                if mal_unwrapped != 0u32.to_fixed::<FU>() {
                    return Ok(Some(mas_unwrapped.saturating_div(mal_unwrapped)));
                }
            };
        };

        Ok(None)
    }
}
