# MonitorAwake

Un piccolo assistente desktop in Rust: mascotte Tauri, timer e configurazione HTTP locale in Svelte. Tutto funziona sul computer dell'utente, senza account o servizi cloud.

L'interfaccia, i messaggi e le notifiche dell'app sono in inglese. Esempi: `timer tea 5 minutes`, `remind me in 30 minutes`, `how much time is left?`, `add 5 minutes to timer tea`, `cancel timer tea`. I comandi italiani restano compatibili; i nomi dei timer salvati non vengono tradotti.

## Prima versione

Oltre a **Chicken** sono disponibili **Frost gryphon**, un grifone del gelo, ed **Ember wyvern**, una viverna dalle ali cremisi, entrambi con grafica fantasy dipinta. Dal menu **My timers** si accede direttamente a timer attivi, avvisi e storico.

Le **Desktop notes** si creano dal menu con **New note** e si ritrovano in **My notes**: testo e titolo, quattro colori, primo piano, ridimensionamento, salvataggio automatico e promemoria opzionali. Chiudere nasconde; cancellare richiede una conferma separata. Le frasi della vignetta si configurano in **Preferences > Recurring phrases**. L'icona dell'app e neutra e distinta dalla mascotte. Dettagli in [Note desktop e frasi ricorrenti](docs/notes.md).

La mascotte si sceglie dal menu o dalle preferenze; sono disponibili **Chicken**, **Frost gryphon** ed **Ember wyvern**. La gallina guarda in alto durante l'attesa, ogni tanto becca o svolazza e cova quando ci sono notifiche in attesa. Ogni notifica ha un uovo: verde con piu del 50% del tempo rimanente, giallo fino al 20%, rosso nella fase finale. Dopo la scadenza l'uovo rimane incrinato fino alla conferma. Animazioni disattivabili; supporto a riduzione del movimento. Dettagli e provenienza degli asset in `docs/mascot.md`.

Sul desktop rimane solo la gallina, inizialmente posizionata sopra la barra delle applicazioni. Trascinala con il tasto sinistro; il tasto destro apre il pannello con comandi, timer, configurazione, minimizzazione e uscita. Il pannello si chiude con Esc, con un clic sulla gallina o all'esterno, oppure passando a un'altra applicazione. Da tastiera: Tab per raggiungere la gallina, Invio/Spazio o Maiusc+F10 per aprire il pannello. Le scadenze e gli errori mostrano un piccolo avviso sulla mascotte anche con il pannello chiuso. Il posizionamento automatico dipende dalle funzionalita del desktop, in particolare su Wayland.

Il pannello usa una finestra separata: aprirlo e chiuderlo non sposta o ridimensiona la gallina. Per creare un avviso seleziona **Notification**, scrivi il testo e premi **Next**; scegli quindi una quantita da 0 a 100 e **Seconds**, **Minutes**, **Hours** o **Days**. **Start notification** richiede una quantita intera maggiore di zero. La durata massima e 100 giorni. Lo stesso percorso e disponibile nella configurazione web; i comandi testuali restano nella voce **Text command**. Chiudere il pannello con Alt+F4 lo nasconde; chiudere la finestra della gallina termina l'app.

- Mascotte trasparente, trascinabile, sempre in primo piano, con comando testuale, minimizzazione e uscita esplicita.
- Pagina HTTP con timer, storico, mantenimento in attività e preferenze.
- Timer multipli con nome, annullamento, estensione di 5 minuti e conferma dell'avviso.
- Comandi come `timer tè 5 minuti`, `avvisami tra 30 minuti`, `quanto manca?`, `aggiungi 5 minuti al timer tè`, `annulla il timer tè`.
- Salvataggio locale dei timer e delle preferenze. Recupero delle scadenze all'avvio e dopo sospensione.
- Notifiche di sistema nella versione desktop; avviso persistente nella mascotte e nella pagina.
- Windows: mantenimento del sistema oppure di sistema e display tramite API native.
- Linux: prima integrazione logind per il sistema; controllo del display non ancora implementato. Comportamento reale da verificare sulla distribuzione di destinazione.

