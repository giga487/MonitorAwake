# Dati persistenti e migrazioni

Requisito concordato: tutti i dati persistenti devono appartenere a uno schema versionato. La versione del formato dati e distinta dalla versione dell'applicazione.

## Stato attuale

`state.json` contiene una versione globale `schema_version: 1`, i timer e le preferenze, inclusa la mascotte. La versione globale copre l'intero documento, anche i campi delle preferenze: non serve una versione per ogni timer o impostazione.

`Model::validate` rifiuta versioni diverse da 1. `Service::load` carica e valida il documento; i salvataggi sostituiscono il file tramite un file temporaneo. Un documento corrotto o di versione non supportata non viene sovrascritto. Al momento non esiste una catena di migrazioni tra versioni.

Il campo opzionale `settings.mascot` usa un default Serde per leggere i file precedenti come `chicken`. Questo e un caso di aggiunta compatibile; cambiamenti incompatibili richiedono una migrazione esplicita.

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

Le note desktop sono ancora una proposta grafica. La loro implementazione deve includere il formato persistente e le relative migrazioni, non rinviarli a un aggiornamento successivo.

Preferire inizialmente lo stesso archivio dei timer per mantenere coerenti le note e gli eventuali promemoria collegati. Per l'introduzione delle note prevedere una nuova versione dello schema e una migrazione dalla v1 che conservi timer, identificatori, scadenze e preferenze, inizializzando una raccolta note vuota.

Il formato deve descrivere esplicitamente testo, identificatore stabile, colore, visibilita, stato fissato in primo piano e geometria delle note. La chiusura di una nota deve aggiornare la visibilita senza eliminarne il contenuto. Definire unita delle coordinate e fallback quando monitor o scala cambiano. Conservare un eventuale riferimento al timer tramite identificatore, senza una seconda copia indipendente della scadenza.

L'identita dell'app rimane neutra: la gallina e un valore della preferenza mascotte, non il nome o l'identificatore del formato delle note.

## Verifiche necessarie all'implementazione

Conservare fixture dei formati distribuiti e verificare migrazione, riapertura del file migrato, conservazione di testo e identificatori, impostazioni opzionali, riferimenti tra note e timer, file corrotti, versioni future, backup e fallimenti di scrittura. Ripetere il caricamento dopo una migrazione riuscita non deve duplicare note o promemoria. Verificare che un errore lasci intatto il file originale.

Queste sono specifiche per i prossimi incrementi: backup e catena di migrazioni non sono ancora implementati.
