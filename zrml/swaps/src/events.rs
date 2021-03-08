#[derive(
    Clone,
    Debug,
    Default,
    Eq,
    Ord,
    PartialEq,
    PartialOrd,
)]
pub struct GenericPoolEvent<AI>
{
    pub pool_id: u128,
    pub who: AI
}