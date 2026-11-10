# Ember wyvern

The catalog now uses the pixel-art version; see [Pixel-art redesign](pixel-mascots.md). The painted atlas below describes the original version retained as a source asset.

The third mascot is selectable through **Mascot > Ember wyvern** or **Preferences > Your mascot**. Its Horde-inspired fantasy appearance combines a tawny body, copper mane, dark horns, crimson wings, scorpion tail and iron/leather harness. The application identity stays neutral.

`public/ember-wyvern-actions.png` is a transparent PNG generated with ImageGen on November 7, 2026. Its 2 × 2 grid contains waiting, sniffing, fluttering and brooding in the same order as the gryphon. The original uses soft `auto` rendering; the current redesign uses pixelated rendering. Eggs, deadlines and bubbles use shared mascot behavior.

Schema 4 adds `ember_wyvern`. Explicit v3 → v4 migration preserves mascot selection, notes, timers and personal phrases, writing an exact original backup first. Migration chains from formats 1 and 2 are also supported.

Windows verification: 22 Rust tests, 12 frontend tests, Svelte without errors/warnings, Clippy and HTTP checks passed. Native waiting/brooding poses with an egg and **My timers** with active/history entries were inspected. NSIS setup was regenerated. Desktop Linux was not verified.
