# Gallina con viso a schermo

Asset: `public/mascot.png`, PNG trasparente generato con il tool integrato imagegen il 6 novembre 2026. Gallina in pixel art con cresta rossa, ali, zampe e viso su un display verde fosforo, scelta dall'utente al posto del terminale robot. L'icona neutra dell'app usa invece `public/app-icon.svg`, con rigenerazione tramite `npm run tauri -- icon public/app-icon.svg --output src-tauri/icons`.

L'immagine originale rimane disponibile per le illustrazioni statiche. Il widget usa ora `public/chicken-actions.png`, un atlante PNG RGBA trasparente 1254 × 1254, generato con ImageGen il 7 novembre 2026 usando la gallina originale come riferimento. La griglia 2 × 2 contiene quattro pose da 627 × 627: attesa in alto a sinistra, beccata in alto a destra, svolazzo in basso a sinistra, cova in basso a destra. Non contiene uova: vengono disegnate separatamente per rappresentare i timer reali.

## Stati e notifiche

- **Waiting:** guarda in alto e si inclina leggermente.
- **Random action:** ogni 10–25 secondi senza notifiche, becca, tenta di svolazzare mentre le frasi hanno un intervallo indipendente. Beccata e svolazzo durano meno di due secondi; la vignetta sei secondi.
- **Waiting for notification:** cova con una respirazione lenta e un uovo per ogni timer attivo. Se le frasi sono abilitate, mostra una vignetta per sei secondi dopo ogni pausa casuale di 5-10 minuti, configurabile in Preferences > Recurring phrases, restando seduta sulle uova. Non becca e non svolazza durante la cova.
- **Notification ready:** segnala la scadenza; le uova pronte sono rosse e incrinate fino alla conferma dell'avviso.

Ogni uovo ha il proprio tempo: verde quando rimane più del 50%, giallo tra il 50% e il 20%, rosso nell'ultimo 20%. Le tacche distinguono le fasi anche senza colore; numero, tooltip e testo accessibile identificano le notifiche. Le uova confermate o annullate scompaiono. Un nido affollato si può scorrere con il mouse, senza limitare il numero di uova rappresentate.

`src/lib/mascot.ts` calcola gli stati visivi dai timer Rust. `src/lib/Mascot.svelte` gestisce pose e movimento; non scade timer e non invia notifiche. Beccata e svolazzo si interrompono quando arrivano timer attivi; le frasi si fermano quando una notifica scade e quando la pagina non è visibile. La preferenza **Gentle animations** e `prefers-reduced-motion` fermano i movimenti senza nascondere uova e stato.

La scelta della mascotte è salvata in `settings.mascot`, attualmente `chicken`. L'enum Rust `MascotKind` con default Serde conserva la compatibilità con i file precedenti. Il catalogo frontend dichiara l'atlante e la descrizione; prima di aggiungere una mascotte vanno realizzate tutte e tre le azioni richieste.

Il prompt dell'atlante richiede la stessa gallina robot in pixel art, quattro pose allineate in celle uguali, sfondo alpha trasparente, nessuna scritta, nessun nido e nessun uovo incorporato. L'asset originale non è stato sovrascritto.

## Verifica delle frasi durante la cova

Verifica della correzione delle frasi durante la cova (Windows, 7 novembre 2026): 13 test frontend passati, Svelte senza errori o warning, installer rigenerato. Su un profilo isolato con timer attivo sono comparse automaticamente due frasi diverse; posa di cova e uovo sono rimasti visibili. La vignetta non ha preso il focus ed e scomparsa dopo ogni frase. Desktop Linux non verificato.

## Prompt utilizzato

Aggiornamento pause configurabili: 24 test Rust e 14 test frontend passati, Clippy e prova HTTP passati. Nell'app Windows una pausa fissa configurata di un minuto ha ritardato la frase senza interrompere cova e uovo; la vignetta non ha preso il focus. L'intervallo predefinito 5-10 minuti e i limiti personalizzati sono verificati senza attese reali nei test della funzione di pianificazione. Installer NSIS rigenerato. La pagina delle preferenze non e stata aperta in un browser; desktop Linux non verificato.

Use case: stylized-concept. Asset type: one production desktop companion mascot sprite on transparent background. Primary request: a nerdy CHICKEN with a SCREEN FACE. Design a clearly recognizable small round hen robot: ivory/off-white rounded chicken body, red comb on top of the head, tiny red wattles just below the face panel, two feather-shaped little wings, a small fan of tail feathers, two short yellow-orange chicken legs with three-toed feet. The FACE ITSELF is a dark rounded rectangular CRT screen inset into the chicken's head, showing two expressive bright phosphor-green pixel eyes and a tiny pixel beak/mouth. The screen replaces the whole organic face; do not put a human face there. The character should read first as a chicken and second as a retro computer companion, not as a monitor with generic arms. Charming, alert, mildly mischievous nerd personality. Crisp 16-bit pixel art with coherent chunky pixel clusters and a dark outline; restrained retro palette, gentle pixel shading, no smooth vector illustration, no glossy 3D render. Front-facing with slight three-quarter body orientation, symmetrical readable screen face. Entire single character visible from comb to toes, centered square composition, about 85 percent of canvas, clear padding. Must be legible as a desktop pet at 120px high. Truly transparent alpha background. No scene, no ground plane, no cast shadow, no checkerboard, no words, no logo, no extra characters or props.

## Elfo dei boschi

La seconda mascotte è uno sprite trasparente in `public/elf.png`, generato con ImageGen. Si sceglie in **Preferenze → La tua mascotte**. Nel widget l’elfo si inclina come una sentinella e fa un saltello, con una pausa tra i due gesti.

### Prompt della variante bionda

Modifica solo il colore dei capelli di questo elfo in pixel art: biondo miele naturale con ciocche dorate, ombre ocra e riflessi color paglia. Conserva taglio e volumi dei capelli, posa, viso, occhi verdi, orecchie lunghe, tunica verde, spada, scudo a foglia, proporzioni, inquadratura dell'intera figura e stile pixel art dell'originale. Mantieni lo sfondo trasparente.
