# Los métodos en detalle

Cada método es una secuencia de pasadas que SecureDelete escribe sobre los datos.
Internamente cada método implementa `WipeMethod` y puede:

- escribir **patrones fijos** (bytes como `0x00`, `0xFF`, o secuencias estáticas),
- escribir **aleatorios** (rellenados con un PRNG criptográfico),
- escribir **complementos** (el byte actual más su inverso lógico `~b`),
- combinar todo lo anterior.

Los métodos se registran en `src/methods/mod.rs` con nombre canónico y aliases.

## Tabla general

| Método | Alias | Pasadas | Patrón | Origen / creador (año) |
|---|---|---|---|---|
| `zero`/`zeros` | — | 1 | `00` | Convención general |
| `ones` | — | 1 | `FF` | Convención general |
| `random` | — | N (1) | aleatorias | Convención general |
| `usarmy` | `army38019` | 2 | R, R | DoD 5220.22-M/AR 380-19 (variante de herramientas) |
| `navy` | `navso`, `navso5239` | 3 | R, R, R | NAVSO P-5239-26 (variante RLL) |
| `navy_mfm` | `navso_mfm`, `navso5239mfm` | 3 | R, ~R, R | NAVSO P-5239-26 (variante MFM) |
| `usaf` | `afssi5020` | 3 | 00, FF, 00 | AFSSI-5020 (variante de herramientas) |
| `dod` | `dod522022m` | 3 | 00, FF, R | "DoD 5220.22-M" (folklore de proveedores) |
| `dod7` | `dod522022m7`, `dodece` | 7 | 00,FF,R,00,FF,R,00 | DoD ECE (variante no oficial, ~2001) |
| `hmg` | `isc5` | 3 | 00, FF, R | HMG IS5 Baseline/Enhanced (variante) |
| `hmg5` | `isc5e`, `hmg_enhanced` | 5 | 00, FF, R, 00, FF | HMG IS5 Enhanced (variante de 5 pasadas) |
| `rcmp` | `tssit`, `opsii` | 4 | FF, 00, FF, 00 | RCMP TSSIT OPS-II (simplificación de 4 pasadas) |
| `rcmp7` | `b2002` | 7 | 00,FF,00,FF,00,FF,R | RCMP B2-002 / TSSIT OPS-II (7 pasadas) |
| `vsitr` | — | 7 | FF,00,FF,R,R,R,R | VSITR/BSI Hinweisblatt Nr. 11 (variante de herramientas) |
| `gost` | `gost50739` | 2 | 00, 00 | GOST R 50739-95 (variante de herramientas) |
| `nsa` | `nsa1301` | 2 | FF, FF | "NSA 130-1" (fantasma de proveedores; el manual real es 130-2) |
| `schneier` | — | 7 | FF, 00, R×5 | Bruce Schneier, *Applied Cryptography* (1994) |
| `userm` | — | 4 | FF, 00, FF, R | Secuencia interna del proyecto (sin estándar publicado) |
| `userm2` | — | 7 | FF,00,R,00,FF,R,R | Secuencia interna del proyecto (sin estándar publicado) |
| `gutmann` | — | 35 | 4 R + 31 patrones | Peter Gutmann, artículo USENIX (1996) |
| `randomc` | `randomcomp` | 2 | R, ~R | Convención general (patrón + complemento) |
| `randomcc` | — | 3 | R, ~R, R | Convención general |
| `pfitzner` | `pfitzner33` | 33 | R×33 | Roy Pfitzner, comisionado de protección de datos de Brandeburgo (2003) |
| `bmb21` | `bundeswehr` | 6 | FF,00,R,R,R,FF | Bundeswehr, BMB21-2019 |
| `bsi` | `bsi9` | 9 | FF,FE,FD,FB,F7,EF,DF,BF,7F | "BSI 9 pasadas" (atribuido por `scrub`; no es estándar primario) |
| `doe` | `doe20512` | 3 | 00, FF, R | DOE M 205.1-2 (2005) |
| `nist` | `nist80088` | 3 | P, ~P, P | NIST SP 800-88 "Purge" (variante de 3 pasadas) |
| `wipe27` | `wipestatic` | 27 | 27 patrones estáticos | ya-wipe 2.3.1, Thomas M. Vier Jr. (2009) |
| `arm38019` | `usarmy_official`, `army3` | 3 | R, carácter, ~carácter | AR 380-19 (secuencia del documento oficial) |
| `afssi5020_official` | `usaf_official` | 3 | 00, FF, R | AFSSI-5020 (secuencia del documento oficial) |
| `nsa1302` | `nsa_official` | 3 | R, R, patrón conocido | NSA/CSS Manual 130-2 (nov 2000) |
| `vsitr_official` | `vsitr_bsi` | 7 | 00,FF,00,FF,00,FF,55 | VSITR + BSI Hinweisblatt Nr. 11 (secuencia oficial) |
| `hmg_baseline` | `isc5_base`, `hmg1` | 1 | 00 | HMG IS5 Baseline (1 pasada de ceros) |
| `fill` | — | 1 | byte dado por el usuario | Convención general |

