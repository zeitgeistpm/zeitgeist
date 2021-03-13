/// Money matters.
pub mod currency {
    use zeitgeist_primitives::Balance;

    pub const ZGE: Balance = 10_000_000_000;
    pub const DOLLARS: Balance = ZGE / 100; // 100_000_000
    pub const CENTS: Balance = DOLLARS / 100; // 1_000_000
    pub const MILLICENTS: Balance = CENTS / 1000; // 1_000

    // TODO: figure out a good deposit function
}

/// Time and blocks.
pub mod time {
    use zeitgeist_primitives::{BlockNumber, Moment};

    pub const MILLISECS_PER_BLOCK: Moment = 6000;
    pub const SLOT_DURATION: Moment = MILLISECS_PER_BLOCK;

    // Time is measured in number of blocks.
    pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
    pub const HOURS: BlockNumber = MINUTES * 60;
    pub const DAY: BlockNumber = HOURS * 24;
}
