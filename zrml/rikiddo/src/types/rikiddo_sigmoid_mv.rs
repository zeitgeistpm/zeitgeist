//! This module offers the Rikiddo core functionality. The implementation is modular in regards
//! to the caluclation of the fee and the evaluation of the collected market volumes.

extern crate alloc;
use crate::{
    constants::INITIAL_FEE,
    traits::{Fee, Lmsr, MarketAverage, RikiddoMV},
    utils::{convert_to_signed, convert_to_unsigned, fixed_zero, max_value_u128},
};
use alloc::vec::Vec;
#[cfg(feature = "arbitrary")]
use arbitrary::{Arbitrary, Result as ArbiraryResult, Unstructured};
use parity_scale_codec::MaxEncodedLen;
use scale_info::TypeInfo;
use core::ops::{AddAssign, BitOrAssign, ShlAssign};
use frame_support::dispatch::{Decode, Encode};
use hashbrown::HashMap;
use sp_core::RuntimeDebug;
use substrate_fixed::{
    consts::LOG2_E,
    traits::{Fixed, FixedSigned, FixedUnsigned, LossyFrom, LossyInto, ToFixed},
    transcendental::{exp, ln},
    types::{
        extra::{U127, U128, U31, U32},
        I9F23, U1F127,
    },
    FixedI128, FixedI32, FixedU128, FixedU32,
};
#[cfg(feature = "arbitrary")]
use substrate_fixed::{
    types::extra::{LeEqU128, LeEqU32, LeEqU64},
    FixedI64, FixedU64,
};

use super::TimestampedVolume;

/// Configuration values used within the Rikiddo core functions.
#[derive(Clone, Decode, Encode, Eq, MaxEncodedLen, PartialEq, RuntimeDebug, TypeInfo)]
pub struct RikiddoConfig<FI: Fixed> {
    /// An initial fee that is used whenever the fee cannot be calculated.
    pub initial_fee: FI,
    pub(crate) log2_e: FI,
}

#[cfg(feature = "arbitrary")]
macro_rules! impl_arbitrary_for_rikiddo_config {
    ( $t:ident, $LeEqU:ident, $p:ty ) => {
        impl<'a, Frac> Arbitrary<'a> for RikiddoConfig<$t<Frac>>
        where
            Frac: $LeEqU,
            $t<Frac>: FixedSigned + LossyFrom<FixedI32<U31>> + LossyFrom<U1F127>,
        {
            fn arbitrary(u: &mut Unstructured<'a>) -> ArbiraryResult<Self> {
                Ok(RikiddoConfig::<$t<Frac>>::new(<$t<Frac>>::from_bits(
                    <$p as Arbitrary<'a>>::arbitrary(u)?,
                )))
            }

            #[inline]
            fn size_hint(depth: usize) -> (usize, Option<usize>) {
                <$p as Arbitrary<'a>>::size_hint(depth)
            }
        }
    };
}

cfg_if::cfg_if! {
    if #[cfg(feature = "arbitrary")] {
        impl_arbitrary_for_rikiddo_config! {FixedI32, LeEqU32, i32}
        impl_arbitrary_for_rikiddo_config! {FixedI64, LeEqU64, i64}
        impl_arbitrary_for_rikiddo_config! {FixedI128, LeEqU128, i128}
    }
}

impl<FS: FixedSigned + LossyFrom<FixedI32<U31>> + LossyFrom<U1F127>> RikiddoConfig<FS> {
    /// Create a new `RikiddoConfig` instance based on a [`RikiddoConfig`](struct@RikiddoConfig)
    /// configuration. Use `default()` if uncertain which values to use.
    ///
    /// # Arguments
    ///
    /// * See [`RikiddoConfig`](struct@RikiddoConfig).
    pub fn new(initial_fee: FS) -> Self {
        Self { initial_fee, log2_e: FS::lossy_from(LOG2_E) }
    }
}

impl<FS: FixedSigned + LossyFrom<FixedI32<U31>> + LossyFrom<U1F127>> Default for RikiddoConfig<FS> {
    fn default() -> Self {
        // Potentially dangerous unwrap(), should be impossible to fail (tested).
        let converted = convert_to_signed::<FixedU32<U32>, FixedI32<U31>>(INITIAL_FEE).unwrap();
        Self::new(converted.lossy_into())
    }
}

