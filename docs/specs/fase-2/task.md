# TASK-FASE-2 — Tareas de desarrollo: Cifrado y almacenamiento (Crypto & storage)

> Desglose atómico de las tareas de implementación de la fase 2 de **Cofre**.
> Derivado de `docs/PRD.md` (M2 Crypto & storage), `docs/SPEC.md` y
> `docs/specs/fase-2/DEV-FASE-2.md`.
> Estado: **planificado**.

## 1. Propósito y convenciones

Este documento descompone la fase 2 en **tareas atómicas agrupadas por concepto**.
Cada tarea incluye:

- **Descripción**: qué se implementa y en qué módulos (`src/…`).
- **Tests a crear**: casos que deben cubrirse.
- **Casos cubiertos**: comportamiento normal, errores, criterios de aceptación (según
  las historias de usuario de `DEV-FASE-2.md`) y edge cases.

Convenciones:

- **Naming de tests**: `mod test_<modulo>`, casos `fn <comportamiento>_<condición>`.
- **Sin `panic`**: el código de la app no debe hacer `panic` en ninguna condición
  controlable; los tests deben verificar ausencia de `panic` en estados límite.
- **Testabilidad**: la serialización JSON, la crypto (derive/seal/open), el formato
  binario, el guardado y la lógica de CRUD/unlock se implementan como **funciones
  puras** (`models.rs`, `crypto.rs`, `storage.rs`, `app.rs`) para poder testearlas sin
  TTY. El UI solo dibuja; la decisión está en `app.rs`.
- **DoD por tarea**: `cargo fmt --check` sin diferencias, `cargo clippy -- -D warnings`
  sin warnings y `cargo test` verde.
- **Fuera de alcance**: ninguna tarea incluye generador, búsqueda/filtros,
  portapapeles, auto-lock ni settings funcionales (sección §1.2 de `DEV-FASE-2.md`).
- La máquina de pantallas de fase 1 se **conserva**: las tareas de esta fase integran
  estado/lógica en `app.rs` y `ui/` sin cambiar las transiciones existentes.

## 2. Concepto 0 — Proyecto y dependencias

### T-00-01 — Añadir dependencias de crypto, modelos y storage

**Descripción**

- Añadir a `Cargo.toml` (se suman a `crossterm` y `ratatui` de fase 1):
  - `argon2` (KDF Argon2id).
  - `chacha20poly1305` (cifrado autenticado XChaCha20-Poly1305).
  - `rand` (`OsRng` para salt, nonce e ids).
  - `zeroize` (borrado seguro en drop).
  - `serde` + `serde_json` (serialización de la payload).
  - `uuid` (ids uuid-v4 de entradas).
  - `time` (timestamp ISO 8601 de `updated_at`).
- Versiones estables actuales, fijadas en el `Cargo.lock`; compatibles entre sí y con
  las de fase 1. Sin dependencias de red (todo local).

**Tests a crear**

- Check estático: `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo build`.

**Casos cubiertos**

- Normal: compilación limpia con la nueva dependencia.
- Criterio de aceptación (DoD): no se incluyen dependencias fuera de la lista
  (sin generador/clipboard/etc.).
- Edge: versiones compatibles con `ratatui`/`crossterm` de fase 1.

---

### T-00-02 — Estructura de módulos nuevos y fixtures

**Descripción**

- Crear los módulos de negocio del SPEC §1 y de §2.2 de `DEV-FASE-2.md`:
  - `src/models.rs` (Entry, Vault, Settings).
  - `src/crypto.rs` (Argon2id + XChaCha20-Poly1305 + formato).
  - `src/storage.rs` (lectura/escritura del archivo `.cofre`).
- Extender `src/errors.rs` con las variantes de crypto/storage (ver Concepto B).
- Crear el directorio `tests/fixtures/` para los archivos `.cofre` de prueba
  (rellenado en T-STO-04).
- Los módulos nuevos deben compilar como stubs declarados (sin lógica todavía).

**Tests a crear**

- Check estático: `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo build`.

**Casos cubiertos**

- Normal: compilación limpia con la estructura de módulos declarada.
- Criterio de aceptación (DoD): separación de responsabilidades (§2.2) sin acoplar
  crypto a UI.
- Edge: los stubs no rompen las transiciones de fase 1.

---

