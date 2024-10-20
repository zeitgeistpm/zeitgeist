use crate::{
    mock::{runtime::Runtime, types::MockOracle},
    OracleOf,
};
use zeitgeist_primitives::traits::FutarchyBenchmarkHelper;

pub struct MockBenchmarkHelper;

impl FutarchyBenchmarkHelper<OracleOf<Runtime>> for MockBenchmarkHelper {
    fn create_oracle(value: bool) -> OracleOf<Runtime> {
        MockOracle::new(Default::default(), value)
    }
}
