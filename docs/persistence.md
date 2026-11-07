# Dati persistenti e migrazioni

Requisito concordato: tutti i dati persistenti devono appartenere a uno schema versionato. La versione del formato dati e distinta dalla versione dell'applicazione.

## Stato attuale

`state.json` contiene una versione globale `schema_version: 4`, dichiarata da `core::SCHEMA_VERSION`. Copre timer, note e preferenze, incluse mascotte e frasi ricorrenti: non serve una versione per ogni entita. La configurazione mostra versione dell'app e versione dello schema nei dettagli locali.

`persistence::load` legge la versione prima di deserializzare il modello corrente. Supporta il formato corrente, la catena v1 -> v2 -> v3 -> v4 e i passaggi dai formati 2 e 3: conserva timer e preferenze, inizializza le note vuote e assegna i valori predefiniti alle frasi. Valida il risultato, scrive una copia esatta dell'originale in `state.json.backup-v<versione-originale>-<uuid>.json` senza sovrascrivere backup precedenti, poi sostituisce atomicamente il file principale tramite un file temporaneo sincronizzato. Una riapertura del file v4 non ripete la migrazione. File corrotti e versioni future non vengono sovrascritti; non viene tentato un downgrade.

Il campo opzionale `settings.mascot` usa un default Serde per leggere i file precedenti come `chicken`. Anche le preferenze delle frasi hanno default espliciti. La revisione del testo di ogni nota (`content_revision`) impedisce a un editor obsoleto di sovrascrivere modifiche nuove; non e una versione dello schema. Colore, geometria e visibilita non incrementano la revisione del testo.

## Contratto per le prossime modifiche

- Dichiarare la versione corrente in un unico punto del backend Rust e conservare la descrizione di ciascuna versione distribuita.
- Versionare ogni archivio persistente. Se note, preferenze e timer rimangono nello stesso documento, una versione globale li copre tutti. Se vengono separati in altri file, ogni file deve dichiarare il proprio formato e la propria versione.
- Documentare anche la compatibilita tra archivi e i riferimenti tra entita. Evitare aggiornamenti parziali di file diversi che lascino una nota collegata a un promemoria inesistente.
- Rilevare la versione prima di deserializzare nel modello corrente. Distinguere formato vecchio migrabile, formato futuro non supportato e dati corrotti.
- Migrare per passi espliciti, per esempio `v1 -> v2 -> v3`, attraverso tipi o trasformazioni dedicate, senza distribuire conversioni nel frontend.
- Prima della migrazione conservare una copia esatta del file originale in un backup distinto, senza sovrascrivere backup precedenti. Se il backup fallisce, non modificare l'archivio.
- Eseguire la conversione in memoria, validare dati e riferimenti, quindi salvare il risultato con sostituzione atomica. Aggiornare lo stato utilizzato dal servizio solo dopo il salvataggio riuscito.
- In caso di errore conservare originale e backup e mostrare un messaggio utile. Non sostituire mai un documento illeggibile con un profilo vuoto.
- Con una versione futura, rifiutare l'apertura in scrittura e indicare che serve una versione piu recente dell'app. Non tentare un downgrade automatico.
- Valutare la compatibilita sia in lettura sia in riscrittura: un client vecchio che ignora campi nuovi potrebbe cancellarli al salvataggio. Aggiungere un default non e sufficiente per dichiarare compatibile ogni cambiamento.
- Esporre nei dettagli diagnostici la versione dell'app e quella dello schema. Non richiedere all'utente di scegliere manualmente una migrazione ordinaria.

## Incremento previsto per le note

Le note desktop e le frasi ricorrenti sono implementate nello schema 2; il formato persistente e la migrazione fanno parte dello stesso incremento.

Le note usano lo stesso archivio dei timer. La creazione o cancellazione di un promemoria collegato e una sola modifica del modello, validata e salvata prima di diventare visibile all'interfaccia. I riferimenti mancanti o condivisi tra piu note sono rifiutati. Svuotare lo storico rimuove i riferimenti alle scadenze eliminate.

Ogni nota salva identificatore UUID, titolo, testo, colore, visibilita, primo piano, geometria, revisione del contenuto e riferimento opzionale al timer. Le coordinate x/y sono pixel fisici globali; larghezza e altezza sono pixel logici. Il ripristino limita la posizione all'area di lavoro di un monitor disponibile; su Wayland puo decidere il compositor. Nascondere la nota conserva testo e promemoria. L'archivio accetta fino a 100 note (80 caratteri per il titolo, 10.000 per il testo), 50 frasi di 180 caratteri e un file totale di massimo 8 MiB.

L'identita dell'app rimane neutra: la gallina e un valore della preferenza mascotte, non il nome o l'identificatore del formato delle note.

## Verifiche necessarie all'implementazione

Conservare fixture dei formati distribuiti e verificare migrazione, riapertura del file migrato, conservazione di testo e identificatori, impostazioni opzionali, riferimenti tra note e timer, file corrotti, versioni future, backup e fallimenti di scrittura. Ripetere il caricamento dopo una migrazione riuscita non deve duplicare note o promemoria. Verificare che un errore lasci intatto il file originale.

Backup, migrazione v1 -> v2, protezione dei formati futuri e conflitti di modifica sono implementati e coperti dai test Rust e HTTP. Le migrazioni successive richiederanno nuovi passi espliciti e fixture del formato precedente.

## Schema 3: catalogo delle mascotte

Lo schema 3 ammette `settings.mascot = frost_gryphon` oltre a `chicken`. Il passo esplicito v2 -> v3 conserva note, timer, frasi personali e preferenze; non cambia la mascotte selezionata. Il passaggio v1 -> v2 resta separato e viene seguito dal nuovo passo. Le versioni precedenti non riconoscono il nuovo valore e non devono riscrivere il file. Il backup esatto permette il recupero del formato precedente.

## Schema 4: Ember wyvern

Aggiunge il valore `ember_wyvern` al catalogo. Il passo v3 -> v4 conserva selezione, note, timer e frasi. Anche questo ampliamento del dominio enum richiede una versione esplicita per impedire riscritture da applicazioni precedenti.
