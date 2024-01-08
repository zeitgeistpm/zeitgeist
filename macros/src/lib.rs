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
