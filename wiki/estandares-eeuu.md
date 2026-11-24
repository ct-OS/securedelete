# Estándares de Estados Unidos

Estados Unidos es el origen de casi todos los "estándares de N pasadas" que ofrecen las
herramientas de borrado. Esta página separa, para cada uno, **lo que dice el documento
real** de **la variante que difundieron las herramientas**, e indica qué método de
SecureDelete corresponde a cada cosa.

> **Idea clave.** Los manuales describen *procedimientos* (sobrescribir con un carácter,
> su complemento y un valor aleatorio, y verificar), casi nunca bytes concretos. Los
> `00`, `FF` que aparecen en las tablas de internet son la forma en que los programadores
> de los años 90 concretaron esos procedimientos, y de ahí nacieron las variantes.

## Resumen

| Documento | Organismo | Año | Procedimiento del documento | Estado actual | Métodos de SecureDelete |
|---|---|---|---|---|---|
| NCSC-TG-025 | NCSC (NSA) | 1991 | carácter, complemento, aleatorio (purga) | Retirado | — (origen de los demás) |
| NAVSO P-5239-26 | Armada | 1993 | patrón según codificación RLL / MFM, con verificación | Retirado | `navy`, `navy_mfm` |
| DoD 5220.22-M (NISPOM), §8-306 | DoD | 1995 | opción E: carácter, complemento, aleatorio + verificación | Sobrescritura **eliminada en 2006** | `dod`, `dod7` |
| AFSSI-5020 | Fuerza Aérea | 1996 | 00, FF, aleatorio + verificación | Retirado | `usaf`, `afssi5020_official` |
| AR 380-19 | Ejército | 1998 | aleatorio, carácter, complemento + verificación | Sustituido (AR 25-2) | `usarmy`, `arm38019` |
| NSA/CSS Manual 130-2 | NSA | 2000 | aleatorio, aleatorio, patrón conocido + verificación | Sustituido (NSA/CSS 9-12) | `nsa1302` (y `nsa`, ver abajo) |
| DOE M 205.1-2 | Energía | 2005 | carácter, complemento, aleatorio | Sustituido | `doe` |
| NIST SP 800-88 | NIST | 2006 / 2014 / 2025 | **1 pasada** (Clear); comandos nativos o cripto (Purge) | **Vigente** | `random`, `--secure-erase` |

"Retirado" o "sustituido" significa que el documento ya no rige y que sus sucesores
(cuando existen) remiten a NIST SP 800-88 o exigen desmagnetizar o destruir el soporte.

---

## NCSC-TG-025 — el origen: *A Guide to Understanding Data Remanence*

- **Quién/cuándo**: *National Computer Security Center* (dependiente de la NSA), versión 2
  de **septiembre de 1991**. Es el "libro verde bosque" (*Forest Green Book*) de la
  *Rainbow Series*, que desarrolla lo que ya apuntaba CSC-STD-005-85 (1985).
- **Qué dice**: distingue *clearing* (borrar frente a ataques por software) de *purging*
  (frente a ataques de laboratorio). Para purgar por sobrescritura describe escribir **un
  carácter, después su complemento y después un carácter aleatorio**, varias veces si
  hace falta. Advierte que la sobrescritura no sirve para soportes dañados y que la
  desmagnetización o la destrucción son preferibles para información clasificada.
- **Importancia**: es la fuente de la que beben casi todos los manuales posteriores
  (DoD, Fuerza Aérea, DOE); la secuencia "carácter, complemento, aleatorio" se repite en
  todos ellos.
- **SecureDelete**: no tiene un método propio; el procedimiento es el de `dod`.

---

## DoD 5220.22-M (NISPOM) — "DoD 3 pasadas"

- **Quién/cuándo**: Departamento de Defensa. Manual del *National Industrial Security
  Program Operating Manual* (NISPOM), que regula cómo las empresas contratistas protegen la
  información clasificada. Edición de **enero de 1995**; reeditado el **28 de febrero de
  2006**, con cambios en 2013 y 2016; desde **febrero de 2021** es el reglamento
  **32 CFR Parte 117**.
- **Qué decía (1995)**: el capítulo 8, párrafo **8-306**, incluía una *matriz de borrado y
  saneamiento* (*Clearing and Sanitization Matrix*) que asignaba a cada tipo de soporte
  una serie de opciones identificadas por letras. Las herramientas citan dos:
  - **C**: sobrescribir todas las posiciones direccionables con un único carácter.
  - **E**: sobrescribir todas las posiciones direccionables con **un carácter, su
    complemento y después un carácter aleatorio, y verificar**.

  El "DoD 3 pasadas" es la opción **E**. El documento **no fija qué carácter**: el
  `00, FF, aleatorio` es la elección de los programadores, no del manual.
