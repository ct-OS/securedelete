# Secure Delete

Herramienta de línea de comandos escrita en **Rust** para borrar de forma segura ficheros mediante sobrescritura de datos.

A diferencia de un borrado normal (que solo elimina la entrada del sistema de ficheros), `secure-delete` sobreescribe el contenido del fichero una o varias veces, de forma que la recuperación de los datos originales sea mucho más difícil.

> **Advertencia:** esta herramienta destruye datos de forma irreversible. Úsala solo sobre ficheros de los que quieras deshacerte definitivamente.

## Uso

```bash
cargo build --release

# Borrar un fichero con ceros
secure-delete -m zeros documento.pdf

# Borrar con el método por defecto (DoD 5220.22-M, 3 pasadas)
secure-delete documento.pdf
```

## Cómo funciona

1. **Validación:** se comprueba que el objetivo sea un fichero regular.
2. **Sobrescritura:** para cada pasada del método, el fichero se recorre de principio a fin rellenando buffers de 1 MiB. Tras cada pasada se ejecuta `fsync` para forzar el volcado a disco.
3. **Eliminación:** al terminar, el fichero se trunca a 0, se sincroniza de nuevo y se elimina.

## Licencia

MIT
