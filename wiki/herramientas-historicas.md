# Historia de las herramientas

Las herramientas de borrado seguro nacieron a mediados de los 90, empujadas por el
artículo de Gutmann (1996) y por los estándares militares. Esta página recoge las
principales, incluida la genealogía de los métodos que implementa SecureDelete.

---

## GNU shred

- **Autor**: Colin Plumb.
- **Historia**: escrito a partir de la idea del tool `wipe` (ver más abajo), se integró en
  GNU fileutils en **enero de 1999** (copyright 1997–1999). Hoy vive en coreutils.
- **Pasadas**: originalmente **25 pasadas** por defecto. En **febrero de 2009**
  (**coreutils 7.1**), Pádraig Brady lo redujo a **3 pasadas** y añadió la advertencia de
  que una sola pasada es suficiente en discos modernos.
- **Nota**: `shred` no es un método concreto; aplica el esquema de Gutmann si se le pide
  (`--method gutmann`) y permite definir patrones propios.

---

## wipe — Berke Durak

- **Autor**: **Berke Durak** (Bilim OnDört GmbH).
- **Historia**: comenzó en **febrero de 1998** (v0.02) y se mantuvo hasta la
  **v0.24 de noviembre de 2016** (serie 0.x; no existe la numeración 0.16–0.26 que
  citan algunas fuentes).
- **Pasadas**: por defecto aplica **35 pasadas** (34 patrones de Gutmann + 1 final de
  relleno aleatorio, equivalentes al esquema de 35 de Gutmann). Con `-q` usa el modo
  rápido (**4 pasadas aleatorias**); `-X <n>` fija el número de pasadas; `-l` indica la
  **longitud** a borrar.
- **Dato curioso**: el "Paranoia (38 pasadas)" que citan muchas páginas **no existe**: es
  una confusión entre este `wipe` y el `ya-wipe` (abajo).

---

## ya-wipe (Thomas M. Vier Jr.)

- **Autor**: **Thomas M. Vier Jr.**
- **Historia**: la versión **2.3.1 data del 1 de noviembre de 2009**. Desde la 2.3, el
  comportamiento por defecto es **una pasada aleatoria**.
- **Pasadas**: con `-X` usa **27 patrones estáticos** definidos por el autor.
- **SecureDelete**: el método `wipe27`/`wipestatic` implementa exactamente esos **27
  patrones**.

---

## scrub

- **Autor**: **Jim Garlick** (Lawrence Livermore National Laboratory), inspirado en el
  artículo de Gutmann.
- **Historia**: publicado alrededor de **2003** (identificador UCRL-CODE-2003-006 del
  LLNL), como utilidad estándar de la distribución.
- **Pasadas**: implementa el esquema de Gutmann (35) y la versión "nnsa" del
  **NNSA/LLNL**, con 4–6 pasadas aleatorias. Su manual es la fuente que atribuye el
  esquema de **9 pasadas** al BSI alemán (ver [estandares-europa-y-otros.md](estandares-europa-y-otros.md)).

---

## DBAN (Darik's Boot And Nuke)

- **Autor**: **Darik Horn**.
- **Historia**: lanzado como software libre; las betas comenzaron en **enero de 2003** y
  la **1.0.0 salió el 27 de mayo de 2003**. A partir de la versión 2.x el código dejó de
  publicarse y se distribuyó como freeware (el proyecto original quedó cerrado en 2015).
- **Pasadas**: ofrecía 3, 7 y 35 pasadas (DoD, Gutmann, etc.) desde un CD/USB de arranque.

---

## nwipe

- **Autor**: **Martijn van Brummelen**.
- **Historia**: fork de las rutinas de borrado de DBAN, alojado desde **2013**; el primer
  lanzamiento oficial fue el **20 de octubre de 2014**. Es el motor de borrado de
  PartedMagic.
- **Pasadas**: incluye los métodos de estándares (zero, dod, usaf, vsitr, hmg, gost,
  schneier, bmb21…) que SecureDelete también implementa, además de pasadas personalizadas.

---

## Eraser

- **Autor**: originalmente **Sami Tolvanen** (hacia **1997**); desde ~2003 lo mantiene
  **Heidi Computers** (Garrett Trant). El proyecto Eraser 6.x reescribió la herramienta en
  C#.
- **Pasadas**: el más popular fue el **Gutmann (35 pasadas)**, además de métodos del BSI,
  VSITR, US Army y personalizados.

---

## BleachBit

- **Autor**: **Andrew Ziem**.
- **Historia**: primera versión el **24 de diciembre de 2008**.
- **Pasadas**: limpiador del sistema que ofrece sobrescritura del espacio libre (1 a 3
  pasadas de patrones o aleatorio), no un borrador de archivos individuales.

---

## secure-delete / srm (van Hauser)

- **Autor**: **van Hauser (THC)**.
- **Historia**: paquete `secure-delete` de THC con las utilidades `srm`, `sfill`, `sdmem`
  y `sswap`; activo de **2001** en adelante.
- **Pasadas**: `srm` usa el esquema de Gutmann (**38 pasadas** en su numeración, variante
  del 35 original con relleno adicional). Es la fuente de la leyenda del "borrado de 38
  pasadas" que las páginas atribuyen erróneamente a otros estándares.