## 3. Concepto A — Modelos de datos (`models.rs`)

### T-MOD-01 — Tipos `Entry`, `Settings` y `Vault`

**Descripción**

- `src/models.rs`:
  - `Entry { id: Uuid, title: String, username: String, password: String, url: String, notes: String, tags: Vec<String> }`.
  - `Settings { auto_lock_minutes: u32, clipboard_seconds: u32 }` con defaults `5`/`15`
    (se serializan en esta fase aunque la pantalla de ajustes no sea funcional hasta M4).
  - `Vault { version: u8, updated_at: OffsetDateTime, settings: Settings, entries: Vec<Entry> }`.
- Derivar `Serialize`/`Deserialize`, `Clone`, `PartialEq`, `Debug`.

**Tests a crear**

- `mod test_models`:
  - `fn settings_defaults_correctos()` → `5` y `15`.
  - `fn entry_con_campos_obligatorios()` → se crea con los campos mínimos.
  - `fn vault_version_inicial_es_uno()`
  - `fn igualdad_por_valor_para_tests()` → `PartialEq` permite comparar en tests.

**Casos cubiertos**

- Normal: los tipos modelan el SPEC §3.4.
- Criterio de aceptación (DoD): `url`/`notes`/`tags` pueden ir vacíos sin romper.
- Edge: creación de `Vault` sin entries (estado vacío válido).

---

### T-MOD-02 — Serialización JSON de la payload (SPEC §3.4)

**Descripción**

- `src/models.rs`: `serialize_payload(&Vault) -> Result<String>` y
  `deserialize_payload(&str) -> Result<Vault>`.
- Reglas de serialización:
  - `version: 1`.
  - `updated_at` en formato ISO 8601.
  - `url`/`notes` como string vacío si no existen.
  - `tags` como array (puede ir vacío).
- Round-trip consistente (serializar → deserializar → misma payload).

**Tests a crear**

- `mod test_payload`:
  - `fn round_trip_consistente()`
  - `fn campos_opcionales_serializan_vacios()`
  - `fn tags_vacio_serializa_array()`
  - `fn version_serializa_como_uno()`
  - `fn json_invalido_devuelve_error()` → sin `panic`.

**Casos cubiertos**

- Normal: el JSON del disco coincide con el SPEC §3.4.
- Criterio de aceptación (US-3, US-7): los datos persisten y se recuperan igual tras
  reabrir.
- Edge: payload malformada → error de formato legible, sin `panic`.

---

## 4. Concepto B — Tipos de error extendidos

### T-ERR-01 — Variantes de error de crypto/storage

**Descripción**

- `src/errors.rs`: añadir a `CofreError` las variantes:
  - `NotACofre` → "No es un vault de Cofre".
  - `InvalidCredentialsOrCorrupt` → "Credenciales inválidas o archivo dañado".
  - `DiskError(String)` → error de I/O con causa legible.
  - `FormatError` → archivo con largo insuficiente o campos inválidos.
  - `JsonParseError` → payload descifrada no parsea.
- `Display` de cada variante en español, sin revelar secretos (SPEC §8).

**Tests a crear**

- `mod test_errors_fase2`:
  - `fn display_not_a_cofre_mensaje_legible()`
  - `fn display_invalid_credentials_mensaje_ambiguo()` → no distingue contraseña de
    corrupción.
  - `fn display_disk_error_incluye_causa()`
  - `fn display_no_revela_secretos()` → el mensaje no contiene la password.

**Casos cubiertos**

- Normal: cada variante produce el mensaje definido (SPEC §8).
- Criterio de aceptación (DoD): mensajes claros sin `panic`.
- Edge: mensajes sin fugas de datos sensibles.

---

### T-ERR-02 — Manejo sin `panic` y mensajes de usuario

**Descripción**

- Función pura `fn user_message(error: &CofreError) -> String` para mostrar el error en
  la pantalla activa sin salir del TUI.
- Los errores de archivo/disco se muestran en pantalla; la app sigue funcionando con el
  estado en memoria.
- Ningún camino de error hace `panic`; se usan `Result` en toda la pila.

**Tests a crear**

- `mod test_error_handling`:
  - `fn error_se_muestra_y_no_panics()` → cada variante produce un mensaje y no pánico.
  - `fn error_de_disco_no_sale_del_tui()` → el mapeo devuelve mensaje sin `Quit`.
  - `fn error_no_revela_password_ni_clave()` → el mensaje de usuario nunca contiene
    secretos.