// The RikiddoFormulaComponents contain all necessary duplicate information
// thorughout multiple calculations.
pub(crate) struct RikiddoFormulaComponents<FS>
where
    FS: FixedSigned + From<I9F23> + LossyFrom<FixedI32<U31>> + LossyFrom<U1F127>,
{
    pub(crate) one: FS,
    pub(crate) fee: FS,
    pub(crate) sum_balances: FS,
    pub(crate) sum_times_fee: FS,
    pub(crate) emax: FS,
    pub(crate) sum_exp: FS,
    pub(crate) exponents: HashMap<FS, FS>,
    pub(crate) reduced_exponential_results: HashMap<FS, FS>,
}

impl<FS> Default for RikiddoFormulaComponents<FS>
where
    FS: FixedSigned + From<I9F23> + From<i8> + LossyFrom<FixedI32<U31>> + LossyFrom<U1F127>,
{
    fn default() -> Self {
        Self {
            one: 1i8.into(),
            fee: 0i8.into(),
            sum_balances: 0i8.into(),
            sum_times_fee: 0i8.into(),
            emax: 0i8.into(),
            sum_exp: 0i8.into(),
            exponents: HashMap::new(),
            reduced_exponential_results: HashMap::new(),
        }
    }
}

/// Configuration values used within the Rikiddo core functions.
#[derive(Clone, Decode, Default, Encode, Eq, MaxEncodedLen, PartialEq, RuntimeDebug, TypeInfo)]
pub struct RikiddoSigmoidMV<FU, FS, FE, MA>
where
    FU: FixedUnsigned + LossyFrom<FixedU32<U32>>,
    FS: FixedSigned + LossyFrom<FixedI32<U31>> + LossyFrom<U1F127>,
    FE: Fee<FS = FS>,
    MA: MarketAverage<FU = FU>,
{
    /// See [`RikiddoConfig`](struct@RikiddoConfig).
    pub config: RikiddoConfig<FS>,
    /// A structure that implements the [`Fee`](trait@Fee) trait. Used to calculate the fee.
    pub fees: FE,
    /// A structure that implements the [`MarketAverage`](trait@MarketAverage) trait. It is used
    /// to calculate the ema for the shorter ema period.
    pub ma_short: MA,
    /// A structure that implements the [`MarketAverage`](trait@MarketAverage) trait. It is used
    /// to calculate the ema for the longer ema period.
    pub ma_long: MA,
}

#[cfg(feature = "arbitrary")]
macro_rules! impl_arbitrary_for_rikiddo_sigmoid_mv {
    ( $ts:ident, $LeEqUs:ident, $tu:ident, $LeEqUu:ident ) => {
        impl<'a, FracU, FracS, FE, MA> Arbitrary<'a>
            for RikiddoSigmoidMV<$tu<FracU>, $ts<FracS>, FE, MA>
        where
            FracU: $LeEqUu,
            FracS: $LeEqUs,
            $tu<FracU>: FixedUnsigned + LossyFrom<FixedU32<U32>> + LossyFrom<FixedU128<U128>>,
            $ts<FracS>: FixedSigned
                + From<I9F23>
                + From<i8>
                + LossyFrom<FixedI32<U31>>
                + LossyFrom<U1F127>
                + LossyFrom<FixedI128<U127>>
                + PartialOrd<I9F23>,
            <$ts<FracS> as Fixed>::Bits: Copy + ToFixed + AddAssign + BitOrAssign + ShlAssign,
            FE: Fee<FS = $ts<FracS>> + Arbitrary<'a>,
            MA: MarketAverage<FU = $tu<FracU>> + Arbitrary<'a>,
        {
            fn arbitrary(u: &mut Unstructured<'a>) -> ArbiraryResult<Self> {
                Ok(RikiddoSigmoidMV::new(
                    <RikiddoConfig<$ts<FracS>> as Arbitrary<'a>>::arbitrary(u)?,
                    <FE as Arbitrary<'a>>::arbitrary(u)?,
                    <MA as Arbitrary<'a>>::arbitrary(u)?,
                    <MA as Arbitrary<'a>>::arbitrary(u)?,
                ))
            }

            #[inline]
            fn size_hint(depth: usize) -> (usize, Option<usize>) {
                let (min, max) = <RikiddoConfig<$ts<FracS>> as Arbitrary<'a>>::size_hint(depth);

                let fe_size = <FE as Arbitrary<'a>>::size_hint(depth);
                let ma_size = <MA as Arbitrary<'a>>::size_hint(depth);

                let max_accumulated = max
                    .unwrap_or(0)
                    .saturating_add(fe_size.1.unwrap_or(fe_size.0))
                    .saturating_add(ma_size.1.unwrap_or(ma_size.0).saturating_mul(2));
                let min_accumulated = min.saturating_add(fe_size.0).saturating_add(ma_size.0);

                if max_accumulated == usize::MAX {
                    (min_accumulated, None)
                } else {
                    (min_accumulated, Some(max_accumulated))
                }
            }
        }
    };
}

