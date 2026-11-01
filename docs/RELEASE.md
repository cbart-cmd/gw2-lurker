# Publicación de la DLL para Nexus

El addon se distribuye desde **un repositorio compartido con los demás addons**:
`https://github.com/cbart-cmd/gw2addonsreleases`.

Regla de oro: **este repositorio no es nuestro**. Solo se tocan los ficheros de
`gw2_lurker`; nunca los de los proyectos hermanos (`timers`, `fight_analysis_danger`, …).

## Identidad de la distribución

| Dato | Valor |
|---|---|
| Fichero | `gw2_lurker.dll` |
| Checksum para Nexus | `gw2_lurker.dll.md5` (32 hex en minúsculas, **sin BOM ni salto de línea**) |
| `update_link` | `https://raw.githubusercontent.com/cbart-cmd/gw2addonsreleases/main/gw2_lurker.dll` |
| Proveedor | `UpdateProvider::Direct` |
| Flags | `AddonFlags::DisableHotloading` |
| Tag de release | `gw2-lurker-v<version>` siempre con `--latest=false` |

`DisableHotloading` no es opcional: en los proyectos hermanos el intercambio en caliente
dejaba dos instancias del addon con el juego abierto y reventaba el proceso
(0xC0000005 en `d3d11.dll`) justo después, al crear el atlas de fuentes de Nexus. Con el
flag, una actualización exige reiniciar el juego, que es lo que ya hacía falta.

El **MD5** es lo que parsea Nexus para decidir si hay versión nueva. El **SHA-256** que
publicamos aparte (RF-53) es nuestro, para que el usuario pueda comprobar la descarga;
no lo usa Nexus.

### El índice de nombres de clan (ficheros de datos)

`guild-names.json` (miles de nombres, RF-20c) **no va dentro de la DLL**: se publica como
`gw2_lurker_guilds.zip` + `gw2_lurker_guilds.zip.md5` en el mismo repositorio, y el addon lo
descarga la primera vez que arranca sin él (o cuando el jugador pulsa *Descargar el índice* en
los ajustes).

| Dato | Valor |
|---|---|
| Fichero | `gw2_lurker_guilds.zip` (un solo `guild-names.json` en la raíz) |
| Checksum | `gw2_lurker_guilds.zip.md5`, mismo formato que el de la DLL |
| URL por defecto | `https://raw.githubusercontent.com/cbart-cmd/gw2addonsreleases/main/gw2_lurker_guilds.zip` |
| Script | `scripts/pack-guild-index.ps1` (`-Publish` para subirlo) |

Se empaqueta y publica con:

```
pwsh -File scripts/pack-guild-index.ps1              # empaqueta y verifica, sin tocar lo remoto
pwsh -File scripts/pack-guild-index.ps1 -Publish      # sube el ZIP y su .md5
```

El script aborta si el JSON de origen está vacío, si el `.md5` no mide 32 bytes, si el ZIP no
lleva el `guild-names.json` dentro, si el blob preparado es un puntero LFS o si la descarga
publicada no coincide en bytes y MD5. La excepción de LFS para `gw2_lurker_guilds.zip` se añade
al `.gitattributes` del repo de distribución **antes** del `git add`.

El ZIP **no** cuelga de una release: vive en `main`, así que la URL es estable y no hay que
tocar nada en el addon para republicarlo.

## Flujo

Todo esto lo hace `scripts/release.ps1`; los pasos quedan aquí como contrato de lo que el
script comprueba y como salida manual si el script no se puede usar.

Para **probar en local** sin publicar nada: `install.bat` (doble clic) o
`pwsh -File install.ps1 [-Build] [-DryRun] [-Dest <ruta>]`. Compila si se pide, verifica
tamaño, `MZ` y MD5, y copia `gw2_lurker.dll` a `<juego>\addons`, pidiendo elevación solo si
hace falta. Se niega a copiar con GW2 abierto, porque el fichero está bloqueado.

1. Subir la versión en `Cargo.toml` y en `CHANGELOG.md`.
2. `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
   `cargo test --workspace`, `bash ./scripts/check_loc.sh`.
3. `cargo build --release -p gw2lurker-addon` (sin `--target`: la salida va a
   `target\release\gw2_lurker.dll`) y comprobar tamaño con `bash ./scripts/check_size.sh <dll>`.
4. Copiar la DLL a `..\gw2addonsreleases\gw2_lurker.dll`.
5. Calcular MD5 y escribirlo en `..\gw2addonsreleases\gw2_lurker.dll.md5`
   (exactamente 32 caracteres hexadecimales en minúsculas, sin BOM, sin `\n`).
6. En `..\gw2addonsreleases\.gitattributes`, **después** de la regla general de LFS,
   asegurar la excepción de nuestra DLL:
   ```gitattributes
   gw2_lurker.dll -filter -diff -merge -text
   ```
   Sin ella, git guarda un puntero LFS de ~130 bytes y Nexus sustituye el addon por ese
   texto (incidente real en un proyecto hermano).
7. Verificar el blob preparado: `git cat-file -s :gw2_lurker.dll` debe coincidir
   **exactamente** con el tamaño de la DLL compilada. `git check-attr filter diff merge text
   -- gw2_lurker.dll` debe devolver los cuatro atributos como `unset`.
8. Commit y push solo de nuestros ficheros:
   `git -C ..\gw2addonsreleases add gw2_lurker.dll gw2_lurker.dll.md5` y push a `origin/main`.
9. Crear la release con el tag `gw2-lurker-v<version>` y `--latest=false`, adjuntando
   DLL y MD5.
10. **Verificar de verdad**: descargar la URL exacta de `update_link` siguiendo redirecciones
    y comparar bytes y MD5 con la DLL local; debe empezar por `MZ`. Un HTTP 200 no
    verifica nada, y `Invoke-WebRequest.Content` no sirve para binarios: usar
    `HttpClient.GetByteArrayAsync`.

## Retención

Se mantienen **la release actual y la anterior** y nada más: al publicar `vN`, se borra
`v(N-2)` (tag y adjuntos). El fichero `gw2_lurker.dll` del repo siempre es la última.

## Comprobaciones que no se pueden saltar

- Un puntero LFS, una caché de CDN y una DLL correcta se parecen en tamaño a simple vista:
  comparar `MZ` + tamaño + MD5, siempre.
- Cambiar el endpoint no arregla una copia ya instalada: las DLL antiguas conservan su
  `update_link` grabado. Si una quedó rota, hay que sustituirla a mano una vez, con GW2
  cerrado y sin tocar los ajustes.
