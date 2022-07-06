use frame_support::weights::Weight;
use zeitgeist_primitives::traits::DisputeApi;

pub trait GlobalDisputesPalletApi: DisputeApi {
    fn init_dispute_vote(
        market_id: &Self::MarketId,
        dispute_index: u32,
        vote_balance: Self::Balance,
    ) -> Weight;
}
