# Back-up di emergenza

## Prerequisiti
### MacOs
Dopo aver eseguito lo script di setup, fornire a backup_program i permessi di Input Monitoring:

1. Aprire `Preferenze di Sistema`.
2. Selezionare `Sicurezza e Privacy`.
3. Nella scheda `Privacy`, selezionare `Input Monitoring`.
4. Aggiungere `backup_program` alla lista delle applicazioni autorizzate.

### Linux
É necessario che `zenity` sia installato per il corretto funzionamento del programma.  
Sono necessarie le seguenti librerie per la corretta esecuzione del programma:
- ALSA development files
  - Ubuntu/Debian
    - `libasound2-dev`
  - Fedora
    - `alsa-lib-devel`
- `libx11-dev`
- `libxi-dev`
- `libxtst-dev`

## Avviare il Programma
### Windows
1. Aprire `cmd` con permessi di amministratore.
2. Lanciare lo script di setup con il comando `./setup_windows.bat`.

### MacOs/Linux
1. Aprire `Terminale`.
2. Lanciare lo script di setup con il comando `./setup_macos_linux.sh`.

## Configurazione del Backup
All'avvio del programma, sarà necessario configurare i seguenti parametri:

1. **Percorso Sorgente (Source Path):** Specificare la directory o il disco da cui si desidera copiare i file.
2. **Percorso di Destinazione (Destination Path):** Specificare la directory o il disco su cui verranno copiati i file.
3. **Tipo di Backup:**
    - **Directory:** Copia l'intera cartella specificata nel percorso sorgente.
    - **Selective:** Consente di selezionare specifici formati di file da copiare (es. `.jpg`, `.txt`, ecc.).
    - **Full-Disk:** Copia tutti i dati presenti suL disco specificato nel percorso sorgente.

## Avvio del Backup
Per avviare il backup, eseguire la seguente sequenza:

    Disegnare con il mouse una forma sullo schermo:
    - Partire dall'**angolo in alto a sinistra** dello schermo.
    - Spostarsi verso l'**angolo in alto a destra**.
    - Scendere fino all'**angolo in basso a destra**.
    - Infine, completare la forma muovendosi verso l'**angolo in basso a sinistra**.
    - È importante seguire i bordi dello schermo per garantire che il comando venga riconosciuto.

   Una volta che il comando è stato riconosciuto, un segnale acustico confermerà l'azione e apparirà una finestra di conferma sullo schermo.

## Conferma, Annullamento o Riconfigurazione del Backup
Dopo l'apparizione della finestra di conferma, è possibile:

1. **Confermare il Backup:**
    - Eseguire uno slide **da sinistra a destra**, partendo dall'**angolo in basso a sinistra** verso l'**angolo in basso a destra**.
    - Un segnale acustico confermerà la scelta.

2. **Annullare il Backup:**
    - Eseguire uno slide **verso l'alto**, partendo dall'**angolo in basso a sinistra** verso l'**angolo in alto a sinistra**.
    - Anche in questo caso, un segnale acustico confermerà l'annullamento.

3. **Riconfigurare il Backup:**
    - Disegnare una diagonale **dall'angolo in basso a sinistra** verso l'**angolo in alto a destra**.
    - Non ci sarà un segnale acustico; al contrario, verrà mostrata nuovamente la finestra di configurazione iniziale.

## Cleanup
Per rimuovere il programma dal sistema, eseguire l'utility di cleanup `uninstall_service`.