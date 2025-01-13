## Progetto 2.1: Back-up di emergenza  

Questo programma in Rust consente di eseguire backup in situazioni in cui lo schermo non è agibile. L’applicazione è progettata per funzionare in background con un basso consumo di CPU e registra log periodici per il monitoraggio delle prestazioni.

All'avvio del programma, l'utente può configurare i dettagli relativi al backup. La configurazione richiede di specificare:\\n

* **La cartella sorgente** da cui copiare i file.  
* **La cartella di destinazione** dove salvare i file.  
* Facoltativamente, **il tipo di file da includere** nel backup 

Una volta completata la configurazione, il backup può essere avviato disegnando una forma specifica con il mouse. La forma parte dall'angolo in alto a sinistra dello schermo, passa attraverso tutti gli angoli seguendo i bordi dello schermo e termina nell'angolo in basso a sinistra.

Se il comando di avvio viene riconosciuto correttamente, viene emesso un segnale acustico e appare una finestra che guida l'utente sui prossimi passi. A questo punto, l'utente ha tre opzioni:

1. **Confermare il backup**: spostando il mouse dall'angolo in basso a sinistra all'angolo in basso a destra (slide verso destra). Anche in questo caso viene emesso un suono per confermare l'azione.  
2. **Annullare il backup**: spostando il mouse dall'angolo in basso a sinistra all'angolo in alto a sinistra (slide verso l'alto). Un suono conferma l'annullamento.  
3. **Modificare la configurazione del backup**: disegnando una diagonale dall'angolo in basso a sinistra all'angolo in alto a destra riapparirà la finestra di configurazione. 

## Implementazione

**Configurazione del backup**  
Il programma legge gli argomenti passati nella riga di comando. Se l'argomento passato al programma è `"config"`, viene invocata la funzione **`display_window::show_gui_if_needed()`**, che gestisce la creazione della GUI per la configurazione **`ConfigWindow`**.

**Monitoraggio dei movimenti del mouse**  
La risoluzione del monitor viene ottenuta chiamando la funzione **`get_screen_resolution()`**`,`in questo modo il programma è in grado di adattarsi alla dimensione e gestire correttamente il tracciamento del mouse attraverso **`track_mouse(width as f64, height as f64)`**`.`

**Gestione del backup (avvio, conferma, annullamento)**

#### Il tracciamento del mouse viene avviato attraverso **`track_mouse(screen_width: f64, screen_height: f64),`** che usa `rdev` per rilevare i movimenti. La funzione **`contains_corners(points: &Vec<Point>, screen_width: f64, screen_height: f64, enable: bool) -> Action`** verifica se il mouse ha toccato gli angoli dello schermo seguendo una sequenza prestabilita e in base ad essa esegue azioni come avviare, annullare e confermare un backup.

Per confermare il riconoscimento del comando viene riprodotto un suono. La riproduzione è gestita tramite la libreria **`rodio`** da **`play_sound(number: i32)`** che riceve in input quale audio riprodurre`.` I file audio sono gestiti nella directory **`Resources/audio/`**, e l'accesso ai file è relativo al percorso dell'eseguibile. La funzione **`sink.sleep_until_end()`** blocca l'esecuzione del programma fino al termine della riproduzione del suono. 

Nel caso di riconoscimento del comando di avvio del backup viene visualizzata una finestra di conferma che illustra le opzioni successive. Se l'argomento passato al programma nella riga di comando è `"backup"`, il programma invoca la funzione **`display_window::show_backup_gui()`**, responsabile della creazione e visualizzazione della GUI per la gestione del backup **`BackupWindow`**.

**Esecuzione del backup**  
La struttura **`Config`** rappresenta le impostazioni di configurazione per il backup. Essa include:

* **`source_path`**: il percorso della cartella di origine da cui copiare i file.  
* **`destination_path`**: il percorso della cartella di destinazione dove salvare i file di backup.  
* **`backup_type`**: il tipo di backup da eseguire (può essere "full-disk", "directory", o "selective").  
* **`extensions_to_backup`**: una lista di estensioni di file che devono essere incluse nel backup (per il tipo "selective").

La funzione **`backup_files`** esegue l'operazione di backup dei file in base alla configurazione:

* Verifica se il percorso di origine esiste. Se non esiste, restituisce un errore.  
* Se il percorso di destinazione non esiste, crea la directory.  
* In base al tipo di backup:  
  * "full-disk" o "directory": copia l'intera cartella di origine nella destinazione  
  * "selective": copia solo i file con estensioni specifiche, come definito nella configurazione.  
* Dopo il completamento del backup, la funzione **`backup_monitor`** registra i dettagli dell'operazione (dimensione totale dei file copiati e tempo impiegato) in un file di log

**Logging dell'utilizzo della CPU**  
L'operazione di monitoraggio della CPU è gestita in un thread separato per evitare che il programma principale venga bloccato. **`start_cpu_monitor(backup_pid, 120)`** avvia il monitoraggio per un dato processo identificato dal suo PID (Process ID): utilizzando la libreria **`sysinfo`** il programma raccoglie informazioni sul consumo della CPU ogni secondo. Poi, ogni 2 minuti viene calcolato il consumo medio della CPU su tutti i core disponibili e viene registrato nel file di log.

