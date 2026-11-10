# Frost gryphon

The catalog now uses the pixel-art version; see [Pixel-art redesign](pixel-mascots.md). The painted atlas below describes the original version retained as a source asset.

MonitorAwake's second mascot is distinct from the neutral app icon. Select **Mascot > Frost gryphon** in the menu or **Preferences > Your mascot**.

`public/frost-gryphon-actions.png` is a transparent PNG atlas generated with ImageGen on November 7, 2026. A user-provided snowy World of Warcraft screenshot inspired its cool palette and painted fantasy appearance without copying the screenshot, interface or logos.

The character is a small gryphon with white/ice-blue feathers, amber eyes, leather/bronze harness and cyan crystal. The shared 2 × 2 atlas contract places waiting top left, pecking top right, fluttering bottom left and brooding bottom right. Eggs and speech bubbles are separate dynamic elements.

The original painted catalog used `rendering: auto`, while Chicken used `pixelated`; the current redesign uses pixelated rendering for both. States and deadlines are shared, with no mascot-specific timer store. Persistent identifier `frost_gryphon` introduced schema 3 and an explicit migration with exact backups. Migration preserves the existing selection and personal phrases.

Windows verification on November 7, 2026: 21 Rust tests, 12 frontend tests, Clippy, Svelte and HTTP checks passed. Isolated native profiles verified waiting/brooding captures with one egg, and **My timers** with an active timer and five history entries. Updated executable and NSIS installer were generated. Desktop Linux was not verified.
