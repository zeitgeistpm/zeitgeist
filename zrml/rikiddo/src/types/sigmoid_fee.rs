//! This module contains the structures used to calculate the fee based on a sigmoid curve.

use super::convert_to_signed;
use crate::{
    constants::{INITIAL_FEE, M, MINIMAL_REVENUE, N, P},
    traits::Fee,
};
#[cfg(feature = "arbitrary")]
use arbitrary::{Arbitrary, Result as ArbiraryResult, Unstructured};
use parity_scale_codec::MaxEncodedLen;
use scale_info::TypeInfo;
#[cfg(feature = "arbitrary")]
use core::mem;
use frame_support::dispatch::{Decode, Encode};
use sp_core::RuntimeDebug;
use substrate_fixed::{
    traits::{FixedSigned, LossyFrom, LossyInto},
    transcendental::sqrt,
    types::{
        extra::{U127, U24, U32},
        I9F23,
    },
    FixedI128, FixedI32, FixedU32,
};
#[cfg(feature = "arbitrary")]
use substrate_fixed::{
    types::extra::{LeEqU128, LeEqU32, LeEqU64},
    FixedI64,
};

/// Configurable values used to calculate a fee based on a sigmoid curve. The usage of the
/// configuration parameters is depicted in equation `z(r)` within the
/// [Dynamic Automated Market Making] paper from Andrew Nguyed et al. Use the `default()`
/// function if uncertain about which values to take.
///
/// [Dynamic Automated Market Making]: https://files.kyber.network/DMM-Feb21.pdf
#[derive(Clone, Decode, Encode, Eq, MaxEncodedLen, PartialEq, RuntimeDebug, TypeInfo)]
pub struct FeeSigmoidConfig<FS: FixedSigned> {
    /// Parameter to fine tune the fee calcultation (refer to example in paper).
    pub m: FS,
    /// Parameter to fine tune the fee calcultation (refer to example in paper).
    pub p: FS,
    /// Break point used to encourage or discourage trade.
    pub n: FS,
    /// The initial fee, that is added to the sigmoid fee result.
    pub initial_fee: FS,
    /// The minimal revenue sets the lower bound for fees.
    pub min_revenue: FS,
}

