use crate::{
    constants::INITIAL_FEE,
    traits::{Lmsr, MarketAverage, RikiddoMV, Sigmoid},
};
use frame_support::dispatch::{fmt::Debug, Decode, Encode};
use core::ops::{AddAssign, BitOrAssign, ShlAssign};
use substrate_fixed::{FixedI128, FixedI32, FixedU128, FixedU32, consts::LOG2_E, traits::{Fixed, FixedSigned, FixedUnsigned, LossyFrom, LossyInto, ToFixed}, transcendental::{ln, log2}, types::{I9F23, U1F127, extra::{U119, U127, U128, U31, U32}}};

use super::{convert_to_signed, convert_to_unsigned, TimestampedVolume};

#[derive(Clone, Debug, Decode, Encode, Eq, PartialEq)]
pub struct RikiddoConfig<FI: Fixed> {
    pub initial_fee: FI,
    log2_e: FI,
}

impl<FS: FixedSigned + LossyFrom<FixedI32<U31>> + LossyFrom<U1F127>> RikiddoConfig<FS> {
    pub fn new(initial_fee: FS) -> Result<Self, &'static str> {
        Ok(Self { initial_fee, log2_e: FS::lossy_from(LOG2_E) })
    }
}

// TODO: test
impl<FS: FixedSigned + LossyFrom<FixedI32<U31>> + LossyFrom<U1F127>> Default for RikiddoConfig<FS> {
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
    FS: FixedSigned + LossyFrom<FixedI32<U31>> + LossyFrom<U1F127>,
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
    FS: FixedSigned + LossyFrom<FixedI32<U31>> + LossyFrom<U1F127> + LossyFrom<FixedI128<U127>>,
    FE: Sigmoid<FIN = FS, FOUT = FU>,
    MA: MarketAverage<FU = FU>,
{
    pub fn new(config: RikiddoConfig<FS>, fees: FE, ma_short: MA, ma_long: MA) -> Self {
        Self { config, fees, ma_short, ma_long }
    }

    pub fn get_fee(&self) -> Result<FU, &'static str> {
        let mas = if let Some(res) = self.ma_short.get() {
            res
        } else {
            return convert_to_unsigned(self.config.initial_fee);
        };

        let mal = if let Some(res) = self.ma_long.get() {
            res
        } else {
            return convert_to_unsigned(self.config.initial_fee);
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
        self.fees.calculate(ratio_signed)
    }
}

impl<FU, FS, FE, MA> Lmsr for RikiddoSigmoidMV<FU, FS, FE, MA>
where
    FU: FixedUnsigned + LossyFrom<FixedU32<U32>> + LossyFrom<FixedU128<U128>>,
    FS: FixedSigned + LossyFrom<FixedI32<U31>> + LossyFrom<U1F127> + LossyFrom<FixedI128<U127>> + PartialOrd<I9F23>,
    FS::Bits: Copy + ToFixed + AddAssign + BitOrAssign + ShlAssign, 
    FE: Sigmoid<FIN = FS, FOUT = FU>,
    MA: MarketAverage<FU = FU>,
{
    type FU = FU;

    /// Return price P_i(q) for all assets in q
    fn all_prices(&self, asset_balances: Vec<Self::FU>) -> Result<Vec<Self::FU>, &'static str> {
        Err("Unimplemented")
    }

    /// Return cost C(q) for all assets in q
    fn cost(&self, asset_balances: Vec<Self::FU>) -> Result<Self::FU, &'static str> {
        if asset_balances.len() == 0 {
            return Err("[RikiddoSigmoidMV] No asset balances provided");
        };

        // get fee (write helper functions) (min(w, (f+n(r))))
        let fee = self.get_fee()?;
        let mut total_balance = FU::from_num(0u8);

        for elem in &asset_balances {
            if let Some(res) = total_balance.checked_add(*elem) {
                total_balance = res;
            } else {
                return Err("[RikiddoSigmoidMV] Overflow during summation of asset balances");
            }
        }

        let denominator = if let Some(res) = fee.checked_mul(total_balance) {
            res
        } else {
            // Highly unlikely and only possible if fee > 100%
            return Err("[RikiddoSigmoidMV] Overflow during calculation: fee * total_asset_balance");
        };

        let mut exponents: Vec<FS> = Vec::with_capacity(asset_balances.len());
        let mut biggest_exponent: FS = FS::from_num(0u8);

        for elem in &asset_balances {
            let exponent = if let Some(res) = elem.checked_div(denominator) {
                convert_to_signed::<FU, FS>(res)?
            } else {
                // Highly unlikely
                return Err("[RikiddoSigmoidMV] Overflow during calculation: expontent_i = \
                            asset_balance_i / denominator");
            };

            if exponent > biggest_exponent {
                biggest_exponent = exponent;
            }

            // Panic impossible
            exponents.push(exponent);
        }

        // Determine which strategy to use.
        let biggest_exp_times_log2e = if let Some(res) = self.config.log2_e.checked_mul(biggest_exponent) {
            res
        } else {
            // Highly unlikely
            return Err("[RikiddoSigmoidMV] Overflow during calculation: log2_e * biggest_exponent");
        };

        if FS::max_value().int().to_num::<u128>() < asset_balances.len() as u128 {
            return Err("[RikidoSigmoidMV] Number of assets does not fit in FS")
        }

        let log2e_number_of_assets: FS = if let Ok(res) = log2::<FS, FS>(FS::from_num(asset_balances.len())) {
            res
        } else {
            // Impossible, since the cost functions checks if elements are present in asset_balances
            return Err("[RikiddoSigmoidMV] log2(number_of_assets), number_of_assets <= 0");
        };
        
        //let required_bits = if let Some(res) = biggest_exp_times_log2e.checked_add()
        // sum over every element
        // qi = current
        // sum over every element to calculate exponents (helper function)
        // Decide strategy (normal (faster) or overflow-failsafe)
        // total_result += ln(sum(e^results))
        // sum end: total_result *= fee
        Err("Unimplemented")
    }

    /// Return price P_i(q) for asset q_i in q
    fn price(
        &self,
        asset_in_question_balance: Self::FU,
        asset_balances: Vec<Self::FU>,
    ) -> Result<Self::FU, &'static str> {
        Err("Unimplemented")
    }
}

/*
impl<FU, FS, FE, MA> RikiddoMV for RikiddoSigmoidMV<FU, FS, FE, MA>
where
    FU: FixedUnsigned + LossyFrom<FixedU32<U32>> + LossyFrom<FixedU128<U128>>,
    FS: FixedSigned + LossyFrom<FixedI32<U31>> + LossyFrom<U1F127> + LossyFrom<FixedI128<U127>>,
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
*/