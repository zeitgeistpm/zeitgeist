#[cfg(feature = "parachain")]
mod service_parachain;
#[cfg(not(feature = "parachain"))]
mod service_standalone;

#[cfg(feature = "parachain")]
pub use service_parachain::{new_full, new_partial};
#[cfg(not(feature = "parachain"))]
pub use service_standalone::{new_full, new_light, new_partial};

pub struct ExecutorDispatch;

impl sc_executor::NativeExecutionDispatch for ExecutorDispatch {
  #[cfg(feature = "runtime-benchmarks")]
  type ExtendHostFunctions = frame_benchmarking::benchmarking::HostFunctions;
  #[cfg(not(features = "runtime-benchmarks"))]
  type ExtendHostFunctions = ();

  fn dispatch(method: &str, data: &[u8]) -> Option<Vec<u8>> {
    zeitgeist_runtime::api::dispatch(method, data)
  }

  fn native_version() -> sc_executor::NativeVersion {
    zeitgeist_runtime::native_version()
  }
}