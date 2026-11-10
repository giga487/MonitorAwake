# Pixel-art redesign

Frost gryphon and Ember wyvern now use `public/frost-gryphon-pixel-actions.png` and `public/ember-wyvern-pixel-actions.png`, with `pixelated` rendering like Chicken. Earlier painted atlases are retained. IDs, preferences and stored data are unchanged: this asset-only update needs no migration.

Generated on November 7, 2026 with integrated ImageGen and transparent alpha. The original chicken is the style reference; earlier atlases provide species, colors and poses only. The new gryphon also provides consistency reference for the wyvern. Four square cells cover waiting, pecking/sniffing, fluttering and brooding. Eggs and speech bubbles remain separate.

## Prompt set

Shared specification: chunky 16-bit pixel art matching the reference chicken; bold dark plum stepped outlines; compact round body, large expressive simplified face, tiny feet; limited flat pixel-cluster shading; approximately 64x64 native pixels per cell, enlarged with nearest-neighbor. No painted fur, realistic anatomy, glossy 3D eyes, smooth gradients, eggs, nest, scenery, text or shadows. Four equal square cells in a 2x2 transparent atlas, consistent scale and identity, padding within each cell. Poses: upright looking up; head lowered; wings open with feet lifted; sitting low brooding.

Frost gryphon: white and ice-blue feathers, simple golden beak, amber pixel eyes, blue wings, tiny talons, round lion hindquarters, tufted tail, simple bronze collar with cyan gem. Natural bird face.

Ember wyvern: tawny body, copper mane simplified into a few spikes, short dark horns, amber pixel eyes, crimson bat wings, tiny paws, curved scorpion tail with red stinger, simple dark iron/red leather collar with green gem. Natural animal face. Match the new gryphon's pixel scale, outline thickness and shading.

Verification: 14 frontend tests passed; Svelte reported no errors/warnings. Native Windows waiting/brooding poses for both mascots were captured and inspected at widget size with a dynamic egg. NSIS setup was regenerated. Desktop Linux was not verified.
