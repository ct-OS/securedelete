# Secure Delete

Herramienta de línea de comandos escrita en **Rust** para borrar de forma segura ficheros o dispositivos de almacenamiento mediante sobrescritura de datos.

A diferencia de un borrado normal (que solo elimina la entrada del sistema de ficheros), `secure-delete` sobreescribe el contenido del fichero o dispositivo una o varias veces, de forma que la recuperación de los datos originales sea mucho más difícil.

> **Advertencia:** esta herramienta destruye datos de forma irreversible. Úsala solo sobre ficheros o dispositivos de los que quieras deshacerte definitivamente.

> **Wiki:** historia del programa, de cada método de borrado y de las herramientas de sanitización en [`wiki/`](wiki/README.md).

## Características

- Sobrescritura de ficheros o de **dispositivos de bloque** completos (por ejemplo `/dev/sdX`).
- Varios métodos de borrado: relleno con ceros, datos aleatorios, DoD 5220.22-M y Gutmann.
- Número de pasadas aleatorias configurable (`random:N` o `--passes N`).
- Barra de progreso con pasada actual, porcentaje y tiempo restante.
- `fsync`/`sync_all` por pasada y tras el truncado, para que los datos no queden en caché.
- Comprobaciones de seguridad: rechaza directorios, symlinks, la raíz del sistema y requiere `--force` para dispositivos de bloque.

## Métodos de borrado

| Método | Alias | Pasadas | Descripción |
| ------ | ----- | ------- | ----------- |
| `zeros` | `zero` | 1 | Sobrescribe todo con bytes `0x00`. Rápido, útil en SSDs. |
| `ones` | — | 1 | Sobrescribe todo con bytes `0xFF`. |
| `random` | — | 1 (configurable) | Sobrescribe con datos aleatorios criptográficamente seguros. |
| `usarmy` | `army38019` | 2 | Estándar US Army AR 380-19 (2 pasadas aleatorias). |
| `navy` | `navso`, `navso5239` | 3 | Estándar US Navy NAVSO P-5239-26 RLL (3 pasadas aleatorias). |
| `navy_mfm` | `navso_mfm`, `navso5239mfm` | 3 | US Navy NAVSO P-5239-26 MFM (aleatorio, complemento, aleatorio). |
| `usaf` | `afssi5020` | 3 | Estándar US Air Force AFSSI-5020 (0x00, 0xFF, 0x00). |
| `userm` | — | 4 | Secuencia interna del proyecto (0xFF, 0x00, 0xFF, aleatorio). |
| `dod` | `dod522022m` | 3 | DoD 5220.22-M (0x00, 0xFF, aleatorio). |
| `dod7` | `dod522022m7`, `dodece` | 7 | DoD 5220.22-M ECE (0x00, 0xFF, aleatorio ×3, 0x00). |
| `userm2` | — | 7 | Secuencia interna del proyecto (0xFF, 0x00, aleatorio, 0x00, 0xFF, aleatorio, aleatorio). |
| `hmg` | `isc5` | 3 | Estándar UK HMG IS5 (0x00, 0xFF, aleatorio). |
| `hmg5` | `isc5e`, `hmg_enhanced` | 5 | UK HMG IS5 Enhanced (0x00, 0xFF, aleatorio, 0x00, 0xFF). |
| `rcmp` | `tssit`, `opsii` | 4 | Estándar canadiense RCMP TSSIT OPS-II (0xFF, 0x00, 0xFF, 0x00). |
| `vsitr` | — | 7 | Estándar alemán VSITR (0xFF, 0x00, 0xFF + 4 aleatorias). |
| `gost` | `gost50739` | 2 | Estándar ruso GOST R 50739-95 (2× 0x00). |
| `nsa` | `nsa1301` | 2 | NSA 130-1 (2× 0xFF). |
| `schneier` | — | 7 | Algoritmo de Bruce Schneier (0xFF, 0x00 + 4 aleatorias, 0xFF). |
| `randomc` | `random_complement` | 2 | Aleatorio seguido de su complemento bit a bit. |
| `randomcc` | `complement_cycle` | 3 | Ciclo aleatorio → complemento → aleatorio. |
| `pfitzner` | `pfitzner33` | 33 | Método de Pfitzner (33 pasadas aleatorias). |
| `bmb21` | `bundeswehr` | 6 | Bundeswehr BMB21-2019 (0xFF, 0x00, aleatorio ×3, 0xFF). |
| `bsi` | `bsi9` | 9 | German BSI clásico (0xFF, 0xFE, 0xFD, 0xFB, 0xF7, 0xEF, 0xDF, 0xBF, 0x7F). |
| `doe` | `doe20512` | 3 | US DOE M 205.1-2 (0x00, 0xFF, aleatorio). |
| `rcmp7` | `b2002` | 7 | RCMP B2-002 (0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF, aleatorio). |
| `nist` | `nist80088` | 3 | NIST SP 800-88 Purge (patrón aleatorio, su inversión, el mismo patrón). |
| `gutmann` | — | 35 | 4 pasadas aleatorias + 31 patrones del algoritmo de Gutmann. Muy lento. |
| `wipe27` | `wipestatic` | 27 | 27 patrones estáticos del tool `ya-wipe` (derivados de Gutmann). |
| `arm38019` | `usarmy_official`, `army3` | 3 | AR 380-19 oficial (aleatorio, carácter especificado, complemento). |
| `afssi5020_official` | `usaf_official` | 3 | AFSSI-5020 oficial (0x00, 0xFF, aleatorio). |
| `nsa1302` | `nsa_official` | 3 | NSA/CSS 130-2 (aleatorio, aleatorio, patrón conocido 0x00). |
| `vsitr_official` | `vsitr_bsi` | 7 | VSITR/BSI Hinweisblatt Nr. 11 (0x00, 0xFF ×3 y 0x55). |
| `hmg_baseline` | `isc5_base`, `hmg1` | 1 | HMG IS5 Baseline (una pasada de ceros). |
| `fill:XX` | `pattern:XX` | 1 | Rellena con el byte hexadecimal `XX` (p. ej. `fill:aa`, `pattern:0xff`). |

