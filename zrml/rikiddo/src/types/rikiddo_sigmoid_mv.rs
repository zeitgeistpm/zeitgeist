use crate::{
    constants::INITIAL_FEE,
    traits::{Lmsr, MarketAverage, RikiddoMV, Sigmoid},
};
use frame_support::dispatch::{fmt::Debug, Decode, Encode};
use substrate_fixed::{FixedI128, FixedU32, traits::{Fixed, FixedSigned, FixedUnsigned, FromFixed, LossyFrom, LossyInto, ToFixed}, types::extra::{U128, U32}};

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
    FS: FixedSigned + PartialOrd<FU> + LossyFrom<FixedI128<U128>>,
    FE: Sigmoid<FIN = FS, FOUT = FU>,
    MA: MarketAverage<FU = FU>,
{
    pub fn new(config: RikiddoConfig<FU>, fees: FE, ma_short: MA, ma_long: MA) -> Self {
        Self { config, fees, ma_short, ma_long }
    }

    pub fn get_fee(&self) -> Result<FU, &'static str> {
        let mas = self.ma_short.get();
        let mas_unwrapped = if let Some(res) = mas {
            res
        } else {
            return Ok(self.config.initial_fee);
        };

        let mal = self.ma_long.get();
        let mal_unwrapped = if let Some(res) = mal {
            res
        } else {
            return Ok(self.config.initial_fee);
        };

        if mal_unwrapped.to_num::<u128>() == 0u128 {
            return Err("[RikiddoSigmoidMV] Zero division error during calculation: ma_short / ma_long");
        }

        let ratio = if let Some(res) = mas_unwrapped.checked_div(mal_unwrapped) {
            res
        } else {
            // In most cases this is impossible (divisor always >= 1 if not 0)
            return Err("[RikiddoSigmoidMV] Overflow during calculation: ma_short / ma_long")
        };

        if FS::max_value() < ratio.int() {
            // On most configurations, this does not happen.
            return Err("[RikiddoSigmoidMV] Overflow during conversion from ratio into type FS")
        }

        let integer_part_unsigned = i128::from_fixed(ratio.int());
        // We can safely cast because until here we know that the sign bit is not set
        let integer_part: FS = (integer_part_unsigned as i128).to_fixed();
        let fractional_part: FixedI128<U128> = ratio.frac().to_fixed();

        if let Some(res) = integer_part.checked_add(fractional_part.lossy_into()) {
            return self.fees.calculate(res);
        }
        
        // This error should be impossible to reach.
        return Err("[RikiddoSigmoidMV] Something went wrong during ratio to FS type conversion.");
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
        // get fee (write helper functions) (min(w, (f+n(r))))
        // sum over every element
            // qi = current
            // sum over every element to calculate exponents (helper function)
            // Decide strategy (normal (faster) or overflow-failsafe)
            // total_result += ln(sum(e^results))
        // total_result *= fee
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
