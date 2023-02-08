- [ ] Check if there are any TODOs which are not linked to an issue of the
      zeitgeist repository like this: `TODO(#999)`.
- [ ] Does the PR link relevant issues and contain a detailed description?
- [ ] If the PR changes the business logic, does it add the corresponding
      labels?
- [ ] Are the doc strings up to date?
- [ ] If the PR adds or changes extrinsics or functions used by extrinsics:
  - [ ] Is the _Weight_ section in the documentation up to date?
  - [ ] Are the benchmarks up to date?
  - [ ] Do the call filters need to be adjusted (forbidden scoring rules,
        dispute mechanisms)?
  - [ ] Do the extrinsics emit all the required events (see [Events](#events)
        below)?
- [ ] Is the module `README.md` up to date?
- [ ] Is [docs.zeitgeist.pm] up to date?
- [ ] Is `docs/changelog_for_devs.md` up to date, specifically:
  - [ ] Are changes relevant to the Frontend Team (extrinsics changed, new
        functions) mentioned here?
  - [ ] Are all new events explained so they can easily be integrated into the
        indexer?
  - [ ] Are breaking changes marked as such?
  - [ ] Is the file formatted with `prettier -w docs/changelog_for_devs.md`?
- Sanity tests:
  - [ ] The local node produces blocks.
  - [ ] `try-runtime` passes on Zeitgeist and Battery Station.
  - [ ] All runtime benchmarks pass on Zeitgeist and Battery Station (don't just
        test against the mock!).
  - [ ] The node syncs with Zeitgeist and Battery Station.
- [ ] Code quality:
  - [ ] Are there any compiler/clippy warnings?
  - [ ] Is integer arithmetic saturated/checked and are all panickers removed?
  - [ ] Is code contained in macro invocations (benchmarks,
        `runtime/common/src/lib.rs`, `decl_runtime_apis!`) correctly formatted?
  - [ ] Are all `*.toml` files formatted with `taplo`?
  - [ ] Are all copyright notices up to date?
  - [ ] Are enums sorted alphabetically, except for enums used in storage (to
        prevent migrations) or errors and events?
- [ ] If an action is required by the Frontend Team, add an issue to
      zeitgeistpm/ui.
- [ ] If the PR adds a new pallet, is the pallet added to the benchmark
      configuration in `scripts/`?
- [ ] If you are changing/removing configuration values, storage items: Do the
      changes require a storage migration?
- [ ] If you are adding/changing configuration values on the mainnet: Have the
      implications been discussed with the product owners?
- [ ] If the changes include a storage migration:
  - [ ] Is the version number in the pallet bumped?
  - [ ] Add try-runtime checks and ensure the following:
    - [ ] The storage migration bumps the pallet version correctly.
    - [ ] The try-runtime _fails_ if you comment out the migration code.
    - [ ] The try-runtime passes without any warnings (substrate storage
          operations often just log a warning instead of failing, so these
          warnings usually point to problem which could break the storage).

## Events

_All_ modifications of the on-chain storage **must** be broadcast to our indexer
by emitting a high-level event. The term _high-level_ event refers to an event
which may or may not contextualize _low-level_ events emitted by pallets that
our business logic builds on, for example pallet-balances. Examples of
high-level events are:

- `SwapExactAmountIn` (contextualizes a couple of low-level events like
  `Transfer`)
- `PoolActive` (doesn't add context, but describes a storage change of a pool
  structure)

Furthermore, these modifications need to be detailed by specifying either a diff
or the new storage content, unless the change is absolutely clear from the
context. For example, `SwapExactAmountIn` specifies the balance changes that the
accounts suffer, but `PoolActive` only provides the id of the pool, _not_ the
new status (`Active`), which is clear from the context. Information that is
implicitly already available to the indexer **may** be provided, but this is not
necessary. For example, the `MarketRejected(market_id, reason)` event not only
specifies that the market with `market_id` was rejected and for what `reason`,
but also that the advisory bond and oracle bond are settled, but it doesn't
include the info how much the oracle bond actually was.

Additional info (similar to the remark emitted by
[`remark_with_event`](https://github.com/paritytech/substrate/blob/6a504b063cf66351b6e352ef18cc18d49146487b/frame/system/src/lib.rs#L488-L499))
**may** be added to the event. For example,
`MarketRequestedEdit(market_id, reason)` contains a `reason` which is not stored
anywhere in the chain storage.

[docs.zeitgeist.pm]: docs.zeitgeist.pm
