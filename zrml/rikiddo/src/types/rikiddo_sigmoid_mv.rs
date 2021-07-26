use crate::{
    constants::INITIAL_FEE,
    traits::{Lmsr, MarketAverage, RikiddoMV, Sigmoid},
};
use core::ops::{AddAssign, BitOrAssign, ShlAssign};
use frame_support::dispatch::{fmt::Debug, Decode, Encode};
use hashbrown::HashMap;
use substrate_fixed::{
    consts::LOG2_E,
    traits::{Fixed, FixedSigned, FixedUnsigned, LossyFrom, LossyInto, ToFixed},
    transcendental::{exp, ln, log2},
    types::{
        extra::{U127, U128, U31, U32},
        I9F23, U1F127,
    },
    FixedI128, FixedI32, FixedU128, FixedU32,
};

use super::{convert_to_signed, convert_to_unsigned, TimestampedVolume};

#[derive(Clone, Debug, Decode, Encode, Eq, PartialEq)]
pub struct RikiddoConfig<FI: Fixed> {
    pub initial_fee: FI,
    pub(crate) log2_e: FI,
}

impl<FS: FixedSigned + LossyFrom<FixedI32<U31>> + LossyFrom<U1F127>> RikiddoConfig<FS> {
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
    pub(crate) ln_sum_exp: FS,
    pub(crate) exponents: HashMap<FS, FS>,
    pub(crate) reduced_exponential_results: HashMap<FS, FS>,
}

impl<FS> Default for RikiddoFormulaComponents<FS>
where
    FS: FixedSigned + From<I9F23> + LossyFrom<FixedI32<U31>> + LossyFrom<U1F127>,
{
    fn default() -> Self {
        let zero = 0.to_fixed();
        Self {
            one: 1i32.to_fixed(),
            fee: zero,
            sum_balances: zero,
            sum_times_fee: zero,
            emax: zero,
            sum_exp: zero,
            ln_sum_exp: zero,
            exponents: HashMap::new(),
            reduced_exponential_results: HashMap::new(),
        }
    }
}

