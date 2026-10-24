# Opening and editing PDF pages

Use **File > Open** or drop a PDF onto the document tab area. The **Import PDF** dialog lets you select pages (or embedded images), choose the crop box, resolution, colour mode and bit depth, then open the selection as documents. Ctrl/Cmd-click toggles selections and Shift-click selects a range. Password-protected files prompt for their password.

Imported pages are raster images at the selected resolution. You can paint, select, transform and add new layers, then use **Save As** to save a layered `.psd` or `.pcraft` document. This does not reconstruct the PDF's original text objects or a PSD layer structure. A scan containing only one image may open directly, without the dialog.

Pages opened through the import dialog are untitled and require a new save destination. Keep the original PDF and use a separate destination for edited work. PDF export still produces one flattened page; save as PSD or PhotoCraft to retain the new layers.

## Implementation and attribution

This integration preserves the commits of [PR #6990](https://github.com/storytold/photocraft/pull/6990) by Unemars, using the pure-Rust hayro renderer. It combines that work with upstream `96042b8e7a7fcaf5d3d37997aaac824022a7def5`, retaining current RAW demosaic/resolution preferences, PSD recovery, CLI overwrite protection, and TIFF/JPEG export settings. PDF is included in the desktop and browser open-file filters. The added workflow regression verifies PDF page import, a new layer, and PSD/native save at both 8 and 16 bits.

PDF import does not preserve interactive forms, links, digital signatures or independently editable PDF text/vector objects. Some JBIG2 encodings are rejected, nonembedded fonts may be substituted, and colour/crop rendering has known differences described in the original PR. Input and decoded-image budgets apply. Do not treat the renderer as an exact Photoshop or Acrobat reproduction.

## Integration validation

The prior validation run passed the 63 PDF-focused tests (13 engine, 36 I/O and 14 UI), `cargo xtask layers`, `cargo xtask wasm`, and the browser application's wasm check. The PDF workflow test saves an edited copy through `file.saveACopy` and checks both PSD and PhotoCraft at 8 and 16 bits.

The complete native test run was interrupted by the environment killing the UI test compiler (SIGKILL). The corpus run stopped because the local `photocraft-corpus` checkout was unavailable. Full-suite completion, clippy, panic-hunt completion, and a fresh integrated UI screenshot remain review gates; this integration is published as a draft, not a validated release.
