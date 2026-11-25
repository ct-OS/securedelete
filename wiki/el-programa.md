# El programa: SecureDelete

**SecureDelete** es una herramienta de línea de comandos escrita en **Rust** que borra
datos de forma segura sobrescribiendo el contenido de archivos o de dispositivos de
bloque completos. En lugar de confiar en que el sistema elimine el archivo (que en la
mayoría de sistemas de archivos solo marca los bloques como libres y deja los datos
físicos intactos), sobrescribe cada bloque con patrones de bytes definidos por el
método elegido.

## Qué hace

- **Sobrescritura de archivos**: sobrescribe el archivo en el sitio y después lo elimina.
- **Borrado de dispositivos**: sobrescribe un dispositivo de bloque entero (por ejemplo
  un disco o una partición) leyendo su tamaño real a través del kernel.
- **28 métodos de borrado** (ver [metodos.md](metodos.md)): desde una sola pasada de
  ceros hasta esquemas de 35 pasadas (Gutmann) o 33 aleatorias (Pfitzner).
- **Verificación** opcional de que la sobrescritura se realizó correctamente.
- **Barra de progreso** y cancelación limpia con `Ctrl+C`.
- **Comprobaciones de seguridad** antes de tocar un dispositivo (evita dispositivos de
  sistema, de arranque o montados).

## Plataformas

| Sistema operativo | Soporte | Cómo se obtiene el tamaño del dispositivo |
|---|---|---|
| Linux | Sí (nativo) | `ioctl(BLKGETSIZE64)` |
| Windows | Sí (MSVC o GNU) | `DeviceIoControl(IOCTL_DISK_GET_DRIVE_GEOMETRY_EX)` |
| macOS | Sí | `ioctl(DKIOCGETBLOCKCOUNT)` / `DKIOCGETBLOCKSIZE` |
| FreeBSD | Sí | `ioctl(DIOCGMEDIASIZE)` |
| OpenBSD | Sí | `ioctl(DIOCGDINFO)` + `struct disklabel` |
| NetBSD | Sí | `ioctl(DIOCGMEDIASIZE)` |
| DragonFly BSD | Sí | `ioctl(DIOCGMEDIASIZE)` |

Los cinco sistemas *nix* usan la dependencia `libc` (declarada en `Cargo.toml` solo para
esos objetivos) para las llamadas `ioctl`. En macOS y BSD los dispositivos de bloque son
*character devices*, por lo que el detector de dispositivos los acepta en esas plataformas.

## Uso

```
secure_delete [OPCIONES] RUTA...
```

Opciones principales (ver `--help` para la lista completa):

- `-m, --method <MÉTODO>` — método de borrado (por defecto `zero`).
- `-d, --device` — tratar la ruta como dispositivo de bloque y borrarlo entero.
- `-n, --passes <N>` — número de pasadas (para métodos aleatorios o de pasadas variables).
- `-v, --verify` — leer y verificar que los datos quedaron sobrescritos.
- `-l, --list-methods` — listar los métodos disponibles con sus aliases.

### Ejemplos

```
secure_delete -m dod7 documento.pdf
secure_delete -m nist -v /mnt/disco-usb
secure_delete -d -m random /dev/sdb
secure_delete --list-methods
```

## Arquitectura

```
src/
├── main.rs          → CLI, argumentos, ayuda
├── wiper.rs         → bucle de sobrescritura, progreso, cancelación
├── methods/
│   ├── mod.rs       → registro de los 28 métodos y sus aliases
│   ├── standards.rs → métodos de estándares con patrones fijos (dod, usaf, vsitr…)
│   ├── random.rs    → métodos basados en pasadas aleatorias
│   ├── complement.rs→ métodos de patrón + complemento (y NIST Purge)
│   └── wipe.rs      → los 27 patrones estáticos del tool `ya-wipe`
└── platform/
    ├── mod.rs       → despacho por sistema operativo
    ├── unix.rs      → Linux (ioctl BLKGETSIZE64)
    ├── macos.rs     → DKIOCGETBLOCKCOUNT / DKIOCGETBLOCKSIZE
    ├── bsd.rs       → FreeBSD/NetBSD/DragonFly (DIOCGMEDIASIZE)
    └── openbsd.rs   → DIOCGDINFO + struct disklabel
```

## Construcción

Las versiones publicadas se distribuyen como paquetes `.deb` para Deepin y Debian (línea de
órdenes e interfaz gráfica DTK6), `.zip` para Windows y un binario universal para macOS;
los detalles están en el [README](../README.md#paquetes-publicados). Para compilar desde
el código:

```
cargo build --release
cargo test          # pruebas unitarias de métodos, wiper y dispositivos
cargo clippy --all-targets
```

El binario resultante es `target/release/secure_delete`.
