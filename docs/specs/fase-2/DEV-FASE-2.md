# DEV-FASE-2 — Definición: Cifrado y almacenamiento (Crypto & storage)

> Definición de la segunda fase de desarrollo de **Cofre**.
> Derivada de `docs/PRD.md` (M2 Crypto & storage) y de `docs/SPEC.md`.
> Estado: **definido**. Las tareas de implementación se detallarán en una etapa posterior.

## 1. Objetivo y alcance

La segunda fase de desarrollo de Cofre convierte el **esqueleto funcional** de la
fase 1 en una aplicación con **cifrado en reposo y persistencia reales**: derivación
de clave Argon2id, cifrado autenticado XChaCha20-Poly1305, formato binario `.cofre`,
creación y apertura de vault con master password, y CRUD completo de credenciales.

Esta fase **no implementa aún las funciones de confort de UX**: no hay generador de
contraseñas, ni búsqueda/filtros, ni portapapeles con auto-clear, ni auto-lock, ni
pantalla de ajustes funcional. Las pantallas `unlock`, `list`, `detail` y `form` de
la fase 1 se hacen **funcionales** dentro del alcance de crypto/almacenamiento/CRUD.

### 1.1 Incluido en esta fase

- Módulos nuevos de negocio: `crypto.rs`, `storage.rs`, `models.rs`; extensión de
  `errors.rs` (el skeleton de fase 1 se mantiene).
- Derivación de clave **Argon2id** con los parámetros por defecto del SPEC §3.1 y
  salt aleatorio de 16 bytes por vault (con `OsRng`).
- Cifrado autenticado **XChaCha20-Poly1305** (SPEC §3.2) con nonce nuevo por guardado
  y tag Poly1305 anexado al ciphertext.
- Formato binario `.cofre` **v1** (SPEC §3.3): escritura, lectura, validación de
  `magic`/`version` y estructura de campos big-endian.
- Payload JSON (SPEC §3.4): modelos `Entry`, `Vault`, `Settings`, campo `updated_at`.
- **Creación de vault**: master password con input oculto, confirmación, regla de
  mínimo 8 caracteres, salt nuevo y guardado inicial.
- **Apertura de vault (unlock)**: verificación por autenticación del cifrado; 3
  intentos fallidos → backoff de 5 s con cuenta atrás visible.
- **CRUD de entradas** (SPEC §4.2): crear, listar, ver, editar y borrar con
  confirmación; `form` funcional con validación de `title`/`username`.
- Guardado **atómico** (SPEC §8): escritura a `.tmp` + `rename`; el vault nunca queda
  truncado si falla el guardado.
- `zeroize` de master password, clave derivada y passwords de entradas al descartarlas.
- Manejo de errores de archivo/disco con mensajes claros y sin `panic`.
- Tests: round-trip de crypto, discriminación de clave errónea, formato binario con
  fixtures `tests/fixtures/*.cofre`, lógica CRUD y guardado atómico.

### 1.2 Excluido de esta fase (fases posteriores)

- Generador de contraseñas, búsqueda/filtros, portapapeles y auto-lock → M3 (fase 3).
- Pantalla `settings` funcional y cambio de master password → M4 (fase 4).
- `require_password_on_delete` (depende de settings, M4); el borrado se confirma
  inline con `y`/`n` en esta fase.
- Detección de contraseñas débiles/reutilizadas e import/export → extensiones futuras.
- Migraciones de versión del formato (solo se soporta `version = 1`; la
  infraestructura de versión ya existe en el formato).
- Fuzzing del parseo binario (nice-to-have del SPEC §11, fuera de esta fase).

## 2. Requerimientos detallados

### 2.1 Dependencias nuevas

Se añaden a las de fase 1 (`crossterm`, `ratatui`):

| Crate | Uso |
|---|---|
| `argon2` | KDF Argon2id (derivación de clave) |
| `chacha20poly1305` | Cifrado autenticado XChaCha20-Poly1305 |
| `rand` | `OsRng` para salt, nonce e ids |
| `zeroize` | Borrado seguro de claves y passwords en drop |
| `serde` + `serde_json` | Serialización de la payload |
| `uuid` | Ids de entradas (uuid-v4) |
| `time` | Timestamp ISO 8601 de `updated_at` |

- Versiones estables actuales, fijadas en el lock al implementar; compatibles entre
  sí y con `ratatui`/`crossterm` de fase 1.