**Casos cubiertos**

- Criterio de aceptación (US-7, US-8, DoD): errores claros, sin `panic`, sin secretos.
- Edge: error de disco en medio de un guardado → mensaje en pantalla, archivo intacto.

---

## 5. Concepto C — Crypto (`crypto.rs`)

### T-CRY-01 — Derivación de clave Argon2id

**Descripción**

- `src/crypto.rs`:
  - `derive_key(master_password: &str, salt: &[u8], params: &Argon2Params) -> [u8; 32]`
    con Argon2id.
  - Parámetros por defecto del SPEC §3.1: `m_cost = 19456 KiB`, `t_cost = 2`,
    `p_cost = 1`; salt de 16 bytes.
- `Argon2Params` se construye desde los valores del header (almacenados por vault).

**Tests a crear**

- `mod test_derive_key`:
  - `fn misma_password_mismo_salt_misma_clave()` → determinista.
  - `fn salt_distinto_clave_distinta()`
  - `fn password_distinta_clave_distinta()`
  - `fn clave_derivada_es_de_32_bytes()`
  - `fn parametros_default_coinciden_spec()` → m=19456, t=2, p=1.

**Casos cubiertos**

- Normal: la clave se deriva correctamente para abrir el vault.
- Criterio de aceptación (US-1, US-2): la clave vive solo en memoria; nunca a disco.
- Edge: salt de largo distinto de 16 → se valida o se documenta el comportamiento.

---

### T-CRY-02 — Cifrado autenticado `seal`/`open`

**Descripción**

- `src/crypto.rs`:
  - `seal(plaintext: &[u8], key: &[u8; 32]) -> (nonce: [u8; 24], ciphertext_with_tag: Vec<u8>)`
    con XChaCha20-Poly1305.
  - `open(nonce: &[u8; 24], ciphertext_with_tag: &[u8], key: &[u8; 32]) -> Result<Vec<u8>>`.
- El nonce es **nuevo por operación** (24 bytes, `OsRng`); el tag (16 B) va anexado al
  ciphertext (SPEC §3.2).

**Tests a crear**

- `mod test_seal_open`:
  - `fn round_trip_recupera_plaintext()`
  - `fn nonce_nuevo_por_operacion()` → dos `seal` del mismo plaintext producen nonces
    distintos.
  - `fn ciphertext_difiere_con_nonce_distinto()`
  - `fn tag_anexado_es_de_16_bytes()`

**Casos cubiertos**

- Normal: cifrado y descifrado autenticado correctos.
- Criterio de aceptación (US-2, DoD): el fallo de autenticación rechaza el dato.
- Edge: ciphertext vacío o de largo inválido → error, sin `panic`.

---

### T-CRY-03 — Discriminación de clave errónea

**Descripción**

- `open` con clave incorrecta debe fallar por **autenticación**.
- El error resultante se mapea a `InvalidCredentialsOrCorrupt`: no se distingue entre
  "contraseña incorrecta" y "archivo corrupto" (SPEC §3.2).

**Tests a crear**

- `mod test_wrong_key`:
  - `fn clave_erronea_falla_open()`
  - `fn archivo_alterado_falla_open()` → modificar un byte del ciphertext rompe la
    autenticación.
  - `fn clave_erronea_y_archivo_corrupto_mismo_error()` → misma variante
    `InvalidCredentialsOrCorrupt`.

**Casos cubiertos**

- Criterio de aceptación (US-2, US-7): no se revela cuál de los dos casos falló.
- Edge: cifrado con una clave y apertura con otra → siempre error de autenticación.

---

## 6. Concepto D — Formato `.cofre` y storage (`storage.rs`)

### T-STO-01 — Escritura del archivo `.cofre` v1

**Descripción**

- `src/storage.rs`: `write_vault(path, &Vault, master_password, salt) -> Result<()>`
  (o función equivalente que reciba los datos cifrados).
- Layout binario del SPEC §3.3, enteros big-endian:
  - Magic `"COFRE1"` (7 B) + version `u8 = 1`.
  - `m_cost`, `t_cost`, `p_cost` (u32 cada uno).
  - Salt (16 B, fijo del vault).
  - Nonce (24 B) + ciphertext + tag Poly1305 (16 B).
