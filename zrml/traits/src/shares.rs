use codec::FullCodec;
use sp_runtime::{
    traits::{AtLeast32Bit, MaybeSerializeDeserialize},
    DispatchResult,
};
use sp_std::fmt::Debug;

pub trait Shares<AccountId, Balance, Hash> {
	/// The balance of an account.
	type Balance: AtLeast32Bit + FullCodec + Copy + MaybeSerializeDeserialize + Debug + Default;

    // Getters
    fn free_balance(share_id: Hash, who: &AccountId) -> Balance;
    fn total_supply(share_id: Hash) -> Balance;

    // Mutables
    fn destroy(share_id: Hash, from: &AccountId, amount: Balance) -> DispatchResult;
    /// Deletes all shares with a given `share_id`.
    fn destroy_all(share_id: Hash) -> DispatchResult;
    fn ensure_can_withdraw(share_id: Hash, who: &AccountId, amount: Balance) -> DispatchResult;
    /// Sets a given `amount` for `to` account and increases the total supply.
    fn generate(share_id: Hash, to: &AccountId, amount: Balance) -> DispatchResult;
    fn transfer(share_id: Hash, from: &AccountId, to: &AccountId, amount: Balance) -> DispatchResult;
}

pub trait ReservableShares<AccountId, Balance, Hash> {
    fn can_reserve(share_id: Hash, who: &AccountId, value: Balance) -> bool;
    fn reserved_balance(share_id: Hash, who: &AccountId) -> Balance;
    fn reserve(share_id: Hash, who: &AccountId, value: Balance) -> DispatchResult;
    fn unreserve(share_id: Hash, who: &AccountId, value: Balance) -> Balance;
}

pub trait WrapperShares<AccountId, Balance, Hash> {
    fn get_native_currency_id() -> Hash;
    fn do_wrap_native_currency(who: AccountId, amount: Balance) -> DispatchResult;
    fn do_unwrap_native_currency(who: AccountId, amount: Balance) -> DispatchResult;
}