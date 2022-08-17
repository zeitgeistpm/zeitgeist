/// Listing of parachains we integrate with.
/// For each parachain, we are interested in stating their parachain ID
/// as well as any of their token key ID that we possibly support in our
/// XCM configuration. These token keys are defined in said parachain
/// and must always match the value there defined, which is expected to
/// never change once defined since they help define the canonical id
/// of said tokens in the network, which is relevant for XCM transfers.
pub mod karura {
    pub const ID: u32 = 2000;
    pub const AUSD_KEY: &[u8] = &[0, 129];
}

pub mod zeitgeist {
    pub const ID: u32 = 2101;
    pub const ZTG_KEY: &[u8] = &[0, 1];
}

// TODO: Integrate MOVR
// TODO: Integrate USDT