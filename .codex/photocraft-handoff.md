# PDF import continuation — 2026-10-29 (America/Sao_Paulo)

- state: em_andamento
- ready_for_delivery: false
- Reason: this checkpoint preserves the actual pending integration, replacing the earlier diagnostic-only checkpoint. Final native tests and mandatory checks are still pending. Do not create a PR or merge this checkpoint.
- Stable release: v0.17.0, verified as published and non-prerelease.
- Base and last observed upstream main: 5f9b4391874aabc9deb2fcb905267613e4d93ec8. The complete upstream changes since the previous integration were applied with git apply --3way; no conflicts remain in the working tree. Final remote head is reported separately.
- Checkpoint branch: automation/photocraft-2026-10-29.
- Origin: feat/pdf-open / https://github.com/storytold/photocraft/pull/9117; remote PR head 3440cc08ff4a40cac067f4270bb5dee50a4823a3, open and unmerged when checked.
- Preserved history: helper 71fa2b15d3f24d421789a22a6bc15ff440872631 by vangel76; original import contribution by Unemars (#6990). No contributor credits or licences were removed.

## Behavior and acceptance

Current PR behavior under review: rebuilding PDF smart-object contents could use the default page instead of the selected one; password input lived in inspectable dialog fields; CCITT Columns could narrow unsafely.

Pending implementation: embedded placement stores the imported raster page as a native smart-object source, preserving selection/crop/depth without retaining the PDF password. Linked PDF placement is rejected explicitly until linked selection metadata exists. Password entry is masked, kept private and set through a write-only automation path; held-file identity uses the actual dialog ID. CCITT Columns outside the u32 range are rejected. Engine/UI/CLI import paths have been adapted to the current module structure and egui API.

Acceptance: selected-page cache rebuild and native save/reopen regressions pass at 8/16 bits; password is absent from ui.inspect; dialog fields cannot redirect a held file; hostile CCITT dimensions fail safely. All applicable repository gates must pass before ready_for_delivery becomes true.

## Changed files and scope

The inherited PDF feature and integration change 57 files versus the recorded upstream: I/O PDF renderer/options and tests, engine file commands/jobs and tests, UI import dialog/routing/jobs plus translations, desktop/web file filters, CLI options and automation, Cargo metadata, attribution and format documentation/scorecard. This is continuation and preservation of an existing large contribution, not a second feature. No other contribution was started. See the commit diff for the exact file list.

## Validation

This continuation:
- git apply --3way of the complete upstream interval: exit 0, no unresolved files.
- git diff --check: exit 0.
- Rust toolchain restored to 1.95.0 after the environment lost it; no repository dependency changes were requested.
- cargo fmt --all, followed by cargo fmt --all --check: both exit 0; the check produced no formatting diff.
- Focused command started: CARGO_TARGET_DIR=/tmp/pdf-continue-target CARGO_BUILD_JOBS=3 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 RUST_MIN_STACK=33554432 cargo test --config 'profile.dev.package."*".opt-level=0' --config 'profile.test.opt-level=0' -p photocraft-engine -p photocraft-ui-egui -p photocraft-cli --lib pdf -- --test-threads=2. A completed successful result has not yet been established for this checkpoint.
- Implementation checkpoint: 4654927c777f71fc6c1e24670870bdf679d9b200. Its code is preserved, but it is not a successfully tested implementation commit. Do not infer validation from dependency-download or compilation progress.
- GitHub tree for that implementation: f3d425bc28a3b9cb0c70fccf6957c64cdf1076d0, identical to local git write-tree.
- Pre-publication upstream recheck: still 5f9b4391874aabc9deb2fcb905267613e4d93ec8.
- Focused Rust test command had not completed beyond dependency preparation when this handoff was saved; no successful current test result is claimed.

Historical evidence (earlier uncommitted tree, not this commit): 452 I/O tests passed, 2 ignored, with 32 MiB thread stacks; layers passed for 31 crates; wasm passed for 24 crates plus codecs HEIF. The earlier full I/O run aborted on a Krita stack overflow with the default stack; the larger-stack rerun passed. A corpus run began but completion was not established.

Pending: complete native engine/UI/CLI and automation tests, clippy --all-targets -D warnings, fresh layers/wasm, panic-hunt, corpus and offscreen UI review. These are blocking readiness checks. No PR was opened, no comment was sent upstream and main was not modified. The original PR branch is not updated by this checkpoint.

## Next step and future PR text

Resume from this checkpoint, finish focused tests and mandatory gates, fix any verified failures, then remove this coordination file from the intended upstream diff. Update existing PR #9117 rather than opening a duplicate PDF PR. Do not describe the branch as release-ready while checks remain pending.

Title: Fix PDF import integration and preserve selected smart-object pages

Description: Continue Unemars's PDF import work and vangel76's integration, adapting it to current PhotoCraft modules and egui. Preserve the selected rasterised PDF page in embedded smart-object sources, mask and isolate PDF passwords, and reject overflowing CCITT dimensions. Pages remain raster images; editable PDF text/vector objects and PSD layers are not reconstructed. Linked PDF placement is explicitly unsupported. Final integrated validation is pending; this checkpoint is not ready for a draft PR.