cfg_if::cfg_if! {
    if #[cfg(feature = "arbitrary")] {
        impl_arbitrary_for_rikiddo_sigmoid_mv! {FixedI32, LeEqU32, FixedU32, LeEqU32}
        impl_arbitrary_for_rikiddo_sigmoid_mv! {FixedI64, LeEqU64, FixedU64, LeEqU64}
        impl_arbitrary_for_rikiddo_sigmoid_mv! {FixedI128, LeEqU128, FixedU128, LeEqU128}
    }
}

impl<FU, FS, FE, MA> RikiddoSigmoidMV<FU, FS, FE, MA>
where
    FU: FixedUnsigned + LossyFrom<FixedU32<U32>> + LossyFrom<FixedU128<U128>>,
    FS: FixedSigned
        + From<I9F23>
        + From<i8>
        + LossyFrom<FixedI32<U31>>
        + LossyFrom<U1F127>
        + LossyFrom<FixedI128<U127>>
        + PartialOrd<I9F23>,
    FS::Bits: Copy + ToFixed + AddAssign + BitOrAssign + ShlAssign,
    FE: Fee<FS = FS>,
    MA: MarketAverage<FU = FU>,
{
    /// Initialize the structure based on a configuration.
    ///
    /// # Arguments
    ///
    /// * See [`RikiddoSigmoidMV`](struct@RikiddoSigmoidMV)
    pub fn new(config: RikiddoConfig<FS>, fees: FE, ma_short: MA, ma_long: MA) -> Self {
        Self { config, fees, ma_short, ma_long }
    }

    // Cost function that returns the cost and parts of the formula that can be reused for the
    // price calculation.
    // Setting `for_price = true` returns the cost minus the sum of quantities.
    // Setting `add_exponents = true` leads to saving the exponents and the reduced exponential
    // results into the `formula_components` struct.
    pub(crate) fn cost_with_forumla(
        &self,
        asset_balances: &[FU],
        formula_components: &mut RikiddoFormulaComponents<FS>,
        for_price: bool,
        add_exponents: bool,
    ) -> Result<FS, &'static str> {
        if asset_balances.is_empty() {
            return Err("[RikiddoSigmoidMV] No asset balances provided");
        };

        let fee = self.fee()?;
        formula_components.fee = convert_to_signed(fee)?;
        let mut total_balance = fixed_zero::<FU>()?;

        for elem in asset_balances {
            if let Some(res) = total_balance.checked_add(*elem) {
                total_balance = res;
            } else {
                return Err("[RikiddoSigmoidMV] Overflow during summation of asset balances");
            }
        }

        formula_components.sum_balances = convert_to_signed(total_balance)?;

        let denominator = if let Some(res) = fee.checked_mul(total_balance) {
            res
        } else {
            // Highly unlikely and only possible if fee > 100%
            return Err("[RikiddoSigmoidMV] Overflow during calculation: fee * total_asset_balance");
        };

        formula_components.sum_times_fee = convert_to_signed(denominator)?;

        let mut exponents: Vec<FS> = Vec::with_capacity(asset_balances.len());
        let mut biggest_exponent: FS = fixed_zero::<FS>()?;

        for elem in asset_balances {
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

            if add_exponents {
                let elem_fs = convert_to_signed(*elem)?;
                formula_components.exponents.insert(elem_fs, exponent);
            }
        }

        formula_components.emax = biggest_exponent;

        if max_value_u128::<FS>()? < asset_balances.len() as u128 {
            return Err("[RikidoSigmoidMV] Number of assets does not fit in FS");
        }

        let ln_sum_e = self.optimized_cost_strategy(
            &exponents,
            &biggest_exponent,
            formula_components,
            add_exponents,
        )?;

        if for_price {
            if let Some(res) = formula_components.fee.checked_mul(ln_sum_e) {
                return Ok(res);
            }
            Err("[RikiddoSigmoidMV] Overflow during calculation: fee * ln(sum_i(e^i))")
        } else {
            if let Some(res) = formula_components.sum_times_fee.checked_mul(ln_sum_e) {
                return Ok(res);
            }

            Err("[RikiddoSigmoidMV] Overflow during calculation: fee * total_asset_balance * \
                 ln(sum_i(e^i))")
        }
    }

    // Calculates the first quotient in the price formula after the cost / sum_balances
    pub(crate) fn price_helper_first_quotient(
        &self,
        asset_balances: &[FS],
        asset_in_question_balance: &FS,
        formula_components: &RikiddoFormulaComponents<FS>,
    ) -> Result<FS, &'static str> {
        let exponent_of_balance_in_question =
            if let Some(res) = formula_components.exponents.get(asset_in_question_balance) {
                *res
            } else {
                return Err("[RikiddoSigmoidMV] Cannot find exponent of asset balance in \
                            question in RikiddoFormulaComponents HashMap");
            };

        let mut sum = fixed_zero::<FS>()?;
        let mut skipped = false;

        for elem in asset_balances {
            if elem == asset_in_question_balance && !skipped {
                skipped = true;
                continue;
            }

            let elem_div_sum_fee = if let Some(res) = formula_components.exponents.get(elem) {
                *res
            } else {
                return Err("[RikiddoSigmoidMV] Cannot find exponent of asset balance in \
                            question RikiddoFormulaComponents HashMap");
            };

            let exponent =
                if let Some(res) = elem_div_sum_fee.checked_sub(exponent_of_balance_in_question) {
                    res
                } else {
                    // Should be impossible (negative exponent not possible unless manually entered)
                    return Err("[RikiddoSigmoidMV] Overflow during calculation: exponent - \
                                exponent_balance_in_question");
                };

            let exponential_result = if let Ok(res) = exp::<FS, FS>(exponent) {
                res
            } else {
                // In that case the final result will not fit into the fractional bits
                // and therefore is approximated to zero. Cannot panic.
                fixed_zero::<FS>()?
            };

            sum = if let Some(res) = sum.checked_add(exponential_result) {
                res
            } else {
                // In that case the final result will not fit into the fractional bits
                // and therefore is approximated to zero. Cannot panic.
                return fixed_zero::<FS>();
            };
        }

        sum = if let Some(res) = sum.checked_add(formula_components.one) {
            res
        } else {
            // In that case the final result will not fit into the fractional bits
            // and therefore is approximated to zero. Cannot panic.
            return fixed_zero::<FS>();
        };

        if let Some(res) = formula_components.one.checked_div(sum) {
            Ok(res)
        } else {
            fixed_zero::<FS>()
        }
    }

    // Calculates the second quotient in the price formula after the cost / sum_balances
    pub(crate) fn price_helper_second_quotient(
        &self,
        asset_balances: &[FS],
        formula_components: &RikiddoFormulaComponents<FS>,
    ) -> Result<FS, &'static str> {
        let mut numerator = fixed_zero::<FS>()?;
        let mut skipped = false;

        for elem in asset_balances {
            let exponent_of_balance_in_question =
                if let Some(res) = formula_components.exponents.get(elem) {
                    *res
                } else {
                    return Err("[RikiddoSigmoidMV] Cannot find exponent of asset balance in \
                                question in RikiddoFormulaComponents HashMap");
                };

            let elem_times_reduced_exponential_result;

            if exponent_of_balance_in_question == formula_components.emax && !skipped {
                elem_times_reduced_exponential_result = *elem;
                skipped = true;
            } else {
                let exponential_result = if let Some(res) = formula_components
                    .reduced_exponential_results
                    .get(&exponent_of_balance_in_question)
                {
                    *res
                } else {
                    return Err("[RikiddoSigmoidMV] Cannot find reduced exponential result of \
                                current element");
                };

                elem_times_reduced_exponential_result =
                    if let Some(res) = elem.checked_mul(exponential_result) {
                        res
                    } else {
                        // Should be impossible, since reduced_exponential_result ∈ ]0, 1]
                        return Err("[RikiddoSigmoidMV] Overflow during calculation: element * \
                                    reduced_exponential_result");
                    };
            }

            numerator =
                if let Some(res) = numerator.checked_add(elem_times_reduced_exponential_result) {
                    res
                } else {
                    return Err("[RikiddoSigmoidMV] Overflow during calculation: sum_j += \
                                elem_times_reduced_exponential_result");
                };
        }

        let denominator = if let Some(res) =
            formula_components.sum_balances.checked_mul(formula_components.sum_exp)
        {
            res
        } else {
            return Err("[RikiddoSigmoidMV] Overflow during calculation: sum_balances * sum_exp");
        };

        if let Some(res) = numerator.checked_div(denominator) {
            Ok(res)
        } else {
            Err("[RikiddoSigmoidMV] Overflow during calculation (price helper 2): numerator / \
                 denominator")
        }
    }

    // Calculates the price.
    pub(crate) fn price_helper_combine_all_parts(
        &self,
        cost_part: FS,
        first_quotient: FS,
        second_quotient: FS,
    ) -> Result<FU, &'static str> {
        let quotient_sub = if let Some(res) = first_quotient.checked_sub(second_quotient) {
            res
        } else {
            // Should be impossible
            return Err("[RikiddoSigmoidMV] Overflow during calculation of price: first_quotient \
                        - second_quotient");
        };

        if let Some(res) = cost_part.checked_add(quotient_sub) {
            convert_to_unsigned(res)
        } else {
            // Should be impossible
            Err("[RikiddoSigmoidMV] Overflow during calculation of price: cost_part + quotient_sub")
        }
    }

    pub(crate) fn optimized_cost_strategy(
        &self,
        exponents: &[FS],
        biggest_exponent: &FS,
        formula_components: &mut RikiddoFormulaComponents<FS>,
        add_reduced_exponential_results: bool,
    ) -> Result<FS, &'static str> {
        let mut biggest_exponent_used = false;

        if max_value_u128::<FS>()? < 1u128 {
            // Impossible due to trait bounds (at least 1 sign bit and 8 integer bits)
            return Err("[RikiddoSigmoidMV] Error, cannot initialize FS with one");
        }

        let mut exp_sum = formula_components.one;
        let result = *biggest_exponent;

        for elem in exponents {
            if !biggest_exponent_used && elem == biggest_exponent {
                biggest_exponent_used = true;
                continue;
            }

            let exponent = if let Some(res) = elem.checked_sub(*biggest_exponent) {
                res
            } else {
                // Should be impossible
                return Err("[RikiddoSigmoidFee] Overflow during calculation: current_exponent - \
                            biggest_exponent");
            };

            let e_power_exponent = if let Ok(res) = exp::<FS, FS>(exponent) {
                res
            } else {
                // In this case the result is zero (or is too small to fit) and can be ignored
                continue;
            };

            if add_reduced_exponential_results {
                formula_components.reduced_exponential_results.insert(*elem, e_power_exponent);
            }

            if let Some(res) = exp_sum.checked_add(e_power_exponent) {
                exp_sum = res;
            } else {
                // Highly unlikely
                return Err("[RikiddoSigmoidFee] Overflow during calculation: sum_i(e^(i - \
                            biggest_exponent))");
            };
        }

        formula_components.sum_exp = exp_sum;

        let ln_exp_sum = if let Ok(res) = ln::<FS, FS>(exp_sum) {
            res
        } else {
            // Impossible
            return Err("[RikiddoSigmoidMV] ln(exp_sum) (optimized), exp_sum <= 0");
        };

        if let Some(res) = result.checked_add(ln_exp_sum) {
            Ok(res)
        } else {
            // Highly unlikely
            Err("[RikiddoSigmoidMV] Overflow during calculation: biggest_exponent + ln(exp_sum) \
                 (optimized)")
        }
    }
}