L'esecuzione di script e la pianificazione di task secondari sono sviluppi successivi: questo prototipo non esegue comandi di shell ricevuti dall'interfaccia.

## Sviluppo su Windows

Servono Rust stable (Cargo, rustfmt, Clippy), Node.js 22.12+ o una LTS successiva compatibile con Vite, Microsoft C++ Build Tools x64 e Windows SDK. Per la finestra desktop serve WebView2.

Se Rust è già installato tramite rustup, questo script configura Cargo, compilatore e SDK nella **sessione PowerShell corrente**, senza modificare il PATH permanente:

```powershell
. .\scripts\windows-env.ps1
npm.cmd ci
npm.cmd run desktop:dev
```

Se manca il Windows SDK moderno, lo script può scaricare i pacchetti ufficiali Microsoft NuGet in `%LOCALAPPDATA%\MonitorAwake\build-tools`, senza installazione amministrativa:

```powershell
. .\scripts\windows-env.ps1 -BootstrapSdk
```

Lo script richiede comunque un compilatore MSVC x64 già installato. La prima build scarica e compila le dipendenze; le successive sono incrementali.

`desktop:dev` prepara prima gli asset della configurazione HTTP, poi avvia Vite e la finestra desktop. Vite non apre browser. Le modifiche della mascotte hanno hot reload; per aggiornare la pagina HTTP incorporata, ricompila gli asset e riavvia l'app.

## Generare l'app Windows

Lo script prepara l'ambiente, installa le dipendenze frontend se assenti e genera eseguibile e installer:

```powershell
.\scripts\build-windows.ps1
```

Puoi eseguirlo anche da un'altra cartella usando il percorso completo. Se manca il Windows SDK, aggiungi `-BootstrapSdk` per scaricare i pacchetti ufficiali Microsoft nella cache locale. Lo script si ferma in caso di errore e non avvia l'app o il browser.

Risultati nella cartella `target/release`: eseguibile `monitorawake.exe` e installer in `bundle/nsis/`. I pacchetti locali non sono firmati. L'installer può predisporre WebView2 se assente; l'opzione predefinita può richiedere rete durante l'installazione del runtime. Dopo l'installazione, timer e configurazione sono offline.

## Sviluppo su Linux

Installa Rust, Node.js e i prerequisiti Tauri della tua distribuzione. Per Ubuntu/Debian, la base è:

```sh
sudo apt install build-essential libwebkit2gtk-4.1-dev libssl-dev libxdo-dev libayatana-appindicator3-dev librsvg2-dev
npm ci
npm run desktop:dev
# Pacchetto per la distribuzione compatibile con l'ambiente di build:
npm run desktop:build -- --bundles deb
```

La mascotte richiede una sessione grafica. Su Wayland posizionamento, trascinamento e primo piano dipendono dal compositor. Il backend logind richiede il bus di sistema e l'autorizzazione all'inibizione di `sleep`; non mantiene il display acceso e non aggira le politiche del sistema. L'inibizione `sleep` può essere considerata anche per richieste esplicite di sospensione, secondo le politiche logind.

## Configurazione HTTP

Avvia l'app e premi l'ingranaggio della mascotte: il browser si apre **solo con questa azione**. Il pulsante `URL` permette di selezionare e copiare il link. Chiudere la pagina non ferma timer e mascotte; uscire dall'app arresta anche il server.

Il server sceglie una porta libera e ascolta esclusivamente su `127.0.0.1`. Il link contiene una chiave casuale per quella sessione nel frammento `#token=...`; la pagina la sposta in `sessionStorage` e la rimuove dalla barra degli indirizzi. Le API richiedono tale chiave, controllano Host e Origin e non abilitano CORS. Non condividere il link. Al riavvio la vecchia sessione non è valida: riapri dalla mascotte.

Per lavorare sul motore senza la finestra Tauri:

```sh
npm run server
```

Stampa il link privato nel terminale e non apre browser. In questa modalità gli avvisi vengono scritti sul terminale, non nelle notifiche desktop. `Ctrl+C` arresta il server e rilascia il mantenimento in attività. È consentita una sola istanza per cartella dati.

## Dati e comportamento