- El archivo **no contiene JSON en claro** ni la master password.

**Tests a crear**

- `mod test_write_vault`:
  - `fn archivo_comienza_con_magic()`
  - `fn version_escrita_es_uno()`
  - `fn campos_numericos_big_endian()` → offsets 8..20 con los u32 en BE.
  - `fn no_hay_json_en_claro()` → el contenido no contiene `"entries"` legible.
  - `fn salt_se_conserva_en_header()`

**Casos cubiertos**

- Normal: el archivo resultante cumple el layout del SPEC §3.3.
- Criterio de aceptación (US-7, DoD): cifrado autenticado en reposo.
- Edge: paths con directorio inexistente → error de disco, sin `panic`.

---

### T-STO-02 — Lectura y validación del archivo

**Descripción**

- `src/storage.rs`: `open_vault(path, master_password) -> Result<Vault>` (o función
  equivalente):
  1. Leer archivo → validar `magic`/`version`.
  2. Leer parámetros Argon2 y salt.
  3. Derivar clave.
  4. `open()` → si falla la autenticación → `InvalidCredentialsOrCorrupt`.
  5. Parsear JSON → cargar `Vault`.
- Largos insuficientes → `FormatError`, nunca `panic` (SPEC §3.3).

**Tests a crear**

- `mod test_read_vault`:
  - `fn round_trip_escritura_lectura()`
  - `fn magic_invalida_devuelve_not_a_cofre()`
  - `fn version_invalida_devuelve_not_a_cofre()`
  - `fn largo_insuficiente_devuelve_format_error()`
  - `fn json_invalido_tras_descifrado_devuelve_json_parse_error()`
  - `fn password_incorrecta_devuelve_invalid_credentials()`

**Casos cubiertos**

- Normal: apertura completa del flujo del SPEC §3.6.
- Criterio de aceptación (US-2, DoD): verificación por autenticación; errores claros.
- Edge: archivo truncado/corrupto → mismo mensaje que password incorrecta.

---

### T-STO-03 — Guardado atómico (`.tmp` + `rename`)

**Descripción**

- Todo guardado escribe primero a `<archivo>.tmp` y luego `rename` sobre el original
  (SPEC §8).
- Si falla la escritura del `.tmp` (disco lleno, permisos) o el `rename`:
  - El archivo original **no se modifica**.
  - Se reporta el error; el `.tmp` no queda sin avisar (limpieza o reporte).

**Tests a crear**

- `mod test_atomic_save`:
  - `fn fallo_escribiendo_tmp_no_toca_original()` → simular error de escritura.
  - `fn rename_exitoso_reemplaza_archivo()`
  - `fn fallo_rename_reporta_y_conserva_original()`
  - `fn tmp_no_queda_sin_reporte()` → o se limpia o se informa.

**Casos cubiertos**

- Criterio de aceptación (US-7, DoD): fallo de disco no corrompe el vault.
- Edge: disco lleno / sin permisos → mensaje claro, app sigue en memoria.

---

### T-STO-04 — Fixtures de prueba (`tests/fixtures/`)

**Descripción**

- Crear fixtures binarias `tests/fixtures/*.cofre` generadas por el propio código de
  fase 2 (un fixture se crea en el test y se commitea como dato conocido):
  - `fixture-valid.cofre`: vault con 1–2 entries y una password conocida.
  - `fixture-invalid-magic.cofre`: magic inválida.
  - `fixture-truncated.cofre`: largo insuficiente.
- Los tests de integración abren el fixture con la password conocida y rechazan la
  incorrecta.

**Tests a crear**

- `mod test_fixtures` (tests de integración):
  - `fn abrir_fixture_con_password_conocida()`
  - `fn rechazar_fixture_con_password_incorrecta()`
  - `fn fixture_magic_invalida_error()`
  - `fn fixture_truncada_error_de_formato()`

**Casos cubiertos**

- Criterio de aceptación (DoD): formato binario testeado con fixtures.
- Edge: fixtures versionadas → cambios de formato futuros rompen tests
  (migraciones, fuera de alcance).

---

## 7. Concepto E — Pantalla `unlock`: creación y apertura de vault

### T-UNL-01 — Crear un vault nuevo

**Descripción**