impl<FU, FS, FE, MA> Lmsr for RikiddoSigmoidMV<FU, FS, FE, MA>
where
    FU: FixedUnsigned + LossyFrom<FixedU32<U32>> + LossyFrom<FixedU128<U128>>,
    FS: FixedSigned
        + From<I9F23>
        + From<i8>
        + LossyFrom<FixedI32<U31>>
        + LossyFrom<U1F127>
        + LossyFrom<FixedI128<U127>>
        + PartialOrd<I9F23>,
    FS::Bits: Copy + ToFixed + AddAssign + BitOrAssign + ShlAssign,
    FE: Fee<FS = FS>,
    MA: MarketAverage<FU = FU>,
{
    type FU = FU;

    /// Returns a vector of prices for a given set of assets (same order as `asset_balances`).
    /// This function is significantly faster compared to sequentially invoking `price(...)` for
    /// every asset.
    ///
    /// # Arguments
    ///
    /// * `asset_balances`: The balance vector of the assets.
    fn all_prices(&self, asset_balances: &[Self::FU]) -> Result<Vec<Self::FU>, &'static str> {
        let mut formula_components = RikiddoFormulaComponents::default();
        let mut asset_balances_signed = Vec::with_capacity(asset_balances.len());

        for asset_balance in asset_balances {
            let signed_balance = convert_to_signed(*asset_balance)?;
            asset_balances_signed.push(signed_balance);
        }

        let cost_part =
            self.cost_with_forumla(asset_balances, &mut formula_components, true, true)?;

        let mut result = Vec::with_capacity(asset_balances.len());

        for asset_balance in &asset_balances_signed {
            let first_quotient = self.price_helper_first_quotient(
                &asset_balances_signed,
                asset_balance,
                &formula_components,
            )?;
            let second_quotient =
                self.price_helper_second_quotient(&asset_balances_signed, &formula_components)?;
            result.push(self.price_helper_combine_all_parts(
                cost_part,
                first_quotient,
                second_quotient,
            )?);
        }

        Ok(result)
    }

    /// Returns the total cost for a specific vector of assets (see [LS-LMSR paper]).
    ///
    /// [LS-LMSR paper]: https://www.eecs.harvard.edu/cs286r/courses/fall12/papers/OPRS10.pdf
    ///
    /// # Arguments
    ///
    /// * `asset_balances`: The balance vector of the assets.
    fn cost(&self, asset_balances: &[Self::FU]) -> Result<Self::FU, &'static str> {
        convert_to_unsigned(self.cost_with_forumla(
            asset_balances,
            &mut Default::default(),
            false,
            false,
        )?)
    }

    /// Returns the current fee.
    fn fee(&self) -> Result<Self::FU, &'static str> {
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

        if mal == fixed_zero::<FU>()? {
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
        convert_to_unsigned::<FS, FU>(self.fees.calculate_fee(ratio_signed)?)
    }

    /// Returns the initial quantities of outstanding event outcome assets.
    /// If 4 event outcome assets exist and this function returns 100, then the outstanding
    /// shares for every single of those event outcome assets are 100.
    ///
    /// # Arguments
    ///
    /// * `num_assets`: The number of distinct outcome events.
    /// * `subsidy`: The initial total subsidy gathered.
    fn initial_outstanding_assets(
        &self,
        num_assets: u32,
        subsidy: Self::FU,
    ) -> Result<Self::FU, &'static str> {
        let fee = self.fees.minimum_fee();
        let conversion_error = "[RikidoSigmoidMV] Number of assets does not fit in FU";
        let fee_overflow = "[RikidoSigmoidMV] Overflow during calculation: fee * num_assets";
        let ln_error = "[RikiddoSigmoidMV] ln(num_assets) failed";
        let denom_error =
            "[RikidoSigmoidMV] Overflow during calculation: fee * num_assets * ln(num_assets) + 1";
        let laste = "[RikidoSigmoidMV] Overflow during calculation: numerator / denominator";
        let num_assets_fs = num_assets.checked_to_fixed().ok_or(conversion_error)?;
        let fee_times_num_assets = fee.checked_mul(num_assets_fs).ok_or(fee_overflow)?;
        // This should not fail
        let ln_num_assets: FS = ln(num_assets_fs).map_err(|_| ln_error)?;
        let denominator = fee_times_num_assets
            .checked_mul(ln_num_assets)
            .ok_or(denom_error)?
            .checked_add(1.checked_to_fixed().ok_or(conversion_error)?)
            .ok_or(denom_error)?;
        convert_to_unsigned(
            convert_to_signed::<Self::FU, FS>(subsidy)?.checked_div(denominator).ok_or(laste)?,
        )
    }

    /// Returns the price of one specific asset.
    ///
    /// # Arguments
    ///
    /// * `asset_in_question`: The balance of the asset for which the price should be returned.
    /// * `asset_balances`: The balance vector of the assets.
    fn price(
        &self,
        asset_balances: &[Self::FU],
        asset_in_question_balance: &Self::FU,
    ) -> Result<Self::FU, &'static str> {
        let mut formula_components = RikiddoFormulaComponents::default();
        let mut asset_balances_signed = Vec::with_capacity(asset_balances.len());
        let mut asset_in_question_balance_signed = fixed_zero()?;
        let mut asset_in_question_found = false;

        for asset_balance in asset_balances {
            let signed_balance = convert_to_signed(*asset_balance)?;
            asset_balances_signed.push(signed_balance);

            if asset_balance == asset_in_question_balance {
                asset_in_question_balance_signed = signed_balance;
                asset_in_question_found = true;
            }
        }

        if !asset_in_question_found {
            return Err("[RikiddoSigmoidMV] asset_in_question_balance not found in asset_balances");
        }

        let cost_part =
            self.cost_with_forumla(asset_balances, &mut formula_components, true, true)?;
        let first_quotient = self.price_helper_first_quotient(
            &asset_balances_signed,
            &asset_in_question_balance_signed,
            &formula_components,
        )?;
        let second_quotient =
            self.price_helper_second_quotient(&asset_balances_signed, &formula_components)?;
        self.price_helper_combine_all_parts(cost_part, first_quotient, second_quotient)
    }
}

