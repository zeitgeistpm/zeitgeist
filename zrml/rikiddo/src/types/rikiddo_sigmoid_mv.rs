use crate::{
    constants::INITIAL_FEE,
    traits::{Lmsr, MarketAverage, RikiddoMV, Sigmoid},
};
use frame_support::dispatch::{fmt::Debug, Decode, Encode};
use substrate_fixed::{
    traits::{Fixed, FixedSigned, FixedUnsigned, FromFixed, LossyFrom, LossyInto, ToFixed},
    transcendental::ln,
    types::extra::{U119, U128, U32},
    FixedI128, FixedU32,
};

use super::TimestampedVolume;

#[derive(Clone, Debug, Decode, Encode, Eq, PartialEq)]
pub struct RikiddoConfig<FI: Fixed> {
    pub initial_fee: FI,
    max_exponent: FixedI128<U119>,
}

impl<FS: FixedSigned + LossyFrom<FixedU32<U32>> + LossyFrom<FixedI128<U119>>> RikiddoConfig<FS> {
    pub fn new(initial_fee: FS) -> Result<Self, &'static str> {
        let fu_int_bits = <FixedI128<U119>>::from_num((FS::int_nbits() - 1) as u8);
        // max exponent = ln(2^fu_int_bits) = ln(2) * fu_int_bits
        let ln_2 = if let Ok(res) =
            ln::<FixedI128<U119>, FixedI128<U119>>(<FixedI128<U119>>::from_num(2u8))
        {
            res
        } else {
            // Should never happen (as long as 128 bits are the maximum width)
            return Err("[RikiddoConfig] Error during derivation of maximum exponent");
        };
        let max_exponent = ln_2 * fu_int_bits;
        Ok(Self { initial_fee, max_exponent })
    }
}

impl<FU: FixedSigned + LossyFrom<FixedU32<U32>>> Default for RikiddoConfig<FU> {
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
    pub config: RikiddoConfig<FS>,
    pub fees: FE,
    pub ma_short: MA,
    pub ma_long: MA,
}

impl<FU, FS, FE, MA> RikiddoSigmoidMV<FU, FS, FE, MA>
where
    FU: FixedUnsigned + LossyFrom<FixedU32<U32>>,
    FS: FixedSigned + LossyFrom<FixedI128<U128>>,
    FE: Sigmoid<FIN = FS, FOUT = FU>,
    MA: MarketAverage<FU = FU>,
{
    pub fn new(config: RikiddoConfig<FS>, fees: FE, ma_short: MA, ma_long: MA) -> Self {
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

        if mal_unwrapped == FU::from_num(0u8) {
            return Err(
                "[RikiddoSigmoidMV] Zero division error during calculation: ma_short / ma_long"
            );
        }

        let ratio = if let Some(res) = mas_unwrapped.checked_div(mal_unwrapped) {
            res
        } else {
            return Err("[RikiddoSigmoidMV] Overflow during calculation: ma_short / ma_long");
        };

        // PartialOrd is bugged, therefore the workaround
        // https://github.com/encointer/substrate-fixed/issues/9
        if FS::max_value().int().to_num::<u128>() < ratio.int().to_num::<u128>() {
            return Err("[RikiddoSigmoidMV] Overflow during conversion from ma. ratio into type FS");
        }

        let integer_part_unsigned = u128::from_fixed(ratio.int());
        // We can safely cast because until here we know that the sign bit is not set
        let integer_part: FS = (integer_part_unsigned as i128).to_fixed();
        let fractional_part: FixedI128<U128> = ratio.frac().to_fixed();

        if let Some(res) = integer_part.checked_add(fractional_part.lossy_into()) {
            return self.fees.calculate(res);
        }

        // This error should be impossible to reach.
        return Err("[RikiddoSigmoidMV] Something went wrong during ratio to FS type conversion");
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
