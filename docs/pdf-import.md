# Opening and editing PDF pages

Use **File > Open** or drop a PDF onto the document tab area. The **Import PDF** dialog lets you select pages (or embedded images), choose the crop box, resolution, colour mode and bit depth, then open the selection as documents. Ctrl/Cmd-click toggles selections and Shift-click selects a range. Password-protected files prompt for their password.

Imported pages are raster images at the selected resolution. You can paint, select, transform and add new layers, then use **Save As** to save a layered `.psd` or `.pcraft` document. This does not reconstruct the PDF's original text objects or a PSD layer structure. A scan containing only one image may open directly, without the dialog.

Pages opened through the import dialog are untitled and require a new save destination. Keep the original PDF and use a separate destination for edited work. PDF export still produces one flattened page; save as PSD or PhotoCraft to retain the new layers.

## Implementation and attribution

This integration preserves the commits of [PR #6990](https://github.com/storytold/photocraft/pull/6990) by Unemars, using the pure-Rust hayro renderer. It combines that work with upstream `5f9b4391874aabc9deb2fcb905267613e4d93ec8`, retaining current RAW demosaic/resolution preferences, PSD recovery, CLI overwrite protection, and TIFF/JPEG export settings. PDF is included in the desktop and browser open-file filters. The added workflow regression verifies PDF page import, a new layer, and PSD/native save at both 8 and 16 bits.

Place Embedded stores the selected rasterised page as a native smart-object source, preserving the selection when its cache is rebuilt or the document is reopened. Place Linked rejects PDF sources until linked page-selection metadata is supported. Password input is masked and kept outside inspectable dialog fields; automation may set `password` but cannot read it back.

PDF import does not preserve interactive forms, links, digital signatures or independently editable PDF text/vector objects. Some JBIG2 encodings are rejected, nonembedded fonts may be substituted, and colour/crop rendering has known differences described in the original PR. Input and decoded-image budgets apply. Do not treat the renderer as an exact Photoshop or Acrobat reproduction.

## Integration validation

Historical validation of an earlier uncommitted integration passed 452 I/O unit tests (2 ignored), the 31-crate layering check, and wasm checks for 24 crates plus the codecs HEIF feature. The I/O test run used a 32 MiB test-thread stack after the default stack overflowed in a Krita test. These results do not validate later upstream changes or replace tests of the final checkpoint.

Full native/CLI/UI completion, corpus completion, clippy, panic-hunt and a fresh integrated screenshot remain required before marking this integration ready. The checkpoint handoff records the exact current validation status. No release readiness is claimed.
