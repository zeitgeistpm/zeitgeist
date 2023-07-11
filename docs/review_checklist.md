- [ ] All todos contain a reference to an issue like this: `TODO(#999)`.
- [ ] The PR links relevant issues and contains a detailed description.
- [ ] All relevant labels were added.
- [ ] The docstrings are up to date.
- [ ] If the PR adds or changes extrinsics or functions used by extrinsics:
  - [ ] The _Weight_ section in the documentation is up to date.
  - [ ] The benchmarks are up to date.
  - [ ] The call filters were adjusted.
  - [ ] The extrinsics emit all the required events (see [Events](#events)
        below).
- [ ] The module `README.md` is up to date.
- [ ] [docs.zeitgeist.pm] is up to date.
- [ ] `docs/changelog_for_devs.md` is up to date, specifically:
  - [ ] Changes relevant to the Frontend Team (extrinsics changed, new
        functions) are mentioned here.
  - [ ] All new events are explained so they can easily be integrated into the
        indexer.
  - [ ] Breaking changes are marked as such.
  - [ ] The file is formatted with `prettier -w docs/changelog_for_devs.md`.
- Sanity tests:
  - [ ] The local temporary development node produces blocks:
        `cargo run --profile=production -- --tmp`.
  - [ ] The node syncs with Zeitgeist and Battery Station:
        `cargo run --profile=production --features=parachain`,
        `cargo run --profile=production --features=parachain -- --chain=battery_station`.
  - [ ] `try-runtime` passes on Zeitgeist and Battery Station.
- [ ] Code quality:
  - [ ] Avoidable compiler warnings were resolved.
  - [ ] Integer arithmetic is only saturated/checked and all panickers are
        removed.
  - [ ] Code contained in macro invocations (benchmarks,
        `runtime/common/src/lib.rs`, `decl_runtime_apis!`) is correctly
        formatted.
  - [ ] All `*.toml` files are formatted with `taplo` (run
        `taplo format --check`).
  - [ ] All copyright notices are up to date.
  - [ ] Enums are sorted alphabetically, except for enums used in storage (to
        prevent migrations), errors and events.
- [ ] In case an action is required by the Frontend Team, an issue was added to
      zeitgeistpm/ui.
- [ ] In case the PR adds a new pallet, the pallet is added to the benchmark
      configuration in `scripts/`.
- [ ] In case configuration items or storage elements were changed: Necessary
      storage migrations are provided.
- [ ] In case configuration values changed: The implications have been discussed
      with the
      [code owners](https://github.com/zeitgeistpm/zeitgeist/blob/main/CODEOWNERS).
- [ ] If the changes include a storage migration:
  - [ ] The affected pallet's `STORAGE_VERSION` was bumped.
  - [ ] Try-runtime checks were added and the following conditions were ensured:
    - [ ] The storage migration bumps the pallet version correctly.
    - [ ] The try-runtime _fails_ if you comment out the migration code.
    - [ ] The try-runtime passes without any warnings (substrate storage
          operations often just log a warning instead of failing, so these
          warnings usually point to problem which could break the storage).

## Events

_All_ modifications of the on-chain storage **must** be broadcast to indexers by
emitting a high-level event. The term _high-level_ event refers to an event
which may or may not contextualize _low-level_ events emitted by pallets that
Zeitgeist's business logic builds on, for example pallet-balances. Examples of
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
