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

//! This module contains a collection of types that are required to implement the Rikiddo core
//! functionality, as well as the Rikiddo core functionality itself.

extern crate alloc;
#[cfg(feature = "arbitrary")]
use arbitrary::{Arbitrary, Result as ArbiraryResult, Unstructured};
#[cfg(feature = "arbitrary")]
use core::mem;
use frame_support::dispatch::{Decode, Encode};
use parity_scale_codec::MaxEncodedLen;
use scale_info::TypeInfo;
use sp_core::RuntimeDebug;
use substrate_fixed::traits::Fixed;
#[cfg(feature = "arbitrary")]
use substrate_fixed::{
    types::extra::{LeEqU128, LeEqU16, LeEqU32, LeEqU64, LeEqU8},
    FixedI128, FixedI16, FixedI32, FixedI64, FixedI8, FixedU128, FixedU16, FixedU32, FixedU64,
    FixedU8,
};

mod ema_market_volume;
mod rikiddo_sigmoid_mv;
mod sigmoid_fee;

pub use ema_market_volume::*;
pub use rikiddo_sigmoid_mv::*;
pub use sigmoid_fee::*;

/// A timestamp that contains the seconds since January 1st, 1970 at UTC.
pub type UnixTimestamp = u64;

/// A 2-tuple containing an unix timestamp and a volume.
#[derive(Clone, Decode, Default, Encode, Eq, MaxEncodedLen, PartialEq, RuntimeDebug, TypeInfo)]
pub struct TimestampedVolume<F: Fixed> {
    /// The timestamp of the volume.
    pub timestamp: UnixTimestamp,
    /// The volume.
    pub volume: F,
}

#[cfg(feature = "arbitrary")]
macro_rules! impl_arbitrary_for_timestamped_volume {
    ( $t:ident, $LeEqU:ident, $p:ty ) => {
        #[allow(clippy::integer_arithmetic)]
        impl<'a, Frac> Arbitrary<'a> for TimestampedVolume<$t<Frac>>
        where
            Frac: $LeEqU,
            $t<Frac>: Fixed,
        {
            fn arbitrary(u: &mut Unstructured<'a>) -> ArbiraryResult<Self> {
                return Ok(TimestampedVolume {
                    timestamp: <UnixTimestamp as Arbitrary<'a>>::arbitrary(u)?,
                    volume: <$t<Frac>>::from_bits(<$p as Arbitrary<'a>>::arbitrary(u)?),
                });
            }

            #[inline]
            fn size_hint(_depth: usize) -> (usize, Option<usize>) {
                let bytecount_fixed = mem::size_of::<$t<Frac>>();
                let bytecount_timestamp = mem::size_of::<UnixTimestamp>();
                let required_bytes = bytecount_fixed + bytecount_timestamp;
                (required_bytes, Some(required_bytes))
            }
        }
    };
}

cfg_if::cfg_if! {
    if #[cfg(feature = "arbitrary")] {
        impl_arbitrary_for_timestamped_volume! {FixedI8, LeEqU8, i8}
        impl_arbitrary_for_timestamped_volume! {FixedI16, LeEqU16, i16}
        impl_arbitrary_for_timestamped_volume! {FixedI32, LeEqU32, i32}
        impl_arbitrary_for_timestamped_volume! {FixedI64, LeEqU64, i64}
        impl_arbitrary_for_timestamped_volume! {FixedI128, LeEqU128, i128}
        impl_arbitrary_for_timestamped_volume! {FixedU8, LeEqU8, u8}
        impl_arbitrary_for_timestamped_volume! {FixedU16, LeEqU16, u16}
        impl_arbitrary_for_timestamped_volume! {FixedU32, LeEqU32, u32}
        impl_arbitrary_for_timestamped_volume! {FixedU64, LeEqU64, u64}
        impl_arbitrary_for_timestamped_volume! {FixedU128, LeEqU128, u128}
    }
}

/// A enum that wrappes an amount of time in different units.
/// An enum that wrappes an amount of time in different units.
#[derive(
    Clone, Copy, Decode, Encode, Eq, MaxEncodedLen, PartialEq, PartialOrd, RuntimeDebug, TypeInfo,
)]
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
pub enum Timespan {
    /// Contains seconds.
    Seconds(u32),
    /// Contains minutes.
    Minutes(u32),
    /// Contains hours.
    Hours(u32),
    /// Contains days.
    Days(u16),
    /// Contains weeks.
    Weeks(u16),
}

impl Timespan {
    /// Convert the current `Timespan` into a number of seconds.
    pub fn to_seconds(&self) -> u32 {
        match *self {
            // Any value that leads to a saturation is greater than
            // 4294967295 seconds, which is about 136 years.
            Timespan::Seconds(d) => d,
            Timespan::Minutes(d) => d.saturating_mul(60),
            Timespan::Hours(d) => d.saturating_mul(60).saturating_mul(60),
            Timespan::Days(d) => {
                u32::from(d).saturating_mul(60).saturating_mul(60).saturating_mul(24)
            }
            Timespan::Weeks(d) => u32::from(d)
                .saturating_mul(60)
                .saturating_mul(60)
                .saturating_mul(24)
                .saturating_mul(7),
        }
    }
}