Il formato attuale usa `schema_version: 4` per l'intero archivio. Le migrazioni dalle versioni 1, 2 e 3 creano un backup esatto prima di salvare il nuovo formato. Dettagli in [Dati persistenti e migrazioni](docs/persistence.md).

- Windows: `%LOCALAPPDATA%\MonitorAwake\state.json`.
- Linux: directory dati locale dell'utente (`$XDG_DATA_HOME/MonitorAwake`, normalmente `~/.local/share/MonitorAwake`).
- `MONITORAWAKE_DATA_DIR` permette di isolare un profilo di prova.
- I salvataggi usano un file temporaneo e sostituzione atomica; file corrotti o versioni non supportate non vengono sovrascritti.
- I timer seguono una scadenza UTC: una correzione dell'orologio cambia il tempo rimanente. Il prototipo non risveglia un computer sospeso.
- Il mantenimento manuale vale per la sessione e riparte disattivato. Ogni timer con `PC sveglio` conserva una richiesta indipendente fino alla scadenza o all'annullamento.
- Una scadenza viene salvata prima della notifica: un crash tra i due passaggi può impedire la notifica esterna, ma l'avviso rimane visibile al riavvio. Non viene promessa consegna esattamente una volta.
- Su Windows le notifiche Tauri richiedono l'app installata per il comportamento previsto; in sviluppo la mascotte e la pagina restano il riferimento per gli avvisi.

## Verifiche

```sh
npm run check
npm run test:widget
npm run build
cargo fmt --all -- --check
cargo test --workspace --no-default-features
cargo clippy --workspace --all-targets -- -D warnings
```

I test Rust coprono scadenze e riavvio, richieste di risveglio sovrapposte, comandi ambigui, durate errate, fallimenti di salvataggio e controllo degli accessi HTTP. Il frontend va compilato prima dei test Rust perché i suoi asset sono incorporati nell'eseguibile.

La build non sostituisce la verifica visiva. Rispettare `AGENTS.md`: nessun browser avviato durante lo sviluppo o i test senza una richiesta esplicita dell'utente.

Test HTTP completo, con profilo temporaneo separato e richiesta di risveglio limitata alla prova:

```sh
cargo build --no-default-features --bin monitorawake-server
python scripts/smoke-server.py target/debug/monitorawake-server.exe
# Su Linux: target/debug/monitorawake-server
```

Il test verifica accesso, origine, host, asset incorporati, scadenza, conferma, esclusione della seconda istanza e riavvio con rotazione della chiave. Arresta il processo di prova anche se un controllo fallisce e conserva il profilo temporaneo per diagnosi.

Stato della verifica iniziale (6 novembre 2026): 10 test Rust passati, frontend senza errori o warning, Clippy passato e installer Windows x64 generato. Test HTTP reale e richieste native Windows di attivazione/disattivazione passati. Backend Linux controllato con `cargo check --no-default-features --target x86_64-unknown-linux-gnu`; nessuna prova del desktop Linux. La finestra Windows espone correttamente i controlli all'accessibilità, ma il tool di verifica non è riuscito ad attivarla (`failed to activate captured window`): trascinamento, resa visiva e notifiche installate restano da verificare manualmente. Nessun browser è stato aperto durante queste verifiche.

## Mappa del codice

| File                        | Responsabilità                                                                    |
| --------------------------- | --------------------------------------------------------------------------------- |
| `src-tauri/src/core.rs`     | Tipi, transizioni dei timer e interprete dei comandi; nessuna dipendenza da Tauri |
| `src-tauri/src/service.rs`  | Stato condiviso e persistenza transazionale                                       |
| `src-tauri/src/http.rs`     | Server Axum, API autenticate e asset incorporati                                  |
| `src-tauri/src/platform.rs` | Adapter Windows/Linux e durata delle richieste native                             |
| `src-tauri/src/lib.rs`      | Ciclo di vita, scheduler e collegamento Tauri                                     |
| `src/App.svelte`            | Mascotte e pannello di configurazione                                             |
| `src/lib/api.ts`            | Trasporto Tauri o HTTP verso lo stesso motore                                     |

Le skill mantenute nel repository sono in `skills/`. Dopo una modifica, sincronizza le copie personali in `~/.codex/skills/`.
