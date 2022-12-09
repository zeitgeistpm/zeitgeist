-   [ ] Check if there are any TODOs which are not linked to an issue of the
        zeitgeist repository like this: `TODO(#999)`.
-   [ ] Does the PR link relevant issues and contain a detailed description?
-   [ ] If the PR changes the business logic, does it add the corresponding
        labels?
-   [ ] If the PR adds or changes functions:
    -   [ ] Are the doc strings up to date?
    -   [ ] Is the _Weight_ section in the documentation up to date?
    -   [ ] Are the benchmarks up to date?
-   [ ] Is the module `README.md` up to date?
-   [ ] Is [docs.zeitgeist.pm] up to date?
-   [ ] Is `docs/changelog_for_devs.md` up to date, specifically:
    -   [ ] Are changes relevant to the Frontend Team (extrinsics changed, new
            functions) mentioned here?
    -   [ ] Are all new events explained so they can easily be integrated into
            the indexer?
    -   [ ] Is the file formatted with `prettier -w docs/changelog_for_devs.md`?
-   Sanity tests:
    -   [ ] The local node produces blocks.
    -   [ ] `try-runtime` passes on Zeitgeist and Battery Station.
    -   [ ] All runtime benchmarks pass on Zeitgeist and Battery Station (don't
            just test against the mock!).
    -   [ ] The node syncs with Zeitgeist and Battery Station.
-   [ ] Code quality:
    -   [ ] Are there any compiler/clippy warnings?
    -   [ ] Is integer arithmetic saturated/checked and are all panickers
            removed?
    -   [ ] Is code contained in macro invocations (benchmarks,
            `runtime/common/src/lib.rs`, `decl_runtime_apis!`) correctly
            formatted?
    -   [ ] Are all `*.toml` files formatted with `taplo`?
    -   [ ] Are all copyright notices up to date?
-   [ ] If an action is required by the Frontend Team, add an issue to
        zeitgeistpm/ui.
-   [ ] If the PR adds a new pallet, is the pallet added to the benchmark
        configuration in `scripts/`?
-   [ ] If you are changing/removing configuration values, storage items: Do the
        changes require a storage migration?
-   [ ] If you are adding/changing configuration values on the mainnet: Have the
        implications been discussed with the product owners?
-   [ ] If the changes include a storage migration:
    -   [ ] Is the version number in the pallet bumped?
    -   [ ] Add try-runtime checks and ensure the following:
        -   [ ] The storage migration bumps the pallet version correctly.
        -   [ ] The try-runtime _fails_ if you comment out the migration code.
        -   [ ] The try-runtime passes without any warnings (substrate storage
                operations often just log a warning instead of failing, so these
                warnings usually point to problem which could break the
                storage).

[docs.zeitgeist.pm]: docs.zeitgeist.pm
