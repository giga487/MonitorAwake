# Frost gryphon

Seconda mascotte di MonitorAwake, distinta dall'icona neutra dell'app. Si seleziona da **Mascot > Frost gryphon** nel menu o da **Preferences > Your mascot**.

`public/frost-gryphon-actions.png` e un atlante PNG a sfondo alpha trasparente, generato con ImageGen il 7 novembre 2026. Il riferimento fornito dall'utente e una schermata di World of Warcraft nella neve: ha ispirato la palette fredda e la resa fantasy dipinta, senza riprodurre schermata, interfaccia o loghi.

Il personaggio e un piccolo grifone con piume bianche e azzurre, occhi ambra, bardatura in cuoio e bronzo e cristallo ciano. La griglia 2 × 2 segue il contratto condiviso delle mascotte: attesa in alto a sinistra, beccata in alto a destra, svolazzo in basso a sinistra, cova in basso a destra. Uova e vignette restano elementi dinamici separati.

Il catalogo dichiara `rendering: auto` per conservare la resa morbida, mentre Chicken mantiene `pixelated`. Gli stati e le scadenze sono gli stessi; nessuna mascotte possiede una copia distinta dei timer. Il nuovo identificatore persistente `frost_gryphon` richiede lo schema 3, con migrazione esplicita e backup dai formati precedenti. La migrazione conserva la selezione esistente e non modifica le frasi personali.

Verifica Windows del 7 novembre 2026: 21 test Rust e 12 test frontend, Clippy, controlli Svelte e prova HTTP passati. L'app nativa e stata verificata su profili isolati con Frost gryphon selezionato: catturate e ispezionate le pose di attesa e cova con un uovo; verificato **My timers** con un timer attivo e cinque voci di storico. Esportati eseguibile e installer NSIS aggiornati. Desktop Linux non verificato.
