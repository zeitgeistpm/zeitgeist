// Copyright 2023 Forecasting Technologies LTD.
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

use super::*;

#[test]
fn shares_of_works() {
    let tree = utility::create_test_tree();
    assert_eq!(tree.shares_of(&3).unwrap(), _1);
    assert_eq!(tree.shares_of(&9).unwrap(), _3);
    assert_eq!(tree.shares_of(&5).unwrap(), _3);
    assert_eq!(tree.shares_of(&7).unwrap(), _1);
    assert_eq!(tree.shares_of(&6).unwrap(), _12);
    assert_eq!(tree.shares_of(&8).unwrap(), _4);
}