- `src/app.rs` + `src/ui/screens.rs`: variante "crear nuevo vault" en `unlock` cuando no
  existe archivo.
- Flujo: master password (input oculto, sin echo) + confirmación.
  - Si no coinciden → error local, no se crea nada.
  - Regla de mínimo **8+ caracteres**: aviso, no bloquea.
- Al confirmar: generar salt nuevo (`OsRng`), derivar clave, guardar el vault inicial
  cifrado (T-STO-01) y transición a `list` (sesión desbloqueada).
- `Esc`/`q` cancela sin crear nada.

**Tests a crear**

- `mod test_create_vault`:
  - `fn variante_crear_se_muestra_sin_archivo()`
  - `fn confirmacion_no_coincide_error_local()`
  - `fn confirmacion_no_coincide_no_crea_archivo()`
  - `fn menor_de_8_avisa_no_bloquea()`
  - `fn crear_exitoso_queda_desbloqueado_en_list()`
  - `fn esc_cancela_sin_crear_archivo()`

**Casos cubiertos**

- Criterio de aceptación (US-1, DoD): creación completa con confirmación y salto a `list`.
- Edge: falla de disco al guardar el vault inicial → mensaje claro, sin archivo parcial.
- Edge: existe archivo → se abre el vault existente (no se sobreescribe).

---

### T-UNL-02 — Apertura con master password (unlock)

**Descripción**

- `src/app.rs`: al confirmar en `unlock` con archivo existente, se ejecuta
  `open_vault(path, master_password)` (T-STO-02).
- Éxito → `list` con las entries cargadas en `AppState`.
- Fracaso → mensaje `InvalidCredentialsOrCorrupt` en pantalla, se permanece en `unlock`.

**Tests a crear**

- `mod test_unlock`:
  - `fn password_correcta_entra_a_list_con_entries()`
  - `fn password_incorrecta_permanece_en_unlock()`
  - `fn password_incorrecta_muestra_mensaje_ambiguo()`
  - `fn archivo_corrupto_mismo_mensaje()`
  - `fn magic_invalida_mensaje_not_a_cofre()`
  - `fn master_password_no_se_almacena_en_disco()`

**Casos cubiertos**

- Criterio de aceptación (US-2, DoD): apertura correcta/rechazo sin distinguir causa.
- Edge: vault con muchas entries → apertura < 500 ms.

---

### T-UNL-03 — Backoff de intentos fallidos

**Descripción**

- `src/app.rs`: contador de **3 intentos fallidos consecutivos** → bloqueo del input
  con **backoff de 5 s** y cuenta atrás visible en `unlock`.
- `Esc`/`q` durante el backoff permite salir sin esperar.
- Un intento exitoso resetea el contador.

**Tests a crear**

- `mod test_backoff`:
  - `fn tras_3_fallidos_consecutivos_activa_backoff()`
  - `fn cuenta_atras_visible_en_estado()` → el estado del backoff expone el tiempo
    restante.
  - `fn esc_sale_durante_backoff_sin_esperar()`
  - `fn intento_exitoso_resetea_contador()`
  - `fn fallido_durante_backoff_no_cuenta_como_tercero()`

**Casos cubiertos**

- Criterio de aceptación (US-2, DoD): backoff de 5 s tras 3 fallidos con cuenta atrás.
- Edge: el intento fallido durante el backoff no se suma.

---

## 8. Concepto F — CRUD de entradas

### T-CRUD-01 — Crear una entrada (`form`)

**Descripción**

- `n` desde `list` abre `form` vacío (transición ya existente de fase 1).
- `src/app.rs`: al guardar, validar que `title` y `username` no estén vacíos; en caso
  contrario error y no persiste.
- `url`, `notes`, `tags` opcionales (se serializan vacíos si no existen).
- Al guardar: añadir entry (id nuevo uuid-v4) al estado y persistir (T-CRUD-05).
- `Esc` descarta sin persistir.

**Tests a crear**

- `mod test_crud_create`:
  - `fn n_desde_list_abre_form_vacio()`
  - `fn title_vacio_error_y_no_persiste()`
  - `fn username_vacio_error_y_no_persiste()`
  - `fn campos_opcionales_vacios_ok()`
  - `fn guardado_anade_la_entry_a_list()`
  - `fn esc_descarta_sin_persistir()`
  - `fn id_generado_es_uuid_v4()`

