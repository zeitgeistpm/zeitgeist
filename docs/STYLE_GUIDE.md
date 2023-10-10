# Style guide

As a basis, the
[Substrate Style Guide](https://docs.substrate.io/build/troubleshoot-your-code/)
should be taken into account. In addition to that, the following sections
further elaborate the style guide used in this repository.

## Comments

- Comments **must** be wrapped at 100 chars per line.
- Comments **must** be formulated in markdown.

## Doc comments

- Documentation is written using Markdown syntax.
- Function documentation should be kept lean and mean. Try to avoid documenting
  the parameters, and instead choose self-documenting parameter names. If
  parameters interact in a complex manner (for example, if two arguments of type
  `Vec<T>` must have the same length), then add a paragraph explaining this.
- Begin every docstring with a meaningful one sentence description of the
  function in third person.
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
- Docstrings for dispatchables **must** include the events that the dispatchable
  emits.
- Use `#![doc = include_str!("../README.md")]`.
- Detail non-trivial algorithms in comments inside the function.

An example of a good docstring would be this:

```rust
/// Rejects a market, destroying the market and unreserving and/or slashing the creator's advisory
/// bond.
///
/// May only be (successfully) called by `RejectOrigin`. The fraction of the advisory bond that is
/// slashed is determined by `AdvisoryBondSlashPercentage`.
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
- Add trailing commas in macro invocations manually, as rustfmt won't add them
  automatically.

  ```rust
  ensure!(
      a_very_very_very_very_very_very_very_long_variable,
      b_very_very_very_very_very_very_very_long_variable, // This comma is not ensured by rustfmt.
  )
  ```

## Code Style

- Never use panickers.
- Prefer double turbofish `Vec::<T>::new()` over single turbofish
  `<Vec<T>>::new()`.
- All branches of match expressions **should** be explicit. Avoid using the
  catch-all `_ =>`.
- When changing enums, maintain the existing order and add variants only at the
  end of the enum to prevent messing up indices.
- Maintain lexicographical ordering of traits in `#[derive(...)]` attributes.

## Crate and Pallet Structure

- Don't dump all code into `lib.rs`. Split code multiple files (`types.rs`,
  `traits.rs`, etc.) or even modules (`types/`, `traits/`, etc.).
- Changes to pallets **must** retain order of dispatchables.
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
  - Trait impelmentations