- Ninguna dependencia de red; todo es local.

### 2.2 Estructura de módulos

```text
cofre/
├── Cargo.toml
├── src/
│   ├── main.rs            # Entry point, orquestación init/teardown (fase 1)
│   ├── app.rs             # Estado global, máquina de pantallas (fase 1)
│   ├── terminal.rs        # Setup ratatui/crossterm (fase 1)
│   ├── ui/                # Render por pantalla (fase 1)
│   ├── crypto.rs          # Argon2id + XChaCha20-Poly1305 + formato (nuevo)
│   ├── storage.rs         # Lectura/escritura del archivo .cofre (nuevo)
│   ├── models.rs          # Entry, Vault, Settings (nuevo)
│   └── errors.rs          # Tipos de error base y de storage/crypto (extendido)
└── tests/
    └── fixtures/          # Archivos .cofre de prueba (nuevo)
```

- `models.rs` define los tipos; `crypto.rs` deriva clave y cifra/descifra;
  `storage.rs` solo hace I/O del archivo; `app.rs`/`ui/` integran el estado con las
  pantallas ya esqueletizadas en fase 1.
- La separación de responsabilidades de fase 1 se mantiene; no se acopla crypto a UI.

### 2.3 Modelos de datos (`models.rs`)

- `Entry { id: Uuid, title: String, username: String, password: String, url: String, notes: String, tags: Vec<String> }`.
- `Settings { auto_lock_minutes: u32, clipboard_seconds: u32 }` con defaults
  `5` / `15`. Se serializan en esta fase (SPEC §3.4) aunque la pantalla de ajustes
  no sea funcional hasta M4.
- `Vault { version: u8, updated_at: OffsetDateTime, settings: Settings, entries: Vec<Entry> }`.
- Serialización JSON acorde al SPEC §3.4: `url`/`notes` como string vacío si no
  existen, `tags` como array (puede ir vacío), `version: 1`.

### 2.4 Crypto (`crypto.rs`)

- `derive_key(master_password, salt, params) -> [u8; 32]`: Argon2id con
  `m_cost = 19456 KiB`, `t_cost = 2`, `p_cost = 1` (SPEC §3.1); salt de 16 bytes.
- `seal(plaintext, key) -> (nonce, ciphertext_with_tag)`: XChaCha20-Poly1305 con
  nonce de 24 bytes nuevo por operación, generado con `OsRng` (SPEC §3.2).
- `open(nonce, ciphertext_with_tag, key) -> Result<plaintext>`: el fallo de
  autenticación indica "contraseña incorrecta **o** archivo corrupto" (no se distingue).
- La clave derivada **nunca se escribe en disco**; vive solo en memoria durante la
  sesión y se borra con `zeroize`.

### 2.5 Formato del archivo `.cofre` y guardado

- Layout binario del SPEC §3.3, enteros big-endian:

```text
│ 0   │ 7  │ Magic "COFRE1"            │
│ 7   │ 1  │ Format version (u8) = 1   │
│ 8   │ 4  │ argon2 m_cost (u32, KiB)  │
│ 12  │ 4  │ argon2 t_cost (u32)       │
│ 16  │ 4  │ argon2 p_cost (u32)       │
│ 20  │ 16 │ Salt                      │
│ 36  │ 24 │ XChaCha nonce             │
│ 60  │ var│ Ciphertext (payload JSON) │
│ +len│ 16 │ Poly1305 tag              │
```

- **Escritura** (SPEC §3.5): mutar estado → `updated_at = now` → serializar JSON →
  `seal()` → escribir header + salt (fijo del vault) + nonce + ciphertext + tag.
- **Lectura/unlock**: validar `magic`/`version` → leer parámetros Argon2 y salt →
  derivar clave → `open()` → parsear JSON → cargar `Vault`.
- **Guardado atómico** (SPEC §8): escribir a `<archivo>.tmp` y `rename` sobre el
  original; ante cualquier error no se toca el archivo previo.
- `magic`/`version` inválidos → error "No es un vault de Cofre"; largos insuficientes
  → error de formato, nunca `panic`.

### 2.6 Pantalla `unlock`: creación y apertura de vault

- **Sin archivo**: sección "crear nuevo vault". Se pide master password (input
  oculto, sin echo) y confirmación; si no coinciden → error local, no se crea nada.
  Regla de mínimo: **8+ caracteres** (aviso, no bloquea). Se genera salt nuevo y se
  guarda el vault inicial.
- **Con archivo**: solo pedir master password; la verificación es el descifrado
  (SPEC §3.2). Error → "Credenciales inválidas o archivo dañado" (no revela cuál).
- Tras **3 intentos fallidos consecutivos** → backoff de **5 s** con cuenta atrás
  visible (SPEC §4.1).
- El vault no encontrado ofrece crear uno nuevo (SPEC §8).

### 2.7 CRUD de entradas (pantallas `list`, `detail`, `form`)

| Operación | Detalle |
|---|---|
| **Crear** | `form`; campos `title*`, `username*`, `password*`, `url`, `notes`, `tags`. Valida que `title` y `username` no estén vacíos. |
| **Listar** | `list` con las entradas reales del vault en memoria; una fila por entry (título + username); navegación y estado vacío "Sin entradas" sin navegar con `Enter`. |
| **Ver** | `detail`; todos los campos, password oculta por defecto con toggle `Tab`/`p`. |
| **Editar** | Mismo `form` pre-rellenado; `Esc` descarta cambios; al guardar se persiste (SPEC §3.5). |
| **Borrar** | Confirmación inline `y`/`n` desde `detail`; irreversible. (La opción de exigir la master password queda para M4.) |

- Los cambios solo persisten en disco tras un guardado explícito exitoso; el estado en
  memoria es la fuente de verdad durante la sesión.

### 2.8 Ciclo de guardado

1. Mutar estado en memoria.
2. `updated_at = now`.
3. Serializar payload → JSON.
4. `seal()` → ciphertext + tag con nuevo nonce.
5. Escritura atómica (`.tmp` + `rename`).

### 2.9 Seguridad de memoria

- Master password, clave derivada y passwords de entries se borran con `zeroize` en
  `drop`/descartes (SPEC §9).
- La payload descifrada vive solo en RAM.
- Al salir de la aplicación (todos los caminos de salida de fase 1) se descartan y
  borran las claves en memoria.

### 2.10 Errores y mensajes

| Caso | Comportamiento |
|---|---|
| Vault no encontrado | Ofrecer crear nuevo vault |
| Contraseña incorrecta / archivo corrupto | "Credenciales inválidas o archivo dañado" |
| `magic`/`version` inválida | "No es un vault de Cofre" |
| Error de disco (permisos, full) | Mensaje claro + no sobreescribir el archivo anterior (guardado atómico) |
| Parseo JSON inválido tras descifrado | Error de formato legible, sin `panic` |

- Nunca `panic` por estas condiciones; se usan tipos `Result` de `errors.rs` y se
  muestran en pantalla sin salir del TUI.

### 2.11 Requisitos no funcionales

- Apertura de vault < **500 ms** (Argon2id incluido), SPEC §9.
- Payload descifrada solo en RAM; `zeroize` en drop.
- Sin red, sin telemetría, sin logs de contraseñas.

## 3. Historias de usuario

### US-1 — Crear un vault

> **Como** usuario,
> **quiero** crear un vault nuevo con mi master password,
> **para** que mis credenciales queden cifradas en disco con un archivo `.cofre`.

**Detalle:** desde `unlock`, sin archivo existente, se pide master password (oculto) y
su confirmación. Si coinciden y cumplen el mínimo, se genera un salt nuevo, se deriva
la clave y se guarda el vault inicial cifrado.

**Criterios de aceptación:**

- [ ] Con el archivo ausente, `unlock` muestra la variante "crear nuevo vault" con
      input oculto y campo de confirmación.
- [ ] Si la confirmación no coincide, se muestra error local y no se crea ningún
      archivo.
- [ ] Una master password de menos de 8 caracteres muestra aviso pero no bloquea.
- [ ] Tras crear, existe un archivo `.cofre` cuyo contenido no es JSON en claro
      (cifrado autenticado).
- [ ] Tras crear, la sesión queda desbloqueada en `list`.

**Edge cases:**

- Cancelar la creación (`Esc`/`q`) → no se crea nada y la terminal queda limpia.
- Intento de crear sobre un archivo ya existente → se abre el vault existente
  (no se sobreescribe sin master password).
- Falla de disco al guardar el vault inicial → mensaje claro, no queda archivo
  parcial (guardado atómico).

### US-2 — Abrir el vault

> **Como** usuario,
> **quiero** abrir mi vault con la master password correcta y que rechace la incorrecta,
> **para** acceder a mis credenciales de forma segura.

**Detalle:** el unlock deriva la clave y descifra; el éxito/fracaso se determina por la
autenticación del ciphertext. Tras 3 intentos fallidos consecutivos hay un backoff de
5 s con cuenta atrás visible.

**Criterios de aceptación:**

- [ ] Con la password correcta, el vault se descifra y se entra a `list` con las
      entradas cargadas.
- [ ] Con la password incorrecta se muestra "Credenciales inválidas o archivo dañado"
      y no se entra a `list`.
- [ ] Tras 3 intentos fallidos consecutivos, el input se bloquea 5 s con cuenta atrás
      visible.
- [ ] La master password y la clave derivada no se almacenan en disco.

**Edge cases:**

- Archivo corrupto / truncado → mismo mensaje que password incorrecta (no se distingue).
- `magic`/`version` inválida → "No es un vault de Cofre".
- Backoff en curso y `Esc`/`q` → se puede salir sin esperar; el intento fallido no
  cuenta como éxito.
- Vault con muchas entradas → apertura < 500 ms.

### US-3 — Crear una entrada

> **Como** usuario,
> **quiero** crear una nueva credencial con título, usuario y contraseña,
> **para** almacenarla cifrada en mi vault.

**Detalle:** `n` desde `list` abre `form`. Se validan `title` y `username` no vacíos.
Al guardar, la entry se añade al estado en memoria y se persiste con el ciclo de
guardado (§2.8).

**Criterios de aceptación:**

- [ ] `n` desde `list` abre `form` vacío.
- [ ] Guardar con `title` o `username` vacíos muestra error y no persiste.
- [ ] Al guardar, la entry aparece en `list` y en el archivo `.cofre` (tras
      reabrir sigue presente).
- [ ] `Esc` en `form` descarta la entrada sin persistir.

**Edge cases:**

- Campo `url`, `notes` o `tags` vacíos → se serializan como string vacío / array vacío
  (SPEC §3.4).
- Guardado con error de disco → la entry queda en memoria pero se informa y no se
  marca como guardada; el archivo previo no se corrompe.

### US-4 — Listar y ver entradas

> **Como** usuario,
> **quiero** ver la lista de mis entradas y el detalle de cada una,
> **para** consultar mis credenciales.

**Detalle:** `list` muestra una fila por entry (título + username) con navegación
`↑/↓`/`j/k`; `Enter` abre `detail` con la password oculta por defecto (toggle `Tab`/`p`).

**Criterios de aceptación:**

- [ ] `list` muestra todas las entradas del vault, una fila por credencial.
- [ ] Con lista vacía se muestra el estado "Sin entradas" y `Enter` no navega.
- [ ] `Enter` sobre una entrada abre `detail` con sus campos.
- [ ] En `detail` la password está oculta por defecto y `Tab`/`p` la muestra/oculta.
- [ ] `Esc` desde `detail` vuelve a `list` conservando la selección.

**Edge cases:**

- Cambios de tamaño de ventana con listas largas → scroll sin `panic` (reutiliza
  fase 1).
- Entry sin URL/notas/tags → se muestran como vacíos, sin errores.

### US-5 — Editar una entrada

> **Como** usuario,
> **quiero** modificar los campos de una entrada existente,
> **para** mantener mis credenciales actualizadas.

**Detalle:** `e` desde `detail` abre `form` pre-rellenado. Al guardar se actualiza la
entry en memoria y se persiste.

**Criterios de aceptación:**

- [ ] `e` desde `detail` abre `form` con los valores actuales.
- [ ] Modificar y guardar actualiza la entry en `list` y en disco.
- [ ] `Esc` descarta los cambios y no modifica nada.
- [ ] La validación de `title`/`username` no vacíos aplica también en edición.

**Edge cases:**

- Editar y guardar con error de disco → los cambios se mantienen en memoria y se
  informa; el archivo previo sigue íntegro.
- Editar una entry mientras se borra el `id` → el `id` no cambia en edición (se
  preserva para integridad de datos).

### US-6 — Borrar una entrada

> **Como** usuario,
> **quiero** eliminar una credencial con confirmación,
> **para** depurar mi vault sin riesgo de borrados accidentales.

**Detalle:** `d` desde `detail` pide confirmación inline `y`/`n`; con `y` se borra la
entry y se persiste. La operación es irreversible.

**Criterios de aceptación:**

