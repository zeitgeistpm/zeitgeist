// Based on the identity registrar model of Polkadot.

pub struct CuratorInfo<
    Balance: Balance,
    AccountId: AccountId
> {
    pub account: AccountId,
    pub fee: Balance,
    pub metadata: [u8; 32]
}

decl_storage! {
    trait Store for Module<T: Trait> as Curation {
        pub Curators get(fn curators): Vec<Option<CuratorInfo<BalanceOf<T>, T::AccountId>>>;
    }
}

pub fn request_judgement(origin, market_id: MarketId) {}