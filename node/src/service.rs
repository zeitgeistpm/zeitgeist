#[cfg(feature = "parachain")]
mod service_parachain;
#[cfg(not(feature = "parachain"))]
mod service_standalone;

#[cfg(feature = "parachain")]
pub use service_parachain::{new_full, new_partial};
#[cfg(not(feature = "parachain"))]
pub use service_standalone::{new_full, new_light, new_partial};

use sc_executor::native_executor_instance;

native_executor_instance!(
  pub Executor,
  zeitgeist_runtime::api::dispatch,
  zeitgeist_runtime::native_version,
  frame_benchmarking::benchmarking::HostFunctions,
);
