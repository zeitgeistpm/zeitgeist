---
name: Release Checklist
about: Template issue for creating release checklists.
title: Checklist for {{ env.VERSION }} release
labels: ''
assignees: ''
---

# Release Checklist

This is the release checklist for Zeitgeist {{ env.VERSION }}. All following checks should be completed before publishing a new release of the Zeitgeist runtime or client.

## Runtime Releases

These checks should be performed on the codebase prior to forking to a release-candidate branch.

- [ ] Verify `spec_version` has been incremented since the last release for any native runtimes from any existing use on public (non-private/test) networks.
- [ ] Verify previously completed migrations are removed for any public (non-private/test) networks.
- [ ] Verify pallet and extrinsic ordering has stayed the same. Bump `transaction_version` if not.
- [ ] Verify new extrinsics have been correctly whitelisted/blacklisted for proxy filters.
- [ ] Verify benchmarks have been updated for any modified runtime logic.

The following checks can be performed after we have forked off to the release-candidate branch or started an additional release candidate branch (rc-2, rc-3, etc)

- [ ] Verify the new migrations complete successfully, and the runtime state is correctly updated for any public (non-private/test) networks.
- [ ] Verify the SDK is up-to-date with the latest changes.
- [ ] Push runtime upgrade to local clone of Battery Station and ensure the upgrade is executed without errors.
- [ ] Push runtime upgrade to Battery Station and ensure network stability for 24 hours.
- [ ] Push runtime upgrade to Zeitgeist mainnet.

## All Releases
- [ ] Check that the new client versions have run on the network without issue for 12 hours.
- [ ] Check that a draft release has been created at https://github.com/zeitgeistpm/zeitgeist/releases with relevant release
notes.
- [ ] Check that build artifacts have been added to the draft-release

## Notes

### Build Artifacts

Add any necessary assets to the release. They should include:

- Linux binary
- GPG signature of the Linux binary
- SHA256 of the Linux binary
- Source code
- Wasm binaries of the runtime

### Release Notes

The release notes should list:

- The priority of the release (i.e. how quickly users should upgrade) - this is based on the max priority of any *client* changes.
- The version of the native runtime
- The proposal hashes of the runtime as built with [srtool](https://gitlab.com/chevdor/srtool)
- Any changes in this release that are still awaiting audit

The release notes may also list:

- Free text at the beginning of the notes mentioning anything important regarding this release.
- Notable changes separated into sections

### Spec Version

A runtime upgrade must bump the spec version number. This may follow a pattern with the client release (e.g. runtime v12 corresponds to v0.8.12, even if the current runtime is not v11).

### Old Migrations Removed

Any previous `on_runtime_upgrade` functions from the old upgrades must be removed to prevent them from executing a second time.

### New Migrations

Ensure that any migrations that are required due to storage or logic changes are included in the `on_runtime_upgrade` function of the appropriate pallets.

### Extrinsic Ordering

Offline signing libraries depend on a consistent ordering of call indices and
functions. Compare the metadata of the current and new runtimes and ensure that
the `module index, call index` tuples map to the same set of functions. In case
of a breaking change, increase `transaction_version`.

To verify the order has not changed:

1. Build the Zeitgeist client from source: `git clone https://github.com/zeitgeistpm/zeitgeist.git zeitgeist-release && pushd zeitgeist-release > /dev/null && cargo build --features=parachain`
2. Run the release-candidate binary using a local chain: `./target/debug/zeitgeist --chain=battery_station --tmp`
3. Use [`polkadot-js-tools`](https://github.com/polkadot-js/tools) to compare the metadata: `docker run --network host jacogr/polkadot-js-tools metadata wss://bsr.zeitgeist.pm ws://localhost:9944`

4. Things to look for in the output are lines like:
  - `[Identity] idx 28 -> 25 (calls 15)` - indicates the index for `Identity` has changed
  - `[+] Society, Recovery` - indicates the new version includes 2 additional modules/pallets.
  - If no indices have changed, every modules line should look something like `[Identity] idx 25 (calls 15)`

Note: Adding new functions to the runtime does not constitute a breaking change
as long as they are added to the end of a pallet (i.e., does not break any
other call index).

### Proxy Filtering

The runtime contains proxy filters that map proxy types to allowable calls. If
the new runtime contains any new calls, verify that the proxy filters are up to
date to include them.

### Benchmarks

Run the benchmarking suite with the new runtime and update any function weights
if necessary.

**Based on Parity's [release checklist for Polkadot](https://raw.githubusercontent.com/paritytech/polkadot/master/.github/ISSUE_TEMPLATE/release.md)**