- **2006**: la reedición del NISPOM **eliminó la matriz y toda especificación de
  sobrescritura**, y dejó la decisión a la agencia de seguridad competente. La guía
  posterior del *Defense Security Service* (DSS, 2007) ya no aceptaba la sobrescritura para
  sanear discos con información clasificada: solo desmagnetizar o destruir.
- **Hoy**: decir que un borrado "cumple DoD 5220.22-M" no tiene respaldo en el manual
  vigente; es una referencia histórica.
- **SecureDelete**: método `dod` (`00, FF, aleatorio`), la concreción habitual de la
  opción E. Uso: `secure_delete -m dod --verify fichero`.

### DoD 5220.22-M ECE — "DoD 7 pasadas"

- **Qué es**: una combinación de opciones de la **misma matriz de 1995**: **E**, después
  **C** y otra vez **E** (de ahí *E-C-E*). Eraser, por ejemplo, la etiqueta como
  "US DoD 5220.22-M (8-306./E, C and E)".
- **Estado**: el manual permitía combinar opciones, pero **nunca definió una secuencia de 7
  pasadas como tal**; el "ECE" es un nombre de las herramientas. Desapareció junto con la
  matriz en 2006.
- **SecureDelete**: método `dod7` (`00, FF, R, 00, FF, R, 00`), la secuencia que usa la
  mayoría de herramientas. No coincide letra por letra con E-C-E (que sería
  `00, FF, R, 00, 00, FF, R`): es otra concreción de proveedores. Uso:
  `secure_delete -m dod7 fichero`.

---

## US Army — AR 380-19

- **Quién/cuándo**: Departamento del Ejército. Reglamento 380-19 (*Information Systems
  Security*), edición de referencia del **27 de febrero de 1998**. Fue sustituido por el
  **AR 25-2** (2003), que remite a la guía general del DoD.
- **Qué dice**: el apéndice sobre borrado prescribe sobrescribir tres veces: primero un
  **carácter aleatorio**, después un **carácter especificado** y por último su
  **complemento**, seguido de verificación.
- **Variante de herramientas**: 2 pasadas aleatorias. Es más sencilla que el procedimiento
  del manual y la que ofrece la mayoría de programas.
- **SecureDelete**:
  - `usarmy` (`R, R`): la variante de herramientas.
  - **`arm38019`** (`R, 00, FF`): el procedimiento del manual, con `0x00` como carácter
    especificado. Uso: `secure_delete -m arm38019 --verify fichero`.

---

## US Navy — NAVSO P-5239-26

- **Quién/cuándo**: *Remanence Security Guidebook*, **Módulo 26**, publicado en
  **septiembre de 1993** por el *Naval Information Systems Management Center*.
- **Qué dice**: adapta el patrón a la **codificación física del disco**, porque los bits
  que se escriben no son los que acaban en la superficie magnética:
  - discos **RLL**: una pasada de un patrón fijo, otra de su complemento y una
    **aleatoria**;
  - discos **MFM**: un patrón pensado para esa codificación, su complemento y una pasada
    aleatoria.

  En ambos casos exige verificar.
- **Hoy**: ningún disco actual usa RLL ni MFM, así que la distinción solo tiene interés
  histórico.
- **SecureDelete**:
  - `navy` (`R, R, R`): variante RLL simplificada a tres pasadas aleatorias.
  - `navy_mfm` (`R, ~R, R`): variante MFM, con el complemento de la primera pasada.

---

## US Air Force — AFSSI-5020

- **Quién/cuándo**: *Air Force System Security Instruction 5020, Remanence Security*,
  emitida en **1996**.
- **Qué dice**: tres pasadas, **`00`, `FF` y un carácter aleatorio**, y verificación.
- **Variante de herramientas**: `00, FF, 00` (tres patrones fijos, sin pasada aleatoria).
- **SecureDelete**:
  - `usaf` (`00, FF, 00`): la variante de herramientas.
  - **`afssi5020_official`** (`00, FF, R`): la secuencia del documento.

---

## NSA — "NSA 130-1" y NSA/CSS Manual 130-2

- **El fantasma "NSA 130-1"**: no existe ninguna publicación de la NSA con ese número y
  ese contenido. Es una atribución de proveedores.
- **El documento real**: **NSA/CSS Manual 130-2, *Media Declassification and Destruction
  Manual*** (**noviembre de 2000**).
  - **Qué dice**: sobrescritura en **tres pasadas**: **aleatorio, aleatorio y un patrón
    conocido**, con verificación.
  - **Sustituido por**: el **NSA/CSS Policy Manual 9-12** (2007 y revisiones posteriores).
    Para discos con información clasificada, ese manual ya no acepta la sobrescritura:
    exige desmagnetizar con un equipo homologado o destruir el soporte.