#[cfg(feature = "arbitrary")]
macro_rules! impl_arbitrary_for_fee_sigmoid_config {
    ( $t:ident, $LeEqU:ident, $p:ty ) => {
        #[allow(clippy::integer_arithmetic)]
        impl<'a, Frac> Arbitrary<'a> for FeeSigmoidConfig<$t<Frac>>
        where
            Frac: $LeEqU,
            $t<Frac>: FixedSigned
                + LossyFrom<FixedI32<U24>>
                + PartialOrd<I9F23>
                + LossyFrom<FixedI128<U127>>,
        {
            fn arbitrary(u: &mut Unstructured<'a>) -> ArbiraryResult<Self> {
                Ok(FeeSigmoidConfig::<$t<Frac>> {
                    m: <$t<Frac>>::from_bits(<$p as Arbitrary<'a>>::arbitrary(u)?),
                    p: <$t<Frac>>::from_bits(<$p as Arbitrary<'a>>::arbitrary(u)?),
                    n: <$t<Frac>>::from_bits(<$p as Arbitrary<'a>>::arbitrary(u)?),
                    initial_fee: <$t<Frac>>::from_bits(<$p as Arbitrary<'a>>::arbitrary(u)?),
                    min_revenue: <$t<Frac>>::from_bits(<$p as Arbitrary<'a>>::arbitrary(u)?),
                })
            }

            #[inline]
            fn size_hint(_depth: usize) -> (usize, Option<usize>) {
                let bytecount = mem::size_of::<$t<Frac>>();
                (bytecount * 5, Some(bytecount * 5))
            }
        }
    };
}

cfg_if::cfg_if! {
    if #[cfg(feature = "arbitrary")] {
        impl_arbitrary_for_fee_sigmoid_config! {FixedI32, LeEqU32, i32}
        impl_arbitrary_for_fee_sigmoid_config! {FixedI64, LeEqU64, i64}
        impl_arbitrary_for_fee_sigmoid_config! {FixedI128, LeEqU128, i128}
    }
}

impl<FS> Default for FeeSigmoidConfig<FS>
where
    FS: FixedSigned + LossyFrom<FixedI32<U24>> + LossyFrom<FixedI128<U127>>,
{
    fn default() -> Self {
        Self {
            m: M.lossy_into(),
            p: P.lossy_into(),
            n: N.lossy_into(),
            // Only case this can panic is, when INITIAL_FEE is >= 1.0 and FS integer bits < 2
            initial_fee: convert_to_signed::<FixedU32<U32>, FS>(INITIAL_FEE.lossy_into()).unwrap(),
            // Only case this can panic is, when MIN_REVENUE is >= 1.0 and FS integer bits < 2
            min_revenue: convert_to_signed::<FixedU32<U32>, FS>(MINIMAL_REVENUE.lossy_into())
                .unwrap(),
        }
    }
}

/// Offers an implementation of `z(r)` as described in [Dynamic Automated Market Making] paper from
/// Andrew Nguyed et al. based on a predetermined set of
/// [configuration values](struct@FeeSigmoidConfig)
///
/// [Dynamic Automated Market Making]: https://files.kyber.network/DMM-Feb21.pdf
#[derive(Clone, Decode, Default, Encode, Eq, MaxEncodedLen, PartialEq, RuntimeDebug, TypeInfo)]
pub struct FeeSigmoid<FS>
where
    FS: FixedSigned + LossyFrom<FixedI32<U24>> + LossyFrom<FixedI128<U127>>,
{
    /// The constants used during the fee calculations.
    pub config: FeeSigmoidConfig<FS>,
}

#[cfg(feature = "arbitrary")]
macro_rules! impl_arbitrary_for_fee_sigmoid {
    ( $t:ident, $LeEqU:ident, $p:ty ) => {
        impl<'a, Frac> Arbitrary<'a> for FeeSigmoid<$t<Frac>>
        where
            Frac: $LeEqU,
            $t<Frac>: FixedSigned
                + LossyFrom<FixedI32<U24>>
                + PartialOrd<I9F23>
                + LossyFrom<FixedI128<U127>>,
        {
            fn arbitrary(u: &mut Unstructured<'a>) -> ArbiraryResult<Self> {
                Ok(FeeSigmoid::new(<FeeSigmoidConfig<$t<Frac>> as Arbitrary<'a>>::arbitrary(u)?))
            }

            #[inline]
            fn size_hint(depth: usize) -> (usize, Option<usize>) {
                <FeeSigmoidConfig<$t<Frac>> as Arbitrary<'a>>::size_hint(depth)
            }
        }
    };
}

#[cfg(feature = "arbitrary")]
cfg_if::cfg_if! {
    if #[cfg(feature = "arbitrary")] {
        impl_arbitrary_for_fee_sigmoid! {FixedI32, LeEqU32, i32}
        impl_arbitrary_for_fee_sigmoid! {FixedI64, LeEqU64, i64}
        impl_arbitrary_for_fee_sigmoid! {FixedI128, LeEqU128, i128}
    }
}

impl<FS> FeeSigmoid<FS>
where
    FS: FixedSigned + LossyFrom<FixedI32<U24>> + LossyFrom<FixedI128<U127>>,
{
    /// Create a new `FeeSigmoid` instance based on a [`FeeSigmoidConfig`](struct@FeeSigmoidConfig)
    /// configuration. Use `default()` if uncertain which values to use.
    ///
    /// # Arguments
    ///
    /// * `config`: Parameters used during the fee calculation.
    pub fn new(config: FeeSigmoidConfig<FS>) -> Self {
        Self { config }
    }
}

impl<FS> Fee for FeeSigmoid<FS>
where
    FS: FixedSigned + LossyFrom<FixedI32<U24>> + PartialOrd<I9F23> + LossyFrom<FixedI128<U127>>,
{
    type FS = FS;

    /// Calculate fee: min(`min_revenue`, `initial_fee` + z(r))
    ///
    /// # Arguments
    ///
    /// * `r`: Some kind of information about the market, for example an ema.
    ///
    /// [z(r)]: https://files.kyber.network/DMM-Feb21.pdf
    fn calculate_fee(&self, r: Self::FS) -> Result<Self::FS, &'static str> {
        let r_minus_n = if let Some(res) = r.checked_sub(self.config.n) {
            res
        } else {
            return Err("[FeeSigmoid] Overflow during calculation: r - n");
        };

        let numerator = if let Some(res) = r_minus_n.checked_mul(self.config.m) {
            res
        } else {
            return Err("[FeeSigmoid] Overflow during calculation: m * (r-n)");
        };

        let r_minus_n_squared = if let Some(res) = r_minus_n.checked_mul(r_minus_n) {
            res
        } else {
            return Err("[FeeSigmoid] Overflow during calculation: (r-n)^2");
        };

        let p_plus_r_minus_n_squared =
            if let Some(res) = self.config.p.checked_add(r_minus_n_squared) {
                res
            } else {
                return Err("[FeeSigmoid] Overflow during calculation: p + (r-n)^2");
            };

        let denominator = sqrt::<FS, FS>(p_plus_r_minus_n_squared)?;

        let sigmoid_result = if let Some(res) = numerator.checked_div(denominator) {
            res
        } else {
            return Err("[FeeSigmoid] Overflow during calculation: numerator / denominator");
        };

        let result = if let Some(res) = sigmoid_result.checked_add(self.config.initial_fee) {
            res
        } else {
            return Err("[FeeSigmoid] initial_fee + sigmoid_result");
        };

        if self.config.min_revenue >= result {
            return Ok(self.config.min_revenue);
        }

        Ok(result)
    }

    /// Return the minimum fee
    fn minimum_fee(&self) -> Self::FS {
        self.config.min_revenue
    }
}
