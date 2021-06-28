---
name: Release Checklist
about: Template issue for creating release checklists.
title: Checklist for vX.Y.Z release
labels: ''
assignees: ''

---

# Release Checklist

This is the release checklist for Zeitgeist vX.Y.Z. All following checks should be completed before publishing a new release of the Zeitgeist runtime or client.

## Runtime Releases

These checks should be performed on the codebase prior to forking to a release-candidate branch.

- [] Verify `spec_version` has been incremented since the last release for any native runtimes from any existing use on public (non-private/test) networks.
- [] Verify previously completed migrations are removed for any public (non-private/test) networks.
- [] Verify pallet and extrinsic ordering has stayed the same. Bump `transaction_version` if not.
- [] Verify new extrinsics have been correctly whitelisted/blacklisted for proxy filters.
- [] Verify benchmarks have been updated for any modified runtime logic.

The following checks can be performed after we have forked off to the release-candidate branch or started an additional release candidate branch (rc-2, rc-3, etc)

- [] Verify the new migrations complete successfully, and the runtime state is correctly updated for any public (non-private/test) networks.
- [] Verify the SDK is up-to-date with the latest changes.
- [] Push runtime upgrade to Battery Park and ensure network stability.

## All Releases
- [] Check that the new client versions have run on the network without issue for 12 hours.
- [] Check that a draft release has been created at https://github.com/zeitgeistpm/zeitgeist/releases with relevant release
notes.
- [] Check that build artifacts have been added to the draft-release

**Based on Parity's [release checklist for Polkadot](https://github.com/paritytech/polkadot/issues/2961)**
