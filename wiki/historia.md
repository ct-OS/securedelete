# Historia de la eliminación de datos

La necesidad de "borrar de verdad" nace con los medios magnéticos: borrar un archivo del
índice del sistema de archivos no elimina los datos físicos. A ese residuo que queda en el
medio se le llama **remanencia de datos** (*data remanence*), y los métodos de borrado
seguro no son otra cosa que el intento de reducirla a niveles aceptables.

## Hitos principales

| Año | Hito |
|---|---|
| 1940 | Durante la Segunda Guerra Mundial se acuña el término **desmagnetizar** (*degauss*): el Comandante **Charles F. Goodeve** de la Royal Navy lo usa para describir el proceso que protegía a los barcos de las minas magnéticas. Las máquinas *degausser* se convierten después en la primera tecnología de destrucción de datos de medios magnéticos. |
| ~1960 | Se documenta la remanencia magnética como problema para la seguridad de la información. |
| 1970s–80s | El Departamento de Defensa de EE. UU. empieza a publicar especificaciones de desmagnetizadores (p. ej. `L14-4-75` de 1975 y `L14-4-A` de 1985). |
| 15 nov 1985 | **CSC-STD-005-85** del NCSC (DoD) formaliza el borrado por sobrescritura y la desmagnetización para el borrado y la purga. |
| 1993 | **NAVSO P-5239-26**, *Remanence Security Guidebook*, módulo 26 de la Armada de EE. UU. |
| 1994 | **Bruce Schneier** publica *Applied Cryptography* y describe su esquema de 7 pasadas (FF, 00, y cinco aleatorias). |
| julio 1996 | **Peter Gutmann** publica su artículo *Secure Deletion of Data from Magnetic and Solid-State Memory* en el 6.º Simposio de Seguridad de la USENIX: 35 pasadas. Se convierte en el esquema más citado de la historia. |
| 1995–2003 | La gran era de los **estándares militares** por sobrescritura: DoD 5220.22-M (1995), RCMP TSSIT (1995), AFSSI-5020 (1996), AR 380-19 (1998), VSITR (1998), GOST R 50739-95 (1995), DOE M 205.1-2 (2005). |
| sept 2006 | **NIST SP 800-88** introduce el marco *Clear / Purge / Destroy* y declara que la sobrescritura con **una sola pasada** es suficiente para medios magnéticos — y que "la sobrescritura múltiple no aporta nada" para discos de hoy. |
| 2006 | El DoD **retira de su NISPOM toda especificación de sobrescritura** de datos (las famosas "pasadas DoD" dejan de estar en el manual). |
| 2009 | Pádraig Brady reduce el valor por defecto de `shred` de 25 a 3 pasadas y añade una nota explicando que una sola pasada es suficiente. |
| 2011–2014 | **HMG IS5** del CESG/GCHQ (Reino Unido): 1 pasada de ceros (Baseline) o 3 pasadas (Enhanced). NIST publica la Rev 1 de SP 800-88 (2014). |
| 2013–2015 | Se popularizan ATA **Secure Erase** y el borrado por comandos nativos del SSD (TRIM/NVMe), que superan a la sobrescritura lógica en discos de estado sólido. |
| 26 sept 2025 | **NIST SP 800-88 Rev 2** actualiza el marco y consolida el fin del mito de las múltiples pasadas: para medios magnéticos modernos se recomienda una pasada única con patrón aleatorio. |

## Dos ideas clave

1. **La sobrescritura destruye datos por densidad, no por cantidad de pasadas.** Los
   discos magnéticos modernos tienen densidades tan altas que los residuos de una
   escritura anterior son indetectables incluso con una sola pasada. Los esquemas de
   muchas pasadas (Gutmann, Pfitzner, "38 pasadas") tienen sentido para *tecnologías de
   grabación antigua* y para disquetes/cintas de baja densidad — y aun así son
   ampliamente considerados exceso de precaución desde ~2006.

2. **La cantidad de pasadas nunca fue obligatoria en los estándares.** Muchos
   "estándares de N pasadas" que circulan (DoD 3, DoD 7, VSITR 7…) son **folklore de
   proveedores de software**: versiones simplificadas o inventadas de documentos reales,
   repetidas durante décadas en manuales de herramientas. La wiki detalla en cada página
   de estándares qué dice el documento original y qué hace en realidad esta herramienta.

## El fin de una era: discos de estado sólido

Con los SSD, la sobrescritura lógica deja de ser fiable (el controlador reubica bloques
internamente y puede no reescribir las celdas físicas). Por eso los estándares modernos
(NIST, IEEE 2883) recomiendan para SSD:

- **Purge** por comando nativo: ATA Secure Erase, NVMe Format/Erase, o el borrado
  criptográfico (destruir la clave de cifrado del disco).

SecureDelete cubre la sobrescritura de medios magnéticos tradicionales; para SSD modernos
la vía correcta son los comandos nativos del fabricante.