Los datos aleatorios se generan con `rand` (`ChaCha12`/`ChaCha20`, alimentado por el sistema operativo).

> **Nota:** los métodos de los estándares militares/históricos (`usarmy`, `navy`, `navy_mfm`, `usaf`, `userm`, `userm2`, `dod7`, `hmg`, `hmg5`, `rcmp`, `rcmp7`, `vsitr`, `gost`, `nsa`, `schneier`, `bmb21`, `bsi`, `doe`, `pfitzner`, `wipe27`) se incluyen por completitud y por motivos documentales, con sus patrones más habituales (las variantes que difundieron otras herramientas). Los métodos `arm38019`, `afssi5020_official`, `nsa1302`, `vsitr_official` y `hmg_baseline` implementan en cambio la secuencia descrita en el documento oficial (ver [`wiki/`](wiki/README.md)). Según NIST SP 800-88, para la mayoría de los soportes modernos una sola sobrescritura con `zeros` o `random` es suficiente.

## Instalación

### Desde el código fuente

Requiere [Rust](https://rustup.rs/) (edición 2021).

```bash
git clone <url-del-repositorio>
cd SecureDelete
cargo build --release
```

El binario queda en `target/release/secure_delete`.

Opcionalmente, instálalo en el sistema:

```bash
cargo install --path .
```

### Uso rápido

```bash
# Borrar un fichero con el método por defecto (DoD, 3 pasadas)
secure-delete documento.pdf

# Borrar un fichero sin confirmación (p. ej. en scripts)
secure-delete -f documento.pdf

# Borrar con ceros (más rápido)
secure-delete -f -m zeros documento.pdf

# Borrar con 5 pasadas aleatorias
secure-delete -f -m random:5 documento.pdf
# o, de forma equivalente:
secure-delete -f -m random --passes 5 documento.pdf

# Borrar con el algoritmo de Gutmann (35 pasadas)
secure-delete -f -m gutmann documento.pdf

# Listar los métodos disponibles
secure-delete --list-methods
```

## Uso completo

```
Usage: secure_delete [OPTIONS] [PATH]

Arguments:
  [PATH]  Ruta del objetivo (fichero o dispositivo)

Options:
  -m, --method <METHOD>  Método de borrado (usa --list-methods para ver todos;
                         soporta random:N)
  -d, --device           Borrar un dispositivo/block device en vez de un fichero
  -f, --force            Omitir la confirmación (obligatorio para borrar dispositivos)
      --passes <N>       Número de pasadas para el método "random" (default: 1)
      --secure-erase     Borrado nativo del firmware (ATA Security Erase / NVMe
                         Format); ideal para SSD. Solo Linux y con -d
      --enhanced         Variante enhanced/criptográfica del borrado nativo
      --list-methods     Listar los métodos de borrado disponibles
  -h, --help             Mostrar la ayuda
```

### Borrar un dispositivo completo

Para borrar un dispositivo de bloque (p. ej. un disco USB o una partición):

```bash
sudo secure-delete -d -f -m random:1 /dev/sdX
```

Es **obligatorio** usar `-d` y `-f`. La herramienta se niega a borrar `/`, directorios, symlinks y ficheros que no sean regulares.

> ⚠️ Asegúrate de haber desmontado el dispositivo y de escribir bien el nombre (`/dev/sdX`). Un error aquí es irreversible.

### SSD: borrado nativo por firmware (solo Linux)

La sobrescritura no es fiable en SSD (el controlador reubica los bloques). Para discos
SATA y NVMe usa el borrado nativo del firmware con `--secure-erase`:

```bash
# SSD SATA: ATA Security Erase
sudo secure-delete -d --secure-erase /dev/sdX

# SSD NVMe: Format NVM con borrado seguro de datos de usuario
sudo secure-delete -d --secure-erase /dev/nvme0n1

# Variante enhanced/criptográfica de ATA Security Erase (si el disco la soporta)
sudo secure-delete -d --secure-erase --enhanced /dev/sdX
```

Internamente: detecta el tipo de medio con el ioctl `BLKROTATIONAL` (`/dev/sdX`), emite
*ATA Security Erase* vía `SG_IO` (ATA PASS-THROUGH 16) para SATA, y *Format NVM* con
*Secure Erase Setting* vía `NVME_IOCTL_ADMIN_CMD` para NVMe. Si el disco está *frozen*
(típico en portátiles), el ATA Security Erase falla: descongélalo con un ciclo
suspender/reanudar antes de reintentar. Para discos rotacionales rechaza el borrado
nativo y recomienda la sobrescritura (`-d -m <method>`).

## Cómo funciona

1. **Validación:** se comprueba que el objetivo sea un fichero regular o un block device, y que no se trate de un directorio, symlink ni la raíz del sistema.
2. **Confirmación:** se pide escribir `yes` a no ser que se use `--force`.
3. **Sobrescritura:** para cada pasada del método, el fichero/dispositivo se recorre de principio a fin rellenando buffers de 1 MiB. Tras cada pasada se ejecuta `fsync` para forzar el volcado a disco.
4. **Eliminación:** al terminar, el fichero se trunca a 0, se sincroniza de nuevo y se elimina.

## Limitaciones

- La sobrescritura no garantiza la destrucción física de datos en **SSD o discos con wear leveling / over-provisioning** (firmware del disco), en discos con sectores reasignados, o en copias de seguridad, versiones anteriores del sistema, journaling, etc. Para SSD se recomienda `--secure-erase` (comandos nativos del firmware), disponible en Linux.
- Los ficheros en sistemas de ficheros con **copia en escritura (COW)** como ZFS o Btrfs pueden dejar copias antiguas. En esos casos el borrado a nivel de dispositivo es más fiable.
- Para garantías reales de destrucción de datos, la destrucción física del soporte es el único método 100 % efectivo.

## Multiplataforma

La herramienta funciona en **Linux, macOS, FreeBSD, OpenBSD, NetBSD, DragonFly BSD, Windows** y otros Unix. El código de plataforma está aislado en `src/platform/`:

| Aspecto | Linux (`unix.rs`) | macOS (`macos.rs`) | FreeBSD/NetBSD/DragonFly (`bsd.rs`) | OpenBSD (`openbsd.rs`) | Windows (`windows.rs`) |
| ------- | ----------------- | ------------------ | ----------------------------------- | ---------------------- | ---------------------- |
| Detección del dispositivo | Block device (`FileTypeExt::is_block_device`) | Block **o** carácter (acepta `/dev/rdiskX`) | Block **o** carácter (los discos son nodos carácter) | Block **o** carácter (`/dev/rsd0c`) | Ruta `\\.\PhysicalDriveN` o `\\.\C:` |
| Tamaño del dispositivo | `seek(SeekFrom::End(0))` | ioctl `DKIOCGETBLOCKCOUNT` × `DKIOCGETBLOCKSIZE` | ioctl `DIOCGMEDIASIZE` | ioctl `DIOCGDINFO` (`struct disklabel`) | `DeviceIoControl(IOCTL_DISK_GET_LENGTH_INFO)` |
| Apertura del dispositivo | `open()` normal | `open()` normal | `open()` normal | `open()` normal | `CreateFile` con `share_mode(0)` y `FILE_FLAG_WRITE_THROUGH` |
| Raíz del sistema (prohibida) | `/` | `/` | `/` | `/` | Unidad raíz (`C:\`, `D:\`, …) |

El `seek(SeekFrom::End(0))` para obtener el tamaño solo es fiable en **Linux**. En macOS y los BSD, los nodos de dispositivo no reportan tamaño con `stat`/`lseek` (devuelven 0), así que se usan ioctls del sistema (`DKIOC…`/`DIOCG…`).

### Rutas de dispositivo por plataforma

| Plataforma | Ejemplos | Notas |
| ---------- | -------- | ----- |
| Linux | `/dev/sda`, `/dev/nvme0n1`, `/dev/mmcblk0` | Block devices clásicos. |
| macOS | `/dev/disk2` (bufferizado) o `/dev/rdisk2` (crudo, más rápido) | Se requiere `sudo`. Tamaño vía `DKIOCGETBLOCKCOUNT`/`DKIOCGETBLOCKSIZE`. |
| FreeBSD | `/dev/da0`, `/dev/ada0`, `/dev/nvd0` | Los discos son nodos **carácter** (no existe `/dev/rda0`). `DIOCGMEDIASIZE`. |
| OpenBSD | `/dev/rsd0c` (crudo; partición `c` = disco completo) | `DIOCGDINFO` + `struct disklabel`. También se acepta `/dev/sd0`. |
| NetBSD | `/dev/rwd0c` (crudo) | `DIOCGMEDIASIZE`. |
| DragonFly | `/dev/da0`, `/dev/nvme0` | `DIOCGMEDIASIZE`. |
| Windows | `\\.\PhysicalDrive1`, `\\.\C:` | Se requiere administrador; ver abajo. |

### Windows

- Los dispositivos se direccionan con rutas de dispositivo: `\\.\PhysicalDrive0`, `\\.\C:`, etc.
- Se requiere **administrador** y `--force`:
  ```powershell
  secure-delete.exe -d -f -m random:1 \\.\PhysicalDrive1
  ```
- El tamaño del disco se obtiene vía `IOCTL_DISK_GET_LENGTH_INFO` y la escritura usa `FILE_FLAG_WRITE_THROUGH` para evitar la caché del sistema.

### Compilar para cada plataforma

```bash
# Linux / macOS / BSD
cargo build --release

# Windows (cruzar desde Linux, solo comprueba; no enlaza)
rustup target add x86_64-pc-windows-gnu
cargo check --target x86_64-pc-windows-gnu

# macOS (cruzar desde Linux)
rustup target add x86_64-apple-darwin
cargo check --target x86_64-apple-darwin

# FreeBSD / NetBSD
rustup target add x86_64-unknown-freebsd x86_64-unknown-netbsd
cargo check --target x86_64-unknown-freebsd
cargo check --target x86_64-unknown-netbsd

# OpenBSD / DragonFly (sin std precompilado: se compila con build-std en nightly)
rustup +nightly component add rust-src
cargo +nightly check -Z build-std=core,alloc,std,proc_macro --target x86_64-unknown-openbsd
cargo +nightly check -Z build-std=core,alloc,std,proc_macro --target x86_64-unknown-dragonfly
```

Las dependencias `windows-sys` (Windows) y `libc` (macOS y los BSD) son exclusivas de sus targets (`[target.'cfg(...)'.dependencies]`), por lo que no afectan a las demás compilaciones.

## Desarrollo

```bash
# Compilar
cargo build

# Ejecutar los tests
cargo test

# Lint y formato
cargo clippy --all-targets
cargo fmt --check
```

## Licencia

MIT