**Casos cubiertos**

- Criterio de aceptación (US-3, DoD): creación validada y persistida.
- Edge: error de disco al guardar → entry en memoria, se informa, archivo intacto.

---

### T-CRUD-02 — Listar y ver entradas

**Descripción**

- `list` muestra las entries reales del vault (una fila por entry: título + username),
  reemplazando las ficticias de fase 1.
- `Enter` sobre una entry abre `detail` con sus campos.
- En `detail`, la password está oculta por defecto; `Tab`/`p` la muestra/oculta.
- Lista vacía → estado "Sin entradas", `Enter` no navega.

**Tests a crear**

- `mod test_crud_list_detail`:
  - `fn list_muestra_entries_reales()`
  - `fn enter_abre_detail_con_campos()`
  - `fn password_oculta_por_defecto()`
  - `fn toggle_tab_o_p_muestra_password()`
  - `fn lista_vacia_mensaje_y_no_navega()`
  - `fn entry_sin_opcionales_se_muestra_vacia()`

**Casos cubiertos**

- Criterio de aceptación (US-4, DoD): listado y detalle funcionales.
- Edge: listas largas con ventanas pequeñas → scroll sin `panic` (fase 1).

---

### T-CRUD-03 — Editar una entrada

**Descripción**

- `e` desde `detail` abre `form` **pre-rellenado** con los valores actuales.
- Al guardar: actualizar la entry en memoria y persistir.
- El `id` **no cambia** en edición (se preserva para integridad).
- `Esc` descarta los cambios sin modificar nada.
- La validación de `title`/`username` no vacíos aplica también en edición.

**Tests a crear**

- `mod test_crud_edit`:
  - `fn e_desde_detail_abre_form_prerellenado()`
  - `fn guardar_actualiza_la_entry_en_list()`
  - `fn id_se_preserva_en_edicion()`
  - `fn esc_descarta_sin_modificar()`
  - `fn validacion_aplica_en_edicion()`

**Casos cubiertos**

- Criterio de aceptación (US-5, DoD): edición con datos preservados.
- Edge: error de disco al guardar → cambios en memoria, archivo íntegro.

---

### T-CRUD-04 — Borrar una entrada

**Descripción**

- `d` desde `detail` pide confirmación inline (`y`/`n`).
  - `n` o `Esc` cancela; la entry permanece.
  - `y` borra la entry, persiste y vuelve a `list`.
- Al borrar la entry seleccionada, la selección se re-posiciona sin `panic`.
- Al borrar la última entry, `list` muestra el estado vacío.

**Tests a crear**

- `mod test_crud_delete`:
  - `fn d_desde_detail_muestra_confirmacion_inline()`
  - `fn n_o_esc_cancela_y_la_entry_permanece()`
  - `fn y_borra_y_persiste()`
  - `fn seleccion_se_reposiciona_sin_panic()`
  - `fn ultima_entry_deja_estado_vacio()`

**Casos cubiertos**

- Criterio de aceptación (US-6, DoD): borrado confirmado e irreversible.
- Edge: error de disco al guardar el borrado → se informa; el archivo no queda truncado.

---

### T-CRUD-05 — Ciclo de guardado y `updated_at`

**Descripción**

- Centralizar la persistencia tras cada CRUD en una única función de guardado
  (ciclo del SPEC §3.5):
  1. Mutar estado en memoria.
  2. `updated_at = now`.
  3. Serializar payload → JSON.
  4. `seal()` → ciphertext + tag con nuevo nonce.
  5. Escritura atómica (`.tmp` + `rename`).
- `updated_at` se actualiza en cada guardado exitoso.

**Tests a crear**

- `mod test_save_flow`:
  - `fn updated_at_se_actualiza_al_guardar()`
  - `fn guardado_tras_cada_operacion_crud()`
  - `fn nonce_nuevo_por_guardado()` → dos guardados del mismo vault producen ciphertext
    distinto.
  - `fn fallo_de_disco_conserva_memoria_y_archivo()`

**Casos cubiertos**

- Criterio de aceptación (US-7, DoD): el guardado tras cada CRUD actualiza `updated_at`
  y usa el ciclo completo.
- Edge: fallo de disco → memoria conserva el cambio, archivo íntegro.

---

## 9. Concepto G — Seguridad de memoria

### T-SEC-01 — `zeroize` de master password y clave derivada

**Descripción**

- La master password y la clave derivada se limpian con `zeroize` al:
  - Completar el unlock (paso de `unlock` a `list`).
  - Salir de la aplicación por cualquier camino (reutiliza la centralización de
    teardown de fase 1).
- La clave derivada vive solo en memoria durante la sesión; nunca en disco.

**Tests a crear**

- `mod test_zeroize_keys`:
  - `fn master_password_limpia_tras_unlock()`
  - `fn clave_derivada_limpia_al_salir()`
  - `fn clave_no_se_escribe_en_disco()` → el archivo `.cofre` no contiene la clave.

**Casos cubiertos**

- Criterio de aceptación (US-8, DoD): claves limpias al salir de la app por cualquier
  camino.
- Edge: salida por `panic` → los `drop` de tipos `zeroize` siguen ejecutándose.

---

### T-SEC-02 — `zeroize` de passwords de entries

**Descripción**

- Las passwords de las entries se limpian con `zeroize` al descartar la entrada en
  memoria (drop).
- No hay logs ni volcados con contraseñas en claro.

**Tests a crear**

- `mod test_zeroize_passwords`:
  - `fn password_limpia_al_descartar_entry()`
  - `fn sin_logs_en_claro()` → los tipos de error/mensajes no incluyen passwords.

**Casos cubiertos**

- Criterio de aceptación (US-8, DoD): passwords limpias al descartar; sin secretos en
  mensajes.
- Edge: entrada borrada → su password no queda en memoria.

---

## 10. Orden de ejecución sugerido

| # | Tarea | Depende de |
|---|---|---|
| 1 | T-00-01, T-00-02 | — |
| 2 | T-MOD-01, T-MOD-02 | T-00-01 |
| 3 | T-ERR-01, T-ERR-02 | T-00-02 |
| 4 | T-CRY-01, T-CRY-02, T-CRY-03 | T-ERR-01 |
| 5 | T-STO-01, T-STO-02, T-STO-03 | T-CRY-01..03, T-MOD-02, T-ERR-01 |
| 6 | T-STO-04 | T-STO-01, T-STO-02 |
| 7 | T-UNL-01, T-UNL-02, T-UNL-03 | T-STO-01, T-STO-02, T-STO-03 |
| 8 | T-CRUD-01, T-CRUD-02, T-CRUD-03, T-CRUD-04, T-CRUD-05 | T-UNL-02, T-UNL-03 |
| 9 | T-SEC-01, T-SEC-02 | T-UNL-02, T-CRUD-05 |

Racional: primero se fijan dependencias y tipos (Conceptos 0 y A), luego los errores
(B) que toda la capa crypto/storage usará. La crypto (C) es base del formato (D), y el
formato es base de unlock (E) y CRUD (F). La seguridad de memoria (G) se cierra al
final sobre los flujos ya integrados, sin fijar detalles de `zeroize` prematuramente.

## 11. Mapa con el Definition of Done de `DEV-FASE-2.md`

| DoD (§4) | Tareas que lo satisfacen |
|---|---|
| Crear vault, abrir con password correcta y rechazar la incorrecta (backoff 3 intentos) | T-UNL-01, T-UNL-02, T-UNL-03 |
| El `.cofre` no puede leerse sin la master password | T-CRY-01, T-CRY-02, T-CRY-03, T-STO-01, T-STO-02 |
| CRUD completo persiste en disco | T-CRUD-01..05, T-STO-03 |
| Guardado atómico: fallo de disco no corrompe el vault | T-STO-03, T-CRUD-05 |
| Errores del SPEC §8 claros sin `panic` | T-ERR-01, T-ERR-02, T-STO-02 |
| `zeroize` de master password, clave derivada y passwords | T-SEC-01, T-SEC-02 |
| `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test` en verde | todos (DoD por tarea), explícito T-00-01 |
| Tests: crypto round-trip, clave errónea, fixtures `.cofre`, CRUD, atómico | T-CRY-01..03, T-STO-04, T-CRUD-01..05, T-STO-03 |
| Sin funcionalidad fuera de alcance (§1.2) | T-00-01, T-00-02 |
| Máquina de pantallas de fase 1 conservada | T-00-02, T-UNL-01, T-CRUD-01..04 |