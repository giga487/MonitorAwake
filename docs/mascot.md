# Screen-faced chicken and mascot behavior

`public/mascot.png` is the original transparent PNG generated with the integrated ImageGen tool on November 6, 2026. The user selected a pixel-art chicken with a red comb, wings, feet and phosphor-green screen face in place of the robot-terminal concept. The neutral application icon is `public/app-icon.svg`; regenerate platform icons with `npm run tauri -- icon public/app-icon.svg --output src-tauri/icons`.

The original remains available for static illustrations. The widget uses `public/chicken-actions.png`, a transparent 1254 × 1254 RGBA atlas generated on November 7 using the original as reference. Its 2 × 2 grid has four 627 × 627 cells: waiting (top left), pecking (top right), fluttering (bottom left), and brooding (bottom right). Eggs are drawn separately from actual timer state.

## States and notifications

- **Waiting:** looks upward with a slight tilt.
- **Random action:** pecks or attempts to flutter every 10–25 seconds without pending timers. Each gesture lasts less than two seconds. Speech has its own independent interval.
- **Waiting for notification:** broods with gentle breathing and an egg per active timer. Enabled speech displays a six-second bubble after each configurable pause (default 5–10 minutes), while the mascot stays seated. No pecking or fluttering during brooding.
- **Notification ready:** signals expiry; ready eggs remain red and cracked until acknowledged.

Each egg follows its own deadline: green above 50% remaining, yellow between 50% and 20%, red in the final 20%. Marks also distinguish phases without color; numbers, tooltips and accessible text identify notifications. Acknowledged/cancelled eggs disappear. A crowded nest can scroll without limiting represented eggs.

`src/lib/mascot.ts` derives visual states from Rust timers. `src/lib/Mascot.svelte` owns poses/movement, not timer expiry or notification delivery. Active timers interrupt pecking/fluttering. Ready notifications and a hidden page stop speech. **Gentle animations** and `prefers-reduced-motion` stop movement without hiding eggs or state.

Selection is stored in `settings.mascot`. Rust's `MascotKind` enum has a Serde default for supported older files. The frontend catalog declares assets and rendering; each new animated mascot should implement waiting, random action and waiting for notification. The catalog includes Chicken, Frost gryphon, Ember wyvern and Woodland elf.

The atlas prompt requires consistent identity and equal aligned cells, transparent alpha, no text and no embedded eggs or nest. The original asset was preserved.

## Speech verification

Windows, November 7, 2026: the brooding-speech fix passed 13 frontend tests and Svelte checks. In an isolated profile with an active timer, two different phrases appeared automatically while brooding and the egg stayed visible. Bubbles did not take focus and disappeared after each phrase. Desktop Linux was not verified.

The configurable-pause increment passed 24 Rust tests, 14 frontend tests, Clippy and HTTP checks. A native Windows one-minute fixed pause delayed speech without interrupting brooding or the egg. Default 5–10 minute pauses and custom bounds were tested deterministically without real waits. NSIS setup was regenerated. No preferences browser was opened; desktop Linux was not verified.

## Original sprite prompt

Use case: stylized-concept. Asset type: one production desktop companion mascot sprite on transparent background. Primary request: a nerdy CHICKEN with a SCREEN FACE. Design a clearly recognizable small round hen robot: ivory/off-white rounded chicken body, red comb on top of the head, tiny red wattles just below the face panel, two feather-shaped little wings, a small fan of tail feathers, two short yellow-orange chicken legs with three-toed feet. The FACE ITSELF is a dark rounded rectangular CRT screen inset into the chicken's head, showing two expressive bright phosphor-green pixel eyes and a tiny pixel beak/mouth. The screen replaces the whole organic face; do not put a human face there. The character should read first as a chicken and second as a retro computer companion, not as a monitor with generic arms. Charming, alert, mildly mischievous nerd personality. Crisp 16-bit pixel art with coherent chunky pixel clusters and a dark outline; restrained retro palette, gentle pixel shading, no smooth vector illustration, no glossy 3D render. Front-facing with slight three-quarter body orientation, symmetrical readable screen face. Entire single character visible from comb to toes, centered square composition, about 85 percent of canvas, clear padding. Must be legible as a desktop pet at 120px high. Truly transparent alpha background. No scene, no ground plane, no cast shadow, no checkerboard, no words, no logo, no extra characters or props.

## Woodland elf

`public/elf.png` is a transparent sprite generated with ImageGen. Select **Preferences > Your mascot > Woodland elf**. The widget tilts the elf like a sentry and performs a small hop, with a pause between gestures.

### Blond variant prompt

Change only this pixel-art elf's hair color to natural honey blond, with golden strands, ochre shading and straw-colored highlights. Preserve the hairstyle and volume, pose, face, green eyes, long ears, green tunic, sword, leaf shield, proportions, full-figure framing and original pixel-art style. Keep the background transparent.
