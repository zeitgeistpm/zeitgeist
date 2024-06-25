# Style guide

As a basis, the
[Substrate Style Guide](https://docs.substrate.io/build/troubleshoot-your-code/)
should be taken into account. In addition to that, the following sections
further elaborate on the style guide used in this repository.

## Comments

- Comments **must** be wrapped at 100 chars per line.

## Comments and Docstrings

- Documentation is written using Markdown syntax.
- Function documentation should be kept lean and mean. Try to avoid documenting
  the parameters, and instead choose self-documenting parameter names. If
  parameters interact in a complex manner (for example, if two arguments of type
  `Vec<T>` must have the same length), then add a paragraph explaining this.
- Begin every docstring with a meaningful one-sentence description of the
  function in the third person.
- Avoid WET documentation such as this:

  ```rust
  /// Rejects a market.
  ///
  /// # Arguments:
  ///
  /// - `origin`: The origin calling the dispatchable
  /// - `market_id`: The market id of the market to reject
  pub fn reject_market(
      origin: OriginFor<T>,
      #[pallet::compact] market_id: MarketIdOf<T>,
  ) -> DispatchResultWithPostInfo {
      // --- snip! ---
  }
  ```

  Self-explanatory function arguments need not be documented if they are
  explained in full text. Instead, focus on describing the effects of the
  function. Someone who doesn't know how to read the code must be able to
  understand what the function does.

- Docstrings for dispatchable **should** contain a section on runtime complexity
  (to make understanding the benchmarks easier).
- Docstrings for dispatchables need not document the `origin` parameter, but
  should specify what origins the dispatchable may be called by.
- Docstrings for dispatchables **must** include the high-level events that the
  dispatchable emits and state under which conditions these events are emitted.
- Document side-effects of functions.
- Use `#![doc = include_str!("../README.md")]`.
- Detail non-trivial algorithms in comments inside the function.

An example of a good docstring would be this:

```rust
/// Rejects a market, destroying the market and unreserving and/or slashing the creator's advisory
/// bond.
///
/// May only be (successfully) called by `RejectOrigin`. The fraction of the advisory bond that is
/// slashed is determined by `AdvisoryBondSlashPercentage`.
///
/// # Emits
///
/// - `MarketRejected` on success.
pub fn reject_market(
    origin: OriginFor<T>,
    #[pallet::compact] market_id: MarketIdOf<T>,
) -> DispatchResultWithPostInfo {
    // --- snip! ---
}
```

Info like "The fraction of the advisory bond that is slashed is determined by
`AdvisoryBondSlashPercentage`" **may** be stored in `README.md` to avoid
duplicating documentation.

## Formatting

- rustfmt and clippy have the last say in formatting.
- Format code contained in macro invocations (`impl_benchmarks!`,
  `decl_runtime_apis!`, homebrew macros in `runtime/`, etc.) and attributes
  (`#[pallet::weight(...)`, etc.) manually.

## Code Style

- Never use panickers.
- All branches of match expressions **should** be explicit. Avoid using the
  catch-all `_ =>`.
- When removing variants from enums that are used in storage or emitted, then
  explicitly state the scale index of each variant:
  ```rust
  enum E {
      #[codec(index = 0)]
      V1,
      #[codec(index = 1)]
      V2,
      /// --- snip! ---
  }
  ```

## Crate and Pallet Structure

- Don't dump all code into `lib.rs`. Split code into multiple files (`types.rs`,
  `traits.rs`, etc.) or even modules (`types/`, `traits/`, etc.).
- Changes to pallets **must** retain the order of dispatchables.
- Sort pallet contents in the following order:
  - `Config` trait
  - Type values
  - Types
  - Storage items
  - Genesis info
  - `Event` enum
  - `Error` enum
  - Hooks
  - Dispatchables
  - Pallet's public and private functions
  - Trait implementations

## Code Style and Design

- Exceed 70 lines of code per function only in exceptional circumstances. Aim
  for less.
- Prefer `for` loops over `while` loops. All loops (of any kind) must have a
  maximum number of passes.
- Use depth checks when using recursion in production. Use recursion only if the
  algorithm is defined using recursion.
- Avoid `mut` in production code if possible without much pain.
- Mark all extrinsics `transactional`, even if they satisfy the
  verify-first/write-later principle.
- Avoid indentation over five levels; never go over seven levels.
- All public functions must be documented. Documentation of `pub(crate)` and
  private functions is optional but encouraged.
- Keep modules lean. Only exceed 1,000 lines of code per file in exceptional
  circumstances. Aim for less (except in `lib.rs`). Consider splitting modules
  into separate files. Auto-generated files are excluded.

## Workflow

- Merges require one review. Additional reviews may be requested.
- Every merge into a feature branch requires a review.
- Feature branches are merged into `develop`, which is merged into
  `release-vX.Y.Z` branches when we're publishing a release.
- Aim for at most 500 LOC added per PR. Only exceed 1,000 LOC lines added in a
  PR in exceptional circumstances. Plan ahead and break a large PR into smaller
  PRs.
- Reviewing a PR should not take longer than two business days. Aim for shorter
  PRs if the changes are complex.
- A PR should not be in flight (going from first `s:ready-for-review` to
  `s:accepted`) for longer than two weeks. Aim for shorter PRs if the changes
  are complex.

## Testing

- Aim for 100% code coverage, excluding only logic errors that are raised on
  inconsistent state. In other words: All execution paths **should** be tested.
  There should be a clear justification for every LOC without test coverage.
- For larger modules, use one test file per extrinsic for unit tests. Make unit
  tests as decoupled as possible from other modules. Place end-to-end and
  integration tests in extra files.
- If possible, test unreachable code and states thought to be impossible using
  the following schema:

  ```rust
  // In code logic
  zeitgeist_macros::unreachable_non_terminating!(condition, log_target, message)
  ```

  ```rust
  // In test
  #[test]
  #[should_panic(expected = message)]
  // Cause assertion
  ```
