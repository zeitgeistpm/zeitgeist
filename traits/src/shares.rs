// use codec::{Codec, FullCodec};
// use sp_runtime::{
//     traits::{AtLeast32Bit, MaybeSerializeDeserialize},
//     DispatchError, DispatchResult,
// };
// use sp_std::{
//     cmp::{Eq, PartialEq},
//     convert::{TryFrom, TryInto},
//     fmt::Debug,
//     result,
// };

pub trait Shares<AccountId, Balance, Hash> {
    fn free_balance(share_id: Hash, who: &AccountId) -> Balance;

    fn generate(
        share_id: Hash,
        to: &AccountId,
        amount: Balance,
    );

    fn destroy(share_id: Hash, from: &AccountId, amount: Balance);
}
