## What this changes

<!-- One or two sentences. Why, not just what. -->

## Checks

- [ ] `npx tsc --noEmit`
- [ ] `cargo test`
- [ ] `cargo clippy --all-targets -- -D warnings`
- [ ] `cargo fmt --check`

## If it touches the write path

`save::apply_and_write` is the only way a save file is modified.

- [ ] There is a test for the failure mode, not only the happy path
- [ ] A save is still never modified without a backup existing first

## If it adds a plugin

- [ ] No bundled game artwork
- [ ] `identify` markers are specific enough not to match another game
- [ ] Ranges reflect values the game is known to accept