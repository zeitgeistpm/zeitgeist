use crate::{
    constants::INITIAL_FEE,
    traits::{Lmsr, MarketAverage, RikiddoMV, Sigmoid},
};
use frame_support::dispatch::{fmt::Debug, Decode, Encode};
use substrate_fixed::{
    traits::{Fixed, FixedSigned, FixedUnsigned, FromFixed, LossyFrom, LossyInto, ToFixed},
    transcendental::ln,
    types::extra::{U119, U127, U128, U31, U32},
    FixedI128, FixedI32, FixedU128, FixedU32,
};

use super::{convert_to_signed, TimestampedVolume};

#[derive(Clone, Debug, Decode, Encode, Eq, PartialEq)]
pub struct RikiddoConfig<FI: Fixed> {
    pub initial_fee: FI,
    max_exponent: FixedI128<U119>,
}

impl<FS: FixedSigned + LossyFrom<FixedI32<U31>> + LossyFrom<FixedI128<U119>>> RikiddoConfig<FS> {
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

// TODO: test
impl<FS: FixedSigned + LossyFrom<FixedI32<U31>> + LossyFrom<FixedI128<U119>>> Default
    for RikiddoConfig<FS>
{
    fn default() -> Self {
        let converted = convert_to_signed::<FixedU32<U32>, FixedI32<U31>>(INITIAL_FEE).unwrap();
        // Potentially dangerous unwrap(), should be impossible to fail (tested).
        Self::new(converted.lossy_into()).unwrap()
    }
}
#[derive(Clone, Debug, Decode, Default, Encode, Eq, PartialEq)]
pub struct RikiddoSigmoidMV<FU, FS, FE, MA>
where
    FU: FixedUnsigned + LossyFrom<FixedU32<U32>>,
    FS: FixedSigned + LossyFrom<FixedI32<U31>> + LossyFrom<FixedI128<U119>>,
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
    FU: FixedUnsigned + LossyFrom<FixedU32<U32>> + LossyFrom<FixedU128<U128>>,
    FS: FixedSigned
        + LossyFrom<FixedI32<U31>>
        + LossyFrom<FixedI128<U119>>
        + LossyFrom<FixedI128<U127>>,
    FE: Sigmoid<FIN = FS, FOUT = FU>,
    MA: MarketAverage<FU = FU>,
{
    pub fn new(config: RikiddoConfig<FS>, fees: FE, ma_short: MA, ma_long: MA) -> Self {
        Self { config, fees, ma_short, ma_long }
    }

    pub fn get_fee(&self) -> Result<FS, &'static str> {
        let mas = if let Some(res) = self.ma_short.get() {
            res
        } else {
            return Ok(self.config.initial_fee);
        };

        let mal = if let Some(res) = self.ma_long.get() {
            res
        } else {
            return Ok(self.config.initial_fee);
        };

        if mal == FU::from_num(0u8) {
            return Err(
                "[RikiddoSigmoidMV] Zero division error during calculation: ma_short / ma_long"
            );
        }

        let ratio = if let Some(res) = mas.checked_div(mal) {
            res
        } else {
            return Err("[RikiddoSigmoidMV] Overflow during calculation: ma_short / ma_long");
        };

        let ratio_signed = convert_to_signed(ratio)?;
        convert_to_signed(self.fees.calculate(ratio_signed)?)
    }
}

impl<FU, FS, FE, MA> Lmsr for RikiddoSigmoidMV<FU, FS, FE, MA>
where
    FU: FixedUnsigned + LossyFrom<FixedU32<U32>>,
    FS: FixedSigned + LossyFrom<FixedI32<U31>> + LossyFrom<FixedI128<U119>>,
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
    FS: FixedSigned + LossyFrom<FixedI32<U31>> + LossyFrom<FixedI128<U119>>,
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

        if let Some(mas) = mas {
            if let Some(mal) = mal {
                if mal != 0u32.to_fixed::<FU>() {
                    return Ok(Some(mas.saturating_div(mal)));
                }
            };
        };

        Ok(None)
    }
}