- **SecureDelete**:
  - `nsa` (`FF, FF`): la variante "NSA 130-1" de las herramientas; se conserva por
    compatibilidad.
  - **`nsa1302`** (`R, R, 00`): el procedimiento real del Manual 130-2.

---

## DOE — M 205.1-2

- **Quién/cuándo**: Departamento de Energía, Manual M 205.1-2 (*Departmental Manual for
  Personnel Security and Information Security*), fechado el **26 de junio de 2005**.
- **Qué dice**: el documento tiene **dos procedimientos**:
  - el **texto principal**, para material UCNI/NNPI, describe la purga por sobrescritura
    como **carácter, complemento, aleatorio** (`00, FF, R`);
  - el **Anexo 5, procedimiento 3**, prescribe **aleatorio, aleatorio, patrón conocido**,
    igual que el NSA/CSS 130-2.
- **SecureDelete**: método `doe` (`00, FF, R`), según el texto principal. Para el
  procedimiento del anexo se puede usar `nsa1302`, que tiene la misma secuencia.

---

## NIST — SP 800-88: la referencia vigente

- **Quién/cuándo**: Instituto Nacional de Estándares y Tecnología. Publicación especial
  800-88, *Guidelines for Media Sanitization*: original de **septiembre de 2006**,
  **Rev. 1 en diciembre de 2014** y **Rev. 2 el 26 de septiembre de 2025**.
- **Qué dice**: no define pasadas, sino **tres niveles de saneamiento** según la amenaza:

  | Nivel | Contra qué protege | Cómo se consigue |
  |---|---|---|
  | **Clear** | Recuperación con herramientas normales (software, comandos del disco) | Sobrescribir los datos de usuario; en discos magnéticos basta **una pasada** |
  | **Purge** | Recuperación en laboratorio | Comandos nativos del dispositivo (ATA Sanitize / Secure Erase, NVMe Format o Sanitize), borrado criptográfico o desmagnetización |
  | **Destroy** | Cualquier recuperación | Triturar, desintegrar, fundir o incinerar |

  Para soportes magnéticos modernos considera suficiente **una sola pasada** (preferiblemente
  aleatoria) y afirma que repetirla no aporta seguridad. En **SSD y memorias flash** la
  sobrescritura no garantiza llegar a todas las celdas (por la nivelación de desgaste y el
  sobreaprovisionamiento) y recomienda los comandos nativos del dispositivo. La Rev. 2
  pide además **verificar** el resultado y documentarlo.
- **SecureDelete**:
  - `random` (1 pasada aleatoria) con `--verify`: es lo que NIST recomienda para *Clear*
    en discos magnéticos. Uso: `secure_delete -m random --verify fichero`.
  - `--secure-erase` (con `--enhanced` para la variante criptográfica): usa los comandos
    nativos ATA o NVMe que NIST propone para *Purge* en SSD. Ver
    [El programa](el-programa.md).
  - `nist` (`R, ~R, R`, con el mismo patrón aleatorio en la primera y la tercera
    pasada): imita la sobrescritura con inversión de patrón que la Rev. 1 menciona como
    opción del comando *SANITIZE* del disco. Hecha desde el sistema operativo y no por
    el firmware, **no equivale a un Purge de NIST**; se ofrece por compatibilidad con
    otras herramientas.

---

## Secuencias propias del proyecto: `userm` y `userm2`

- **Qué son**: secuencias de 4 y 7 pasadas (`FF, 00, FF, R` y `FF, 00, R, 00, FF, R, R`)
  definidas originalmente en esta herramienta. No corresponden a ningún estándar
  publicado ni a ninguna herramienta histórica. Algunos listados antiguos las atribuían a
  `wipe` o a la NSA, y no es correcto.
- **SecureDelete**: métodos `userm` y `userm2`, conservados por compatibilidad con
  configuraciones existentes.

---

## ¿Qué método elegir hoy?

| Situación | Recomendación |
|---|---|
| Fichero o disco **magnético** (HDD) | `-m random --verify`: una pasada aleatoria verificada, como pide NIST SP 800-88 |
| Unidad **SSD / NVMe / USB flash** entera | `--device --secure-erase` (y `--enhanced` si el disco lo admite); la sobrescritura no llega a todas las celdas |
| Fichero en un SSD | La sobrescritura es solo de buena fe: el sistema puede haber escrito copias en otras celdas. Si importa de verdad, cifrado de disco completo desde el principio |
| Un procedimiento que pide "DoD 5220.22-M" | `-m dod --verify`, sabiendo que es una referencia histórica y no un requisito vigente |
| Reproducir exactamente un manual | Las variantes oficiales: `arm38019`, `afssi5020_official`, `nsa1302` |

Más pasadas no hacen el borrado más seguro en discos actuales: solo lo hacen más lento.
La historia de esta conclusión está en [Historia de la eliminación de datos](historia.md).