`P` = patrón (aleatorio en `nist`); `R` = pasada aleatoria; `~R` = complemento.

## Sobre las "pasadas DoD" y los mitos

- **DoD 5220.22-M (NISPOM), publicado en 1995**: el manual **no contiene** el famoso
  esquema de "3 pasadas". El "borrado DoD de 3 pasadas" (00, FF, aleatorio) es una
  invención de proveedores de software de finales de los 90 que se repitió hasta
  convertirse en mito. En **2006 el DoD eliminó de su manual todas las especificaciones
  de sobrescritura**.
- **DoD ECE de 7 pasadas**: tampoco es oficial; es un apéndice informal circulado en
  memorandos internos (~2001) que nunca formó parte de un manual publicado.
- Por eso esta wiki distingue siempre entre **el documento real** y **la variante que
  usan las herramientas** (incluida esta). Ver [estandares-eeuu.md](estandares-eeuu.md).

## Sobre las variantes "oficiales"

Junto a los métodos convencionales (las secuencias que difundieron las herramientas y que
pueden diferir del documento original), SecureDelete incluye variantes **`_official`** que
implementan la secuencia descrita en el documento oficial:

- **`arm38019`** — AR 380-19: aleatorio, carácter especificado (0x00) y su complemento (0xFF).
- **`afssi5020_official`** — AFSSI-5020: 00, FF, aleatorio.
- **`nsa1302`** — NSA/CSS 130-2: aleatorio, aleatorio, patrón conocido (0x00).
- **`vsitr_official`** — VSITR/BSI Hinweisblatt Nr. 11: 00/FF alternados ×6 y 7.ª pasada de 0x55.
- **`hmg_baseline`** — HMG IS5 Baseline: una pasada de ceros.

Nota: los métodos convencionales (`usarmy`, `usaf`, `nsa`, `vsitr`, `hmg`, `hmg5`) se
conservan intactos por compatibilidad con otras herramientas y con configuraciones
existentes; la distinción entre "documento real" y "variante de herramienta" se explica en
las páginas de estándares.

## Notas de precisión

- **NSIST**: el método `nist` de esta herramienta (aleatorio, complemento, mismo patrón)
  es una *variante de borrado* que el propio NIST no prescribe: SP 800-88 **no fija
  número de pasadas** y recomienda una sola pasada aleatoria (o comandos nativos) para
  medios modernos.
- **VSITR**: el documento original del BSI prescribe 6 pasadas de 00/FF alternados y una
  7.ª de 0x55 o 0xAA; la variante `vsitr` de esta herramienta usa FF, 00, FF y cuatro
  aleatorias, tal como la popularizaron herramientas de la época.
- **GOST R 50739-95**: el estándar ruso define clases de seguridad y recomienda ceros
  para las clases inferiores y aleatorio para las superiores; `gost` (00, 00) es la
  simplificación que popularizaron las herramientas.
- **Gutmann**: el artículo recomienda, en realidad, que los discos modernos solo necesiten
  4 pasadas aleatorias y deja el esquema de 35 para tecnología antigua.
