use zeitgeist_primitives::traits::DisputeApi;
use frame_support::weights::Weight;

pub trait GlobalDisputesPalletApi: DisputeApi {
    fn init_dispute_vote(
        market_id: &Self::MarketId,
        dispute_index: u32,
        vote_balance: Self::Balance,
    ) -> Weight;
}
