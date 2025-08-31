# Estándares de Estados Unidos

Cada entrada indica el documento real (quién, cuándo, qué dice) y la variante que
implementa SecureDelete. La regla general: **los manuales definen *procedimientos*, no
tablas de pasadas**, y las "tablas de pasadas" que circulan por internet son casi siempre
folklore de vendedores.

---

## DoD 5220.22-M (NISPOM) — "DoD 3 pasadas"

- **Quién/cuándo**: Departamento de Defensa de EE. UU. El manual se publicó en **1995**
  y se actualizó con cambios conformes en 2013 y 2016; en **enero de 2021** se codificó
  como 32 CFR Parte 117.
- **Qué dice**: es el manual que regula el *Programa Nacional de Protección Industrial de
  Seguridad* (NISPOM). **No contiene ningún esquema de "3 pasadas"**. El "borrado DoD
  00, FF, aleatorio" es folklore de proveedores de software de los 90.
- **En 2006, el DoD eliminó de su manual todas las especificaciones de sobrescritura**,
  dejando la decisión a los responsables de seguridad.
- **SecureDelete**: método `dod` (3 pasadas 00, FF, aleatorio) por tradición; se mantiene
  por compatibilidad histórica con otras herramientas.

---

## DoD ECE — "DoD 7 pasadas"

- **Quién/cuándo**: variante *Extra Completely Erase* que circuló en memorandos internos
  del DoD alrededor de **2001**.
- **Qué dice**: 00, FF, aleatorio, 00, FF, aleatorio, 00.
- **Estado**: **no es un manual oficial**; nunca apareció en ningún documento publicado.
- **SecureDelete**: método `dod7` (7 pasadas).

---

## US Army — AR 380-19

- **Quién/cuándo**: Departamento del Ejército de EE. UU. Regulación 380-19 (*Information
  Systems Security*), edición de referencia del **27 de febrero de 1998**.
- **Qué dice**: el apéndice sobre borrado prescribe sobrescribir tres veces: primero un
  **carácter aleatorio**, después un **carácter especificado**, y por último el
  **complemento**, seguido de verificación.
- **SecureDelete**: método `usarmy` (2 pasadas aleatorias). Es la variante que usan la
  mayoría de las herramientas (2 × aleatorio), más sencilla que el procedimiento real
  del manual (3 ×: R, carácter, complemento). El método **`arm38019`** implementa la
  secuencia del manual: aleatorio, carácter especificado (0x00) y su complemento (0xFF).

---

## US Navy — NAVSO P-5239-26

- **Quién/cuándo**: *Remanence Security Guidebook*, **Módulo 26**, publicado en
  **septiembre de 1993** por el Naval Information Systems Management Center.
- **Qué dice**: para borrar medios magnéticos propone una pasada de **todos unos** (RLL),
  una **patrón MFM/RLL**, y un **patrón no lineal seudoaleatorio** (el trío
  carácter/complemento/aleatorio lo atribuye a DoD 5200.28-M). Requiere verificación.
- **SecureDelete**: métodos `navy` (3 aleatorias, variante RLL) y `navy_mfm` (aleatorio,
  complemento, aleatorio, variante MFM).

---

## US Air Force — AFSSI-5020

- **Quién/cuándo**: *AFSSI-5020, Remanence Security* de la Fuerza Aérea de EE. UU.,
  emitido en **1996**.
- **Qué dice**: tres pasadas: **00, FF, aleatorio**, con verificación. (Otra variante que
  circula es 00, FF, 00.)
- **SecureDelete**: método `usaf` (00, FF, 00), la variante de 3 patrones fijos. El
  método **`afssi5020_official`** implementa la secuencia oficial del documento
  (00, FF, aleatorio).

---

## NSA — "NSA 130-1" y NSA/CSS Manual 130-2

- **Quién/cuándo**: el **"NSA 130-1"** es un *fantasma de proveedores*: no existe ninguna
  publicación de la NSA con ese número y ese contenido. El documento real es el
  **NSA/CSS Manual 130-2, *Media Declassification and Destruction Manual* (noviembre de 2000)**.
- **Qué dice (130-2)**: procedimiento de desclasificación por sobrescritura de **tres
  pasadas**: aleatorio, aleatorio, patrón conocido (R, R, conocido), con verificación.
- **SecureDelete**: método `nsa` (2 pasadas de FF), la variante "NSA 130-1" que
  popularizaron las herramientas y que no corresponde a ningún documento publicado.
  El método **`nsa1302`** implementa el procedimiento real del Manual 130-2
  (aleatorio, aleatorio, patrón conocido 0x00).

---

## DOE — M 205.1-2

- **Quién/cuándo**: Departamento de Energía de EE. UU., Manual M 205.1-2
  (*Departmental Manual for Personnel Security and Information Security*), con fecha de
  **26 de junio de 2005**.
- **Qué dice**: el texto principal describe la purga por sobrescritura como
  **carácter, complemento, aleatorio** (00, FF, R) para material UCNI/NNPI; el Anexo 5,
  procedimiento 3, prescribe en cambio **aleatorio, aleatorio, patrón conocido** (R, R, R/patrón).
- **SecureDelete**: método `doe` (00, FF, aleatorio), siguiendo el texto principal del manual.

---

## NIST — SP 800-88

- **Quién/cuándo**: Instituto Nacional de Estándares y Tecnología. Publicación especial
  800-88, *Guidelines for Media Sanitization*: original de **septiembre de 2006**,
  **Rev 1 en diciembre de 2014** y **Rev 2 el 26 de septiembre de 2025**.
- **Qué dice**: define tres estados — **Clear** (borrado lógico), **Purge** (inaccesible a
  la recuperación física) y **Destroy** (destrucción física). **No prescribe número de
  pasadas**: para medios magnéticos modernos considera suficiente **una sola pasada
  aleatoria**, y para SSD recomienda comandos nativos (ATA Secure Erase, NVMe) o borrado
  criptográfico.
- **SecureDelete**: método `nist` (aleatorio, complemento, mismo patrón) es una *variante
  de "Purge"* de 3 pasadas que no está en el documento; se ofrece por compatibilidad con
  otras herramientas. La recomendación real de NIST está cubierta por el método `random`
  (1–2 pasadas aleatorias).

---

## `userm` y `userm2` — secuencias internas del proyecto

- **Qué son**: secuencias de 4 y 7 pasadas definidas originalmente en esta herramienta
  (FF,00,FF,R y FF,00,R,00,FF,R,R). No corresponden a ningún estándar publicado ni a
  ningún tool histórico; no es correcto atribuirlas al tool `wipe` ni a la NSA, como
  hicieron algunos listados antiguos.
- **SecureDelete**: métodos `userm` y `userm2`. Se conservan por compatibilidad con
  configuraciones existentes.
