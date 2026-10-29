# PhotoCraft contribution handoff

- Date/timezone: 2026-10-29, America/Sao_Paulo.
- Objective: assess continuation of the existing PDF import contribution before starting another improvement.
- State: bloqueado.
- ready_for_delivery: false.
- Reason: diagnostic checkpoint only. The existing integration exceeds the small-run scope and has incomplete validation. Do not open a PR from this branch; removing this coordination file leaves no functional diff.
- Stable release verified on GitHub: v0.17.0 (not draft or prerelease).
- Base SHA: 5f9b4391874aabc9deb2fcb905267613e4d93ec8.
- Last upstream main SHA observed before publication: 5f9b4391874aabc9deb2fcb905267613e4d93ec8.
- Integration: this diagnostic branch starts directly at that full upstream commit. No implementation from the older PDF worktree was copied or represented as integrated.
- Checkpoint branch: automation/photocraft-2026-10-29.
- Previous automation checkpoints: none returned by GitHub's matching refs query for heads/automation/photocraft-.
- Related existing work: IgorsSantana/photocraft:feat/pdf-open, upstream PR https://github.com/storytold/photocraft/pull/9117.
- Verified PR head: 3440cc08ff4a40cac067f4270bb5dee50a4823a3; open, not merged, mergeable=false, mergeable_state=dirty.

## Candidate and acceptance criterion

Only one candidate was assessed: finish the existing PDF import integration. The user requested that their contributions be addressed one at a time, starting with this PR.

Current behavior: upstream provides PDF export, while PDF import is still listed as missing in G10 and partial FILE-215-10. The existing PR implements import but cannot currently merge cleanly. This is already implemented work awaiting integration and validation, not a reason to create a duplicate implementation or PR.

Expected behavior: retain a selected PDF page and its import settings when an embedded smart object is saved, reopened or its cache rebuilt; mask password input and exclude passwords from inspectable dialog state; reject unsafe CCITT dimensions. Keep source PDFs unchanged and preserve current upstream behavior.

Acceptance: regressions for page preservation at 8/16 bits, password privacy/dialog identity and CCITT overflow pass, the existing contribution integrates against current main, and all applicable repository gates pass. Rasterisation does not reconstruct editable PDF text/vector objects or PSD layers.

Scope decision: the preserved local integration differs from its last integrated upstream (69b8fd9c14b276b07c5d5d2cc88bba08dc4474bc) in 57 files, approximately 5,474 added and 71 removed lines, including the inherited feature. Even a small review fix depends on this larger integration. Completing and validating it is not a three-file/~200-line task. No second feature was started and no code was modified in this run.

## Preservation and provenance

The existing worktree at ../photocraft-pdf remains untouched by this run. It contains a pending integration based on helper commit 71fa2b15d3f24d421789a22a6bc15ff440872631, including vangel76's conflict-resolution history and the original Unemars contribution (#6990). Its source branch is feat/pdf-open. Do not reset it, overwrite the remote PR branch, or discard its uncommitted changes.

This checkpoint does not back up or publish those uncommitted functional changes. Resume the original integration separately with an appropriate validation budget; if that worktree is unavailable, recover the remote PR and helper histories and reassess what remains unpublished.

Files changed in this checkpoint: .codex/photocraft-handoff.md only.
Functional diff: none.
Validated implementation commit: none in this run. The verified baseline is the base SHA above; the final diagnostic commit is reported outside this file.

## Evidence from this run

- GitHub GET releases/latest: success; v0.17.0, draft=false, prerelease=false.
- GitHub GET branches/main, initial and pre-publication: success; same SHA recorded above.
- GitHub matching-refs/heads/automation/photocraft-: success; empty list.
- GitHub GET pulls/9117: success; open, unmerged, conflicting, head recorded above.
- Fork repository permissions: push=true.
- git fetch upstream main and git worktree add --detach <base>: succeeded; isolated worktree HEAD matches the verified SHA.
- git switch -c automation/photocraft-2026-10-29: exit 0.
- Read root AGENTS.md, docs/contributing.md, architecture, ROADMAP, G10 and FILE-215-10; inspected the current I/O entry points.
- git diff --shortstat upstream/latest in the existing PDF worktree: exit 0; scope evidence above.
- cargo --version: exit 127, command not found. Environment limitation, not an upstream failure. No Rust installation or expensive build was started for this diagnostic-only checkpoint.
- Handoff-only diff validation and remote commit verification are performed before reporting completion.

## Historical evidence — not current validation

Preserved logs from the earlier, uncommitted PDF integration report 452 passing I/O unit tests (2 ignored) with RUST_MIN_STACK=33554432 and a wasm rerun passing 24 crates plus the codecs HEIF feature. Earlier attempts included compilation errors and a Krita test stack overflow with the default test-thread stack. These logs are not proof for this checkpoint or today's upstream. Full native/CLI/UI, corpus, clippy, panic-hunt and visual validation have not been established for a final committed integration.

## Next minimum step

Continue PR #9117 in its original branch/worktree, reconcile the complete current upstream history, then finish focused regressions and applicable native/wasm/clippy/layers/panic-hunt/corpus/visual gates. Preserve Unemars and vangel76 credits. Publish to the existing fork PR branch only after reviewing the validated state. Do not create a second PDF PR. The delivery task must stop at ready_for_delivery=false; it must not turn this diagnostic branch into a PR.

## Future PR text (not ready to publish)

Title: Fix PDF import integration and preserve selected smart-object pages

Description:
Continue the existing PDF import contribution, preserving Unemars's original implementation and vangel76's integration work. Intended changes preserve the selected rasterised page when embedded smart-object contents are rebuilt, keep PDF passwords out of inspectable dialog fields, and reject overflowing CCITT dimensions. PDF pages remain raster images; original text/vector objects and PSD layers are not reconstructed. Linked PDF placement remains unsupported in the pending implementation.

This diagnostic checkpoint contains no implementation and is not ready for a draft PR. Validation of the final integrated code remains pending; historical test results must not be presented as validation of a new commit.
