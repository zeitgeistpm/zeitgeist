use alloc::vec::Vec;
use core::{num::ParseIntError, str::FromStr};

/// The status of a pool. Closely related to the lifecycle of a market.
#[cfg_attr(feature = "std", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    Ord,
    PartialEq,
    PartialOrd,
    parity_scale_codec::Decode,
    parity_scale_codec::Encode,
)]
pub enum PoolStatus {
    /// Shares can be normally negotiated.
    Active,
    /// No trading is allowed. The pool is waiting to be subsidized.
    CollectingSubsidy,
    /// No trading is allowed. Only liquidity awaiting redemption is present in the pool.
    Stale,
}

/// The status of a pool. Closely related to the lifecycle of a market.
#[cfg_attr(feature = "std", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    Ord,
    PartialEq,
    PartialOrd,
    parity_scale_codec::Decode,
    parity_scale_codec::Encode,
)]

pub struct PoolProfit {
    pub best_case: i128,
    pub worst_case: i128
}

impl core::fmt::Display for PoolProfit {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        //write!(f, "{:?}", self)
        <Self as core::fmt::Debug>::fmt(self, f)
    }
}

impl FromStr for PoolProfit {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let coords: Vec<&str> = s.trim_matches(|p| p == '(' || p == ')' )
            .split(',')
            .collect();

        let best_case = coords[0].parse::<i128>()?;
        let worst_case = coords[1].parse::<i128>()?;

        Ok(PoolProfit { best_case, worst_case })
    }
}

impl Default for PoolProfit {
    fn default() -> Self {
        PoolProfit {
            best_case: i128::MIN,
            worst_case: i128::MIN
        }
    }
}