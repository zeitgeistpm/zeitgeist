use codec::FullCodec;
use sp_runtime::{
    traits::{AtLeast32Bit, MaybeSerializeDeserialize},
    DispatchResult,
};
use sp_std::{
    // cmp::{Eq, PartialEq},
    // convert::{TryFrom, TryInto},
    fmt::Debug,
    // result,
};

pub trait Shares<AccountId, Balance, Hash> {
	/// The balance of an account.
	type Balance: AtLeast32Bit + FullCodec + Copy + MaybeSerializeDeserialize + Debug + Default;

    // Getters
    fn free_balance(share_id: Hash, who: &AccountId) -> Balance;
    fn total_supply(share_id: Hash) -> Balance;

    // Mutables
    fn destroy(share_id: Hash, from: &AccountId, amount: Balance) -> DispatchResult;
    fn ensure_can_withdraw(share_id: Hash, who: &AccountId, amount: Balance) -> DispatchResult;
    fn generate(share_id: Hash, to: &AccountId, amount: Balance) -> DispatchResult;
    fn transfer(share_id: Hash, from: &AccountId, to: &AccountId, amount: Balance) -> DispatchResult;
}

pub trait ReservableShares<AccountId, Balance, Hash> {
    fn can_reserve(share_id: Hash, who: &AccountId, value: Balance) -> bool;
    fn reserved_balance(share_id: Hash, who: &AccountId) -> Balance;
    fn reserve(share_id: Hash, who: &AccountId, value: Balance) -> DispatchResult;
    fn unreserve(share_id: Hash, who: &AccountId, value: Balance) -> Balance;
}