- [ ] `d` desde `detail` muestra confirmación inline (`y`/`n`).
- [ ] `n` o `Esc` cancela y la entry permanece.
- [ ] `y` borra la entry de la lista y del disco tras reabrir.
- [ ] Al borrar la última entry, `list` muestra el estado vacío.

**Edge cases:**

- Borrar durante un error de disco → se informa; el archivo no queda truncado.
- Borrado de la entry seleccionada → la selección se re-posiciona sin `panic`.

### US-7 — Integridad y guardado seguro

> **Como** usuario,
> **quiero** que mi vault nunca se corrompa si falla un guardado y que el archivo no
> sea legible sin la master password,
> **para** confiar en que mis datos están a salvo en disco.

**Detalle:** todo guardado es atómico (`.tmp` + `rename`); un fallo de disco deja el
archivo previo intacto. El contenido cifrado no revela los datos en claro.

**Criterios de aceptación:**

- [ ] El archivo `.cofre` contiene header binario + ciphertext; no contiene JSON en
      claro ni la master password.
- [ ] Simular un fallo de escritura en el paso a `.tmp` → el archivo original no se
      modifica.
- [ ] Un `.cofre` con `magic`/`version` inválida produce el error documentado.
- [ ] El guardado tras cada CRUD actualiza `updated_at`.

**Edge cases:**

- Disco lleno o sin permisos → mensaje claro en pantalla, app sigue funcionando con el
  estado en memoria, sin `panic`.
- `rename` falla → se reporta y no se pierde el `.tmp` sin avisar.

### US-8 — Datos sensibles en memoria

> **Como** usuario,
> **quiero** que mis claves y contraseñas se borren de memoria al descartarse,
> **para** minimizar la exposición de credenciales en el proceso.

**Detalle:** master password, clave derivada y passwords de entries se limpian con
`zeroize` al salir del flujo que los usó o al terminar la sesión.

**Criterios de aceptación:**

- [ ] La master password y la clave derivada se limpian al salir de la app por
      cualquier camino.
- [ ] Las passwords de entries se limpian al descartar la entrada en memoria.
- [ ] No hay logs ni volcados con contraseñas en claro.

**Edge cases:**

- Salida por `panic` → los `drop` de tipos con `zeroize` siguen ejecutándose.
- Cambio de pantalla (p. ej. de `unlock` a `list`) → el buffer del input de la master
  password se limpia al completar el unlock.

## 4. Definition of Done (DoD)

La fase 2 se considera terminada cuando:

- [ ] Se puede crear un vault nuevo, abrirlo con la password correcta y rechazar la
      incorrecta (incluido el backoff de 3 intentos).
- [ ] El archivo `.cofre` no puede leerse sin la master password (cifrado autenticado).
- [ ] CRUD completo (crear, listar, ver, editar, borrar con confirmación) persiste
      correctamente en disco.
- [ ] El guardado es atómico: un fallo de disco no corrompe el vault.
- [ ] Los errores del SPEC §8 muestran mensajes claros sin `panic`.
- [ ] `zeroize` aplica a master password, clave derivada y passwords.
- [ ] `cargo fmt --check` pasa sin diferencias.
- [ ] `cargo clippy -- -D warnings` no reporta `warnings`.
- [ ] `cargo test` pasa, incluyendo unit tests de: crypto round-trip,
      discriminación de clave errónea, formato binario con fixtures
      `tests/fixtures/*.cofre`, validación y lógica CRUD, y guardado atómico.
- [ ] No se implementa funcionalidad fuera de alcance (§1.2): nada de generador,
      búsqueda/filtros, portapapeles, auto-lock ni settings funcionales.
- [ ] La máquina de pantallas de fase 1 se conserva; las transiciones existentes
      siguen siendo válidas.

## 5. Fase siguiente (preview, fuera de esta entrega)

Tras validar crypto, storage y CRUD, la siguiente fase corresponde a **M3 — UX** del
PRD: generador de contraseñas, búsqueda/filtros, portapapeles con auto-clear y
auto-lock por inactividad, según `docs/SPEC.md` §4.3–4.6.

## 6. Nota sobre tareas

Este documento define **qué** se desarrolla (requerimientos, historias de usuario con
criterios de aceptación y edge cases) y **cuándo está terminado** (DoD). El desglose
en **tareas de implementación** (epics, subtareas, estimaciones y orden de trabajo) se
detallará en un documento de planificación posterior.