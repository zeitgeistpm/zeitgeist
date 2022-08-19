// Copyright 2021-2022 Zeitgeist PM LLC.
//
// This file is part of Zeitgeist.
//
// Zeitgeist is free software: you can redistribute it and/or modify it
// under the terms of the GNU General Public License as published by the
// Free Software Foundation, either version 3 of the License, or (at
// your option) any later version.
//
// Zeitgeist is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Zeitgeist. If not, see <https://www.gnu.org/licenses/>.

use crate::types::{CategoryIndex, MarketId, PoolId, SerdeWrapper};
use alloc::{vec, vec::Vec};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use bstringify::bstringify;
use sp_runtime::RuntimeDebug;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

/// The `Asset` enum represents all types of assets available in the Zeitgeist
/// system.
///
/// # Types
///
/// * `MI`: Market Id
#[cfg_attr(feature = "std", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
#[derive(
    Clone, Copy, Debug, Decode, Eq, Encode, MaxEncodedLen, Ord, PartialEq, PartialOrd, TypeInfo,
)]
pub enum Asset<MI: MaxEncodedLen> {
    CategoricalOutcome(MI, CategoryIndex),
    ScalarOutcome(MI, ScalarPosition),
    CombinatorialOutcome,
    PoolShare(SerdeWrapper<PoolId>),
    ZTG,
    // TODO: Use either Roc in Battery Station or KSM on mainnet
    ROC,
    AUSD,
}

impl<MI: MaxEncodedLen> Asset<MI> {
    pub fn is_token(&self) -> bool {
        !matches!(self, Asset::CategoricalOutcome(_, _) | Asset::ScalarOutcome(_, _) | Asset::CombinatorialOutcome | Asset::PoolShare(_))
    }

    pub fn is_outcome_token(&self) -> bool {
        matches!(self, Asset::CategoricalOutcome(_, _) | Asset::ScalarOutcome(_, _) | Asset::CombinatorialOutcome)
    }

    pub fn is_pool_share(&self) -> bool {
        matches!(self, Asset::PoolShare(_))
    }
}

pub trait TokenInfo {
    fn currency_id(&self) -> Option<u8>;
    fn name(&self) -> Option<&str>;
    fn symbol(&self) -> Option<&str>;
    fn decimals(&self) -> Option<u8>;
}

macro_rules! create_currency_id {
    ($(#[$meta:meta])*
    $vis:vis enum TokenSymbol {
        $($(#[$vmeta:meta])* $symbol:ident($name:expr, $deci:literal) = $val:literal,)*
    }) => {
        $(#[$meta])*
        $vis enum TokenSymbol {
            $($(#[$vmeta])* $symbol = $val,)*
        }

        impl TryFrom<u8> for TokenSymbol {
            type Error = ();

            fn try_from(v: u8) -> Result<Self, Self::Error> {
                match v {
                    $($val => Ok(TokenSymbol::$symbol),)*
                    _ => Err(()),
                }
            }
        }

        impl Into<u8> for TokenSymbol {
            fn into(self) -> u8 {
                match self {
                    $(TokenSymbol::$symbol => ($val),)*
                }
            }
        }

        impl<MI: MaxEncodedLen> TryFrom<Vec<u8>> for Asset<MI> {
            type Error = ();
            fn try_from(v: Vec<u8>) -> Result<Asset<MI>, ()> {
                match v.as_slice() {
                    $(bstringify!($symbol) => Ok(Asset::$symbol),)*
                    _ => Err(()),
                }
            }
        }

        impl<MI: MaxEncodedLen> TokenInfo for Asset<MI> {
            fn currency_id(&self) -> Option<u8> {
                match self {
                    $(Asset::$symbol => Some($val),)*
                    _ => None,
                }
            }
            fn name(&self) -> Option<&str> {
                match self {
                    $(Asset::$symbol => Some($name),)*
                    _ => None,
                }
            }
            fn symbol(&self) -> Option<&str> {
                match self {
                    $(Asset::$symbol => Some(stringify!($symbol)),)*
                    _ => None,
                }
            }
            fn decimals(&self) -> Option<u8> {
                match self {
                    $(Asset::$symbol => Some($deci),)*
                    _ => None,
                }
            }
        }

        $(pub const $symbol: Asset<MarketId> = Asset::$symbol;)*

        impl TokenSymbol {
            pub fn get_info() -> Vec<(&'static str, u32)> {
                vec![
                    $((stringify!($symbol), $deci),)*
                ]
            }
        }
    }
}

create_currency_id! {
    // Represent a Token symbol with 8 bit
    //
    // 0 - 19: Zeitgeist & Rococo/Kusama native tokens
    // 20 - 49: Bridged tokens
    // 50 - 127: Rococo / Kusama parachain tokens
    // 128 - 255: Unreserved
    #[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, PartialOrd, Ord, TypeInfo, MaxEncodedLen)]
    #[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
    #[repr(u8)]
    pub enum TokenSymbol {
        // TODO: Use either ROC or KSM depending on runtime that's build
        ROC("Rococo", 12) = 0,
        // 0 - 19: Zeitgeist & Rococo/Kusama native tokens
        ZTG("Zeitgeist", 10) = 1,
        // 20 - 49: Bridged tokens
        // 50 - 127: Rococo / Kusama parachain tokens
        AUSD("Acala Dollar", 12) = 50,
    }
}



/// In a scalar market, users can either choose a `Long` position,
/// meaning that they think the outcome will be closer to the upper bound
/// or a `Short` position meaning that they think the outcome will be closer
/// to the lower bound.
#[cfg_attr(feature = "std", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
#[derive(
    Clone, Copy, Debug, Decode, Eq, Encode, MaxEncodedLen, Ord, PartialEq, PartialOrd, TypeInfo,
)]
pub enum ScalarPosition {
    Long,
    Short,
}
