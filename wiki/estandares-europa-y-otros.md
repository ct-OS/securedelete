# Estándares de Europa y otros países

Estándares gubernamentales y esquemas académicos no estadounidenses, con la variante que
implementa SecureDelete en cada caso.

---

## Reino Unido — HMG IS5 (CESG / GCHQ)

- **Quién/cuándo**: *HMG Infosec Standard 5, Secure Sanitisation* del CESG (parte de
  GCHQ). La edición 4.0 es de **abril de 2011**; hay versiones 5.x y 6.x posteriores. Hoy
  el documento vive bajo el **NCSC** (sucesor del CESG).
- **Qué dice**: dos niveles:
  - **Baseline**: 1 pasada de **ceros**.
  - **Enhanced**: 3 pasadas — **unos → ceros → aleatorio** (con verificación); algunas
    herramientas y descripciones lo expanden a 5 pasadas (00, FF, R, 00, FF).
- **SecureDelete**: métodos `hmg` (00, FF, aleatorio; variante de 3) y `hmg5`
  (00, FF, R, 00, FF; variante de 5). El método **`hmg_baseline`** implementa el
  Baseline oficial (una pasada de ceros).

---

## Canadá — RCMP TSSIT y B2-002

- **Quién/cuándo**: *Royal Canadian Mounted Police — Technical Security Standard for
  Information Technology* (**TSSIT**), publicado en **1995** (BT32-36/6-1995E). La lista
  de productos aprobados **B2-002** es de **mayo de 2009**.
- **Qué dice**: el anexo OPS-II describe la purga por sobrescritura en **7 pasadas**:
  **00, FF, 00, FF, 00, FF, aleatorio** con verificación. El B2-002 no define pasadas:
  es una lista de productos certificados.
- **SecureDelete**: métodos `rcmp` (FF, 00, FF, 00; la simplificación de 4 pasadas que
  popularizaron las herramientas) y `rcmp7`/`b2002` (las 7 pasadas 00/FF + aleatorio del
  OPS-II).

---

## Alemania — VSITR (BSI)

- **Quién/cuándo**: *Verschlusssachen-Informationsverfahren und Technische
  Realisierung* (VSITR) del Ministerio del Interior federal, en vigor desde el
  **1 de septiembre de 1998**; el BSI lo aclaró en el **Hinweisblatt Nr. 11 del
  21 de noviembre de 1999**. **Derogado el 1 de junio de 2006** (sustituido por la
  guía de sanitización del BSI de 2009).
- **Qué dice**: 6 pasadas de **00/FF alternados** y una **7.ª pasada de 0x55 o 0xAA**
  (patrón alternado) con verificación.
- **SecureDelete**: método `vsitr` (FF, 00, FF, R, R, R, R) — la variante que difundieron
  las herramientas, distinta del patrón oficial. El método **`vsitr_official`** implementa
  la secuencia oficial del BSI (00/FF alternados ×6 y una 7.ª pasada de 0x55).

---

## Alemania — "BSI 9 pasadas"

- **Quién/cuándo**: el esquema de **9 pasadas** (FF, FE, FD, FB, F7, EF, DF, BF, 7F)
  aparece en el manual de `scrub` (2003) atribuido al BSI, pero **no corresponde a ningún
  estándar primario publicado**.
- **Contexto real**: la guía BSI IT-Grundschutz (M 2.167) recomendaba 2–3 pasadas, y el
  módulo actual CON.6 exige una única pasada con PRNG para medios magnéticos.
- **SecureDelete**: método `bsi`/`bsi9` (9 patrones) por tradición de herramientas.

---

## Alemania — Bundeswehr BMB21-2019

- **Quién/cuándo**: especificación interna del Ejército alemán, **2019**.
- **Qué dice**: 6 pasadas: **FF, 00, aleatorio, aleatorio, aleatorio, FF** (divulgada vía
  documentación de `nwipe`; el texto primario no es público).
- **SecureDelete**: método `bmb21`/`bundeswehr` (6 pasadas).

---

## Rusia — GOST R 50739-95

- **Quién/cuándo**: estándar estatal ruso **GOST R 50739-95** (1995) del organismo federal
  de metrología, para la protección contra la recuperación de información.
- **Qué dice**: define clases de seguridad. Para los niveles superiores prescribe
  sobrescritura con **patrón aleatorio**; para los inferiores, **ceros** (y bloqueo físico
  del medio). No fija un número de pasadas.
- **SecureDelete**: método `gost`/`gost50739` (00, 00) — la variante de herramientas.

---

## Alemania — Pfitzner 33

- **Quién/cuándo**: **Dipl.-Ing. Roy Pfitzner**, comisionado de protección de datos de
  Brandeburgo, en su artículo *Sicheres Löschen von Dateien und Datenträgern* (**2003**).
- **Qué dice**: **33 pasadas aleatorias** sobre todo el medio (número elegido por
  seguridad de margen; partía del supuesto de que los medios se podrían reutilizar).
- **SecureDelete**: método `pfitzner`/`pfitzner33` (33 aleatorias).

---

## Esquemas académicos: Gutmann y Schneier

### Gutmann (35 pasadas)
- **Quién/cuándo**: **Peter Gutmann**, *Secure Deletion of Data from Magnetic and
  Solid-State Memory*, 6.º Simposio de Seguridad de la **USENIX**, julio de **1996**.
- **Qué dice**: 35 pasadas (4 aleatorias + 31 patrones), diseñadas para la tecnología de
  codificación *MFM/RLL* de los discos de la época. **El propio artículo afirma que para
  discos modernos bastan 4 pasadas aleatorias** y que el esquema completo solo aplica a
  tecnología antigua de baja densidad.
- **SecureDelete**: método `gutmann` (35 pasadas).

### Schneier (7 pasadas)
- **Quién/cuándo**: **Bruce Schneier**, *Applied Cryptography* (**1994**).
- **Qué dice**: 7 pasadas: **FF, 00, y cinco pasadas aleatorias**. Schneier ha matizado
  después que una pasada aleatoria es suficiente en medios modernos.
- **SecureDelete**: método `schneier` (FF, 00, R×5, FF).
