# Pixel-art redesign

Frost gryphon ed Ember wyvern usano ora gli atlanti `public/frost-gryphon-pixel-actions.png` e `public/ember-wyvern-pixel-actions.png`, con rendering `pixelated` come Chicken. I vecchi atlanti dipinti sono conservati. Identificatori, preferenze e dati restano uguali: questo aggiornamento modifica solo le immagini e non richiede migrazioni.

Generati il 7 novembre 2026 con il tool integrato ImageGen, sfondo alpha trasparente. La gallina originale e il riferimento di stile; gli atlanti precedenti servono solo per specie, colori e pose. Il nuovo grifone e anche il riferimento di coerenza per la viverna. Quattro celle quadrate: attesa, beccata/annusata, svolazzo, cova. Uova e vignette restano separate.

## Prompt set

Specifiche condivise: chunky 16-bit pixel art matching the reference chicken; bold dark plum stepped outlines; compact round body, large expressive simplified face, tiny feet; limited flat pixel-cluster shading; approximately 64x64 native pixels per cell, enlarged with nearest-neighbor. No painted fur, realistic anatomy, glossy 3D eyes, smooth gradients, eggs, nest, scenery, text or shadows. Four equal square cells in a 2x2 transparent atlas, consistent scale and identity, padding within each cell. Poses: upright looking up; head lowered; wings open with feet lifted; sitting low brooding.

Frost gryphon: white and ice-blue feathers, simple golden beak, amber pixel eyes, blue wings, tiny talons, round lion hindquarters, tufted tail, simple bronze collar with cyan gem. Natural bird face.

Ember wyvern: tawny body, copper mane simplified into a few spikes, short dark horns, amber pixel eyes, crimson bat wings, tiny paws, curved scorpion tail with red stinger, simple dark iron/red leather collar with green gem. Natural animal face. Match the new gryphon's pixel scale, outline thickness and shading.

Verificati 14 test frontend e Svelte senza errori o warning. Nell'app nativa Windows sono state catturate e ispezionate le pose di attesa e cova di entrambe le mascotte a dimensione widget, con un uovo dinamico. Installer NSIS rigenerato. Desktop Linux non verificato.
