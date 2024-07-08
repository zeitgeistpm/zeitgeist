// Copyright 2023-2024 Forecasting Technologies LTD.
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

#![cfg_attr(not(feature = "std"), no_std)]

/// Creates an `alloc::collections::BTreeMap` from the pattern `{ key => value, ... }`.
///
/// ```ignore
/// // Example:
/// let m = create_b_tree_map!({ 0 => 1, 2 => 3 });
/// assert_eq!(m[2], 3);
///
/// // Overwriting a key:)
/// let m = create_b_tree_map!({ 0 => "foo", 0 => "bar" });
/// assert_eq!(m[0], "bar");
/// ```
#[macro_export]
macro_rules! create_b_tree_map {
    ({ $($key:expr => $value:expr),* $(,)? } $(,)?) => {
        [$(($key, $value),)*].iter().cloned().collect::<alloc::collections::BTreeMap<_, _>>()
    }
}

/// This macro does ensure that a condition `$condition` is met, and if it is not met
/// it will log a message `$message` with optional message arguments `message_args` to
/// an optional log target `$log_target`, cause an assertion in a test environment
/// and execute some optional extra code.
///
/// ```ignore
/// // Examples:
/// unreachable_non_terminating!(a == b, "a does not equal b");
/// unreachable_non_terminating!(a == b, log_target, "a does not equal b");
/// unreachable_non_terminating!(a == b, "{:?} != {:?}", a, b);
/// unreachable_non_terminating!(a == b, log_target, "{:?} != {:?}", a, b);
/// ```
#[macro_export]
macro_rules! unreachable_non_terminating {
    ($condition: expr, $message: literal, $($message_args: tt)*) => {
        let message = format!($message, $($message_args)*);

        #[cfg(test)]
        assert!($condition, "{}", message);

        if !$condition {
            log::warn!("{}", message);
        }
    };
    ($condition: expr, $log_target: ident, $message: literal, $($message_args: tt)*) => {
        let message = format!($message, $($message_args)*);

        #[cfg(test)]
        assert!($condition, "{}", message);

        if !$condition {
            log::warn!(target: $log_target, "{}", message);
        }
    };
    ($condition: expr, $extra_code: expr, $message: literal, $($message_args: tt)*) => {
        let message = format!($message, $($message_args)*);

        #[cfg(test)]
        assert!($condition, "{}", message);

        if !$condition {
            log::warn!("{}", message);
            $extra_code;
        }
    };
    ($condition: expr, $log_target: ident, $extra_code: expr, $message: literal, $($message_args: tt)*) => {
        let message = format!($message, $($message_args)*);

        #[cfg(test)]
        assert!($condition, "{}", message);

        if !$condition {
            log::warn!(target: $log_target, "{}", message);
            $extra_code;
        }
    };
}
