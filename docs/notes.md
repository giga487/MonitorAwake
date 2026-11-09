# Note desktop e frasi ricorrenti

L'app mantiene un'identita neutra. Il simbolo di MonitorAwake e `public/app-icon.svg` (nota e orologio); la gallina e una mascotte selezionabile, non il marchio del formato dati.

## Note

Dal menu della mascotte, **New note** crea una finestra indipendente. **My notes** elenca tutte le note e permette di aprirle, mostrarle tutte o nasconderle. La configurazione web contiene la stessa sezione e permette di modificare le note anche usando il solo server locale.

Ogni nota ha titolo, testo semplice, quattro colori e puntina per il primo piano. La barra superiore si trascina; i bordi della finestra permettono il ridimensionamento. La croce nasconde la nota, senza cancellarla. La cancellazione si trova nel pulsante cestino e richiede la conferma nella nota. Alla riapertura dell'app tornano solo le note lasciate visibili.

Il testo si salva automaticamente dopo una breve pausa nella digitazione. **Saving**, **Unsaved** e **Saved** distinguono lo stato reale. Un conflitto con un altro editor non sovrascrive il testo: si puo ricaricare la nota salvata oppure scegliere esplicitamente **Save my version**. Posizione e aspetto si salvano separatamente dalla revisione del contenuto, per non creare conflitti quando si sposta la finestra.

Prima della chiusura esplicita dell'app, ogni finestra nota riceve una richiesta di salvataggio. Se una finestra non conferma o una scrittura fallisce, l'app rimane aperta e mostra un errore. Come per qualsiasi salvataggio asincrono, un arresto forzato puo perdere modifiche che non hanno ancora raggiunto lo stato **Saved**.

**Add a reminder** collega un timer alla nota: la scadenza rimane gestita dal core Rust e compare come un uovo, come gli altri timer. Una nota senza promemoria non genera uova. Nascondere la nota non annulla il promemoria; eliminarla cancella il timer collegato o conferma un avviso gia scaduto. La pulizia dello storico rimuove anche eventuali riferimenti a timer non piu presenti.

Le finestre si sincronizzano con lo stesso servizio usato dalle API HTTP. Su desktop Linux, soprattutto Wayland, primo piano e posizionamento dipendono dal compositor: se il posizionamento non e consentito, la finestra si apre comunque. Il ripristino limita le coordinate a uno schermo disponibile. Compilazione e test automatici non verificano la resa desktop Linux.

## Frasi ricorrenti

In **Preferences > Recurring phrases** si possono salvare fino a 50 frasi di 180 caratteri, una per riga, o disattivare le vignette. Le frasi vengono scelte localmente; non e necessario un modello linguistico.

La vignetta e la terza azione casuale della mascotte, insieme a beccata e svolazzo: dura sei secondi e non ripete subito l'ultima frase se ci sono alternative. Le frasi si sospendono quando ci sono notifiche da covare o confermare, quando la mascotte non e visibile o quando il movimento e disattivato/ridotto.

La vignetta usa una finestra trasparente separata, non prende il focus e lascia passare i clic. Non ridimensiona e non sposta la gallina. Il menu aperto ha la precedenza sulla vignetta.

**Preview phrase** nel menu mostra subito una delle frasi configurate, anche con le animazioni disattivate. Su Windows la visibilita della vignetta usa `SW_SHOWNOACTIVATE` in un adattatore Rust dedicato, per non attivarla durante la scrittura.

## Verifiche

`npm run test:widget` include la coda di salvataggio condivisa da autosave/chiusura/uscita e la selezione delle frasi. I test Rust coprono conflitti, visibilita, promemoria, migrazioni e file protetti da sovrascrittura. `scripts/smoke-server.py` verifica le API reali con un profilo temporaneo isolato, incluso il riavvio e la conservazione dei backup. La verifica visuale nativa e distinta e va riportata separatamente.

Verifica del 7 novembre 2026 su Windows: 20 test Rust, 11 test frontend, controlli Svelte, Clippy e prova HTTP passati. Sull'app nativa sono stati controllati inserimento e autosave del testo, creazione dal menu, cancellazione con conferma, posizione, primo piano, chiusura senza cancellazione, coordinamento del salvataggio all'uscita e ripristino al riavvio. Nota e vignetta sono state catturate e ispezionate; la vignetta e stata verificata per testo configurato, assenza di focus, passaggio dei clic e chiusura automatica. Nessuna verifica desktop Linux eseguita.
