# Release ferese-shape

The crate version is `0.1.0`; the matching Git tag is `v0.1.0`. The library has
no dependencies and requires Rust 1.85 or newer.

## Check the package

```sh
cargo fmt --check
cargo test --release
cargo clippy --all-targets -- -D warnings
cargo package --list
cargo publish --dry-run
```

The archive is `target/package/ferese-shape-0.1.0.crate`. It includes the Rust
sources, shader source, build script, tests, README, changelog, release instructions
and MIT license.
The build script generates the WGSL controls during compilation; no generated
shader needs to be checked in.

For a local preview with uncommitted documentation changes, add `--allow-dirty`
to the packaging or dry-run command. Commit the final changes before publishing.

## Align the tag

Before publishing, check that the working tree is clean and the tag identifies
the commit being packaged:

```sh
git status --short
git rev-parse HEAD
git rev-parse 'v0.1.0^{commit}'
```

The two commit IDs must match. The existing `v0.1.0` tag already identifies the
completed library code. If release preparation adds a commit, reconcile that
tag before publishing. Updating an existing remote tag requires an explicit
maintainer decision; the workflow does not move tags.

## Publish

The **Release** GitHub Actions workflow accepts the tag and runs formatting,
tests, lints and a publication dry run. It uploads the verified `.crate` as a
workflow artifact. Its **Publish to crates.io and GitHub** option is off by
default.

To publish through the workflow, configure the repository secret
`CARGO_REGISTRY_TOKEN` with permission to publish `ferese-shape`, select
`v0.1.0`, and enable that option. The workflow publishes the crate, then creates
the GitHub release for the same tag and attaches the crate archive. Release notes
come from `CHANGELOG.md`.

To publish from a clean checkout of the tag instead:

```sh
cargo login
cargo publish
gh release create v0.1.0 target/package/ferese-shape-0.1.0.crate \
  --verify-tag --title 'ferese-shape 0.1.0' --notes-file CHANGELOG.md
```

Cargo uses the token stored by `cargo login`. A published version cannot be
overwritten. If `0.1.0` is already published, verify its contents and create the
matching GitHub release without publishing it again.