#[derive(Clone, Debug, Decode, Default, Encode, Eq, PartialEq)]
pub struct RikiddoSigmoidMV<FU, FS, FE, MA>
where
    FU: FixedUnsigned + LossyFrom<FixedU32<U32>>,
    FS: FixedSigned + LossyFrom<FixedI32<U31>> + LossyFrom<U1F127>,
    FE: Sigmoid<FS = FS>,
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
        + From<I9F23>
        + LossyFrom<FixedI32<U31>>
        + LossyFrom<U1F127>
        + LossyFrom<FixedI128<U127>>
        + PartialOrd<I9F23>,
    FS::Bits: Copy + ToFixed + AddAssign + BitOrAssign + ShlAssign,
    FE: Sigmoid<FS = FS>,
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
        convert_to_unsigned::<FS, FU>(self.fees.calculate(ratio_signed)?)
    }

    // Cost function that returns the cost and parts of the formula that can be reused for the
    // price calculation.
    // Setting `for_price = true` returns the cost minus the sum of quantities.
    // Setting `enforce_optimized = true` enforces the optimized strategy, which additionally sets
    // a field (sum_exp) that is reusable within the price function.
    pub(crate) fn cost_with_forumla(
        &self,
        asset_balances: &[FU],
        formula_components: &mut RikiddoFormulaComponents<FS>,
        for_price: bool,
        enforce_optimized: bool,
        add_exponents: bool,
    ) -> Result<FS, &'static str> {
        if asset_balances.is_empty() {
            return Err("[RikiddoSigmoidMV] No asset balances provided");
        };

        let fee = self.get_fee()?;
        formula_components.fee = convert_to_signed(fee)?;
        let mut total_balance = FU::from_num(0u8);

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
        let mut biggest_exponent: FS = FS::from_num(0u8);

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

        // Determine which strategy to use.
        let biggest_exp_times_log2e = if let Some(res) =
            self.config.log2_e.checked_mul(biggest_exponent)
        {
            res
        } else {
            // Highly unlikely
            return Err("[RikiddoSigmoidMV] Overflow during calculation: log2_e * biggest_exponent");
        };

        if FS::max_value().int().to_num::<u128>() < asset_balances.len() as u128 {
            return Err("[RikidoSigmoidMV] Number of assets does not fit in FS");
        }

        // Panic impossible
        let log2_number_of_assets: FS =
            if let Ok(res) = log2::<FS, FS>(FS::from_num(asset_balances.len())) {
                res
            } else {
                // Impossible, since the cost functions checks if elements are present in asset_balances
                return Err("[RikiddoSigmoidMV] log2(number_of_assets), number_of_assets <= 0");
            };

        let log2e_times_biggest_exp_plus_log2_num_assets =
            if let Some(res) = log2_number_of_assets.checked_add(biggest_exp_times_log2e) {
                res
            } else {
                // Highly unlikely
                return Err("[RikiddoSigmoidMV] Overflow during calculation: biggest_exp * \
                            log2(e) + log2(num_assets)");
            };

        let required_bits_minus_one: u128 =
            if let Some(res) = log2e_times_biggest_exp_plus_log2_num_assets.checked_ceil() {
                res.to_num()
            } else {
                // Highly unlikely
                return Err("[RikiddoSigmoidMV] Overflow during calculation: ceil(biggest_exp * \
                            log2(e) + log2(num_assets))");
            };

        let required_bits = if let Some(res) = required_bits_minus_one.checked_add(1) {
            res
        } else {
            // Overflow impossible
            return Err(
                "[RikiddoSigmoidMV] Overflow during calculation: required_bits_minus_one + 1"
            );
        };

        let ln_sum_e: FS;

        // Select strategy to calculate ln(sum_i(e^i))
        if required_bits > FS::int_nbits() as u128 || enforce_optimized {
            ln_sum_e = self.optimized_cost_strategy(
                &exponents,
                &biggest_exponent,
                formula_components,
                add_exponents,
            )?;
        } else {
            ln_sum_e = self.default_cost_strategy(&exponents)?;
        }

        formula_components.ln_sum_exp = ln_sum_e;

        if for_price {
            if let Some(res) = formula_components.fee.checked_mul(ln_sum_e) {
                Ok(res)
            } else {
                Err("[RikiddoSigmoidMV] Overflow during calculation: fee * ln(sum_i(e^i))")
            }
        } else {
            if let Some(res) = formula_components.sum_times_fee.checked_mul(ln_sum_e) {
                Ok(res)
            } else {
                Err("[RikiddoSigmoidMV] Overflow during calculation: fee * total_asset_balance * \
                     ln(sum_i(e^i))")
            }
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
            if let Some(res) = formula_components.exponents.get(&asset_in_question_balance) {
                *res
            } else {
                return Err("[RikiddoSigmoidMV] Cannot find exponent of asset balance in \
                            question in RikiddoFormulaComponents HashMap");
            };

        let mut sum: FS = 0.to_fixed();
        let mut skipped = false;

        for elem in asset_balances {
            if elem == asset_in_question_balance && !skipped {
                skipped = true;
                continue;
            }

            let elem_div_sum_fee = if let Some(res) = formula_components.exponents.get(&elem) {
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
                0.to_fixed()
            };

            sum = if let Some(res) = sum.checked_add(exponential_result) {
                res
            } else {
                // In that case the final result will not fit into the fractional bits
                // and therefore is approximated to zero. Cannot panic.
                return Ok(0.to_fixed());
            };
        }

        sum = if let Some(res) = sum.checked_add(formula_components.one) {
            res
        } else {
            // In that case the final result will not fit into the fractional bits
            // and therefore is approximated to zero. Cannot panic.
            return Ok(0.to_fixed());
        };

        if let Some(res) = formula_components.one.checked_div(sum) {
            Ok(res)
        } else {
            Ok(0.to_fixed())
        }
    }

    // Calculates the second quotient in the price formula after the cost / sum_balances
    pub(crate) fn price_helper_second_quotient(
        &self,
        asset_balances: &[FS],
        formula_components: &RikiddoFormulaComponents<FS>,
    ) -> Result<FS, &'static str> {
        let mut numerator: FS = 0.to_fixed();
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
                let exponential_result = if let Some(res) = formula_components.reduced_exponential_results.get(&exponent_of_balance_in_question) {
                    *res
                } else {
                    return Err(
                        "[RikiddoSigmoidMV] Cannot find reduced exponential result of current element"
                    );
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

    //fn price_helper_combine_all_parts()?;

    pub(crate) fn default_cost_strategy(&self, exponents: &[FS]) -> Result<FS, &'static str> {
        let mut acc: FS = FS::from_num(0u8);

        for elem in exponents {
            let exp_value: FS = if let Ok(res) = exp::<FS, FS>(*elem) {
                res
            } else {
                return Err(
                    "[RikiddoSigmoidMV] Error during calculation: exp(i) in ln sum_i(exp^i)"
                );
            };

            if let Some(res) = acc.checked_add(exp_value) {
                acc = res;
            } else {
                // Impossible (this function should only be called when the sum does fit into FS)
                return Err("[RikiddoSigmoidMV] Overflow during calculation: sum_i(e^i)");
            };
        }

        if let Ok(res) = ln::<FS, FS>(acc) {
            Ok(res)
        } else {
            // Impossible to reach, unless the "exponents" vector is empty
            Err("[RikiddoSigmoidMV] ln(exp_sum), exp_sum <= 0")
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

        if FS::max_value().int().to_num::<u128>() < 1u128 {
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
        + LossyFrom<FixedI32<U31>>
        + LossyFrom<U1F127>
        + LossyFrom<FixedI128<U127>>
        + PartialOrd<I9F23>,
    FS::Bits: Copy + ToFixed + AddAssign + BitOrAssign + ShlAssign,
    FE: Sigmoid<FS = FS>,
    MA: MarketAverage<FU = FU>,
{
    type FU = FU;

    /// Return price P_i(q) for all assets in q
    fn all_prices(&self, _asset_balances: &[Self::FU]) -> Result<Vec<Self::FU>, &'static str> {
        Err("Unimplemented")
    }

    /// Return cost C(q) for all assets in q
    fn cost(&self, asset_balances: &[Self::FU]) -> Result<Self::FU, &'static str> {
        convert_to_unsigned(self.cost_with_forumla(
            asset_balances,
            &mut Default::default(),
            false,
            false,
            false,
        )?)
    }

    /// Return price P_i(q) for asset q_i in q
    fn price(
        &self,
        asset_in_question_balance: &Self::FU,
        asset_balances: &[Self::FU],
    ) -> Result<Self::FU, &'static str> {
        let mut formula_components = RikiddoFormulaComponents::default();

        let cost_part =
            self.cost_with_forumla(asset_balances, &mut formula_components, true, true, true)?;
        let mut asset_balances_signed = Vec::with_capacity(asset_balances.len());
        let mut asset_in_question_balance_signed = 0.to_fixed();
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
            return Err("[RikiddoSigmoidMV] asset_in_question_balance not found in asset_balances")
        }

        let first_quotient = self.price_helper_first_quotient(&asset_balances_signed, &asset_in_question_balance_signed, &formula_components)?;
        let second_quotient = self.price_helper_second_quotient(&asset_balances_signed, &formula_components)?;

        let quotient_sub = if let Some(res) = first_quotient.checked_sub(second_quotient) {
            res
        } else {
            // Highly unlikely
            return Err("[RikiddoSigmoidMV] Overflow during calculation of price: first_quotient - second_quotient")
        };

        if let Some(res) = cost_part.checked_add(quotient_sub) {
            convert_to_unsigned(res)
        } else {
            // Highly unlikely
            Err("[RikiddoSigmoidMV] Overflow during calculation of price: first_quotient - second_quotient")
        }
    }
}

impl<FU, FS, FE, MA> RikiddoMV for RikiddoSigmoidMV<FU, FS, FE, MA>
where
    FU: FixedUnsigned + LossyFrom<FixedU32<U32>> + LossyFrom<FixedU128<U128>>,
    FS: FixedSigned
        + From<I9F23>
        + LossyFrom<FixedI32<U31>>
        + LossyFrom<U1F127>
        + LossyFrom<FixedI128<U127>>
        + PartialOrd<I9F23>,
    FS::Bits: Copy + ToFixed + AddAssign + BitOrAssign + ShlAssign,
    FE: Sigmoid<FS = FS>,
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
