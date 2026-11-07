# Ember wyvern

Aggiornamento grafico: il catalogo usa ora la versione pixel art; dettagli e prompt in [Pixel-art redesign](pixel-mascots.md). I riferimenti agli atlanti dipinti qui sotto descrivono la prima versione, conservata come sorgente.

Terza mascotte, selezionabile da **Mascot > Ember wyvern** o **Preferences > Your mascot**. Il personaggio richiama il fantasy dell'Orda: corpo fulvo, criniera ramata, corna scure, ali cremisi, coda da scorpione e bardatura in ferro e cuoio. L'identita dell'app rimane neutra.

`public/ember-wyvern-actions.png` e un PNG trasparente generato con ImageGen il 7 novembre 2026. La griglia 2 x 2 contiene attesa, annusata, svolazzo e cova, nello stesso ordine del grifone. Il rendering e morbido (`auto`). Uova, scadenze e vignette usano il comportamento condiviso delle altre mascotte.

Lo schema 4 aggiunge `ember_wyvern`. La migrazione esplicita v3 -> v4 conserva mascotte selezionata, note, timer e frasi personali; salva prima una copia esatta del file originale. Sono supportate anche le catene dai formati 1 e 2.

Verifica Windows: 22 test Rust e 12 test frontend passati, controlli Svelte senza errori o warning, Clippy e prova HTTP passati. Verificate e ispezionate nell'app nativa le pose di attesa e cova con un uovo, oltre a My timers con timer attivo e storico. Installer NSIS rigenerato. Desktop Linux non verificato.
