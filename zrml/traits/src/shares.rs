use codec::FullCodec;
use sp_runtime::{
    traits::{AtLeast32Bit, MaybeSerializeDeserialize},
    DispatchResult,
};
use sp_std::fmt::Debug;

/// A share can also be an asset.
pub trait Shares<AccountId, Balance, Hash> {
	/// The balance of an account.
	type Balance: AtLeast32Bit + FullCodec + Copy + MaybeSerializeDeserialize + Debug + Default;

    // Getters

    /// Free `share_id` balance of a given `who`.
    fn free_balance(share_id: Hash, who: &AccountId) -> Balance;
    
    /// Total supply of a given `share_id`.
    fn total_supply(share_id: Hash) -> Balance;

    // Mutables

    /// Destroys a given `amount` of `from`.
    fn destroy(share_id: Hash, from: &AccountId, amount: Balance) -> DispatchResult;

    /// Destroys all stored asset of a given `share_id`.
    fn destroy_all(share_id: Hash) -> DispatchResult;

    /// Checks if `who` has at least a minimum `amount` of `share_id`.
    fn ensure_can_withdraw(share_id: Hash, who: &AccountId, amount: Balance) -> DispatchResult;

    /// Sets a given `amount` for `to` and increases the total supply.
    fn generate(share_id: Hash, to: &AccountId, amount: Balance) -> DispatchResult;

    /// Transfers a given `amount` of `share_id`.
    fn transfer(share_id: Hash, from: &AccountId, to: &AccountId, amount: Balance) -> DispatchResult;
}

pub trait ReservableShares<AccountId, Balance, Hash> {
    /// Checks if a given `share_id` of `who` has a minimum free balance of `value`
    /// that can be used as a reserve.
    fn can_reserve(share_id: Hash, who: &AccountId, value: Balance) -> bool;

    /// Reserved `share_id` balance of a given `who` account
    fn reserved_balance(share_id: Hash, who: &AccountId) -> Balance;

    /// Reserves a given `value` of `share_id` for `who`.
    fn reserve(share_id: Hash, who: &AccountId, value: Balance) -> DispatchResult;

    /// Un-reserves a given reserved `value` of `share_id` for `who`.
    fn unreserve(share_id: Hash, who: &AccountId, value: Balance) -> Balance;
}

pub trait WrapperShares<AccountId, Balance, Hash> {
    fn get_native_currency_id() -> Hash;
    fn do_wrap_native_currency(who: AccountId, amount: Balance) -> DispatchResult;
    fn do_unwrap_native_currency(who: AccountId, amount: Balance) -> DispatchResult;
}