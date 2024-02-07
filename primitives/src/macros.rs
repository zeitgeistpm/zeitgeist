// Copyright 2024 Forecasting Technologies LTD.
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

/// This macro does ensure that a condition `$condition` is met, and if it is not met
/// it will log a message `$message` with optional message arguments `message_args` to
/// an optional log target `$log_target`, cause an assertion in a test environment
/// and execute some optional extra code.
#[macro_export]
macro_rules! unreachable_non_terminating {
    ($condition: expr, $message: literal, $($message_args: tt)*) => {
        let message = format!($message, $($message_args)*);

        #[cfg(test)]
        assert!($condition, "{}", message);

        if $condition {
            log::warn!("{}", message);
        }
    };
    ($condition: expr, $log_target: ident, $message: literal, $($message_args: tt)*) => {
        let message = format!($message, $($message_args)*);

        #[cfg(test)]
        assert!($condition, "{}", message);

        if $condition {
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