impl<FU, FS, FE, MA> RikiddoMV for RikiddoSigmoidMV<FU, FS, FE, MA>
where
    FU: FixedUnsigned + LossyFrom<FixedU32<U32>> + LossyFrom<FixedU128<U128>>,
    FS: FixedSigned
        + From<I9F23>
        + From<i8>
        + LossyFrom<FixedI32<U31>>
        + LossyFrom<U1F127>
        + LossyFrom<FixedI128<U127>>
        + PartialOrd<I9F23>,
    FS::Bits: Copy + ToFixed + AddAssign + BitOrAssign + ShlAssign,
    FE: Fee<FS = FS>,
    MA: MarketAverage<FU = FU>,
{
    /// Clear market data.
    fn clear(&mut self) {
        self.ma_short.clear();
        self.ma_long.clear();
    }

    /// Update market data.
    ///
    /// # Arguments
    ///
    /// * `volume`: The timestamped volume that should be added to the market data.
    fn update_volume(
        &mut self,
        volume: &TimestampedVolume<Self::FU>,
    ) -> Result<Option<Self::FU>, &'static str> {
        let mas = self.ma_short.update_volume(volume)?;
        let mal = self.ma_long.update_volume(volume)?;

        if let Some(mas) = mas {
            if let Some(mal) = mal {
                if mal != fixed_zero::<FU>()? {
                    return Ok(Some(mas.saturating_div(mal)));
                }
            };
        };

        Ok(None)
    }
}
