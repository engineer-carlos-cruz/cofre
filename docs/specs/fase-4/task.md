# TASK-FASE-4 — Tareas de desarrollo: Polish (M4)

> Desglose atómico de las tareas de implementación de la fase 4 de **Cofre**.
> Derivado de `docs/PRD.md` (M4 Polish), `docs/SPEC.md` y
> `docs/specs/fase-4/DEV-FASE-4.md`.
> Estado: **planificado**.

## 1. Propósito y convenciones

Este documento descompone la fase 4 en **tareas atómicas agrupadas por concepto**.
Cada tarea incluye:

- **Descripción**: qué se implementa y en qué módulos (`src/…`).
- **Tests a crear**: casos que deben cubrirse.
- **Casos cubiertos**: comportamiento normal, errores, criterios de aceptación (según
  las historias de usuario de `DEV-FASE-4.md`) y edge cases.

Convenciones:

- **Naming de tests**: `mod test_<modulo>`, casos `fn <comportamiento>_<condición>`.
- **Sin `panic`**: el código de la app no debe hacer `panic` en ninguna condición
  controlable; los tests deben verificar ausencia de `panic` en estados límite.
- **Testabilidad**: los defaults, la validación de rangos, la verificación de master
  password, la lógica de re-salt/re-cifrado y el borrado protegido se implementan como
  **funciones puras** (`config.rs`, `crypto.rs`, `app.rs`) para poder testearlas sin
  TTY. El UI solo dibuja; la decisión está en `app.rs`.
- **DoD por tarea**: `cargo fmt --check` sin diferencias, `cargo clippy -- -D warnings`
  sin warnings y `cargo test` verde.
- **Fuera de alcance**: ninguna tarea incluye extensiones futuras del PRD
  (detección de contraseñas débiles/reutilizadas, import/export, TOTP/2FA, backup,
  FIDO2, recovery kit, multi-vault) ni fuzz (sección §1.2 de `DEV-FASE-4.md`).
- La máquina de pantallas de fases 1–3 se **conserva**: las tareas de esta fase
  integran estado/lógica en `app.rs` y `ui/` sin cambiar las transiciones existentes.

## 2. Concepto 0 — Proyecto y estructura

### T-00-01 — Extender `Settings` y crear `config.rs` (sin dependencias nuevas)

**Descripción**

- **No** se añade ninguna dependencia nueva: se reutilizan las de fases 1–3
  (`crossterm`, `ratatui`, `argon2`, `chacha20poly1305`, `rand`, `zeroize`,
  `serde`/`serde_json`, `uuid`, `time`, `arboard`). Verificar en `Cargo.toml`.
- `src/models.rs`: extender `Settings` con `require_password_on_delete: bool`
  (default `false`), manteniendo `auto_lock_minutes` (default 5) y
  `clipboard_seconds` (default 15) (§2.2–2.3 de `DEV-FASE-4.md`).
- Crear `src/config.rs` como stub declarado (sin lógica todavía), home de defaults y
  validación de rangos (SPEC §1).
- Crear `src/tests/integration.rs` (o `tests/integration.rs`) como stub para los
  flujos completos (rellenado en T-INT-01).

**Tests a crear**

- Check estático: `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo build`.
- `mod test_settings_extended`:
  - `fn settings_defaults_incluyen_require_password_on_delete_false()`
  - `fn settings_existentes_siguen_serializando()` → el JSON del SPEC §3.4 sigue
    válido con el campo nuevo opcional.

**Casos cubiertos**

- Normal: compilación limpia con la estructura declarada y sin dependencias nuevas.
- Criterio de aceptación (DoD): no se incluyen dependencias fuera de la lista ni
  funcionalidad de extensiones futuras (§1.2).
- Edge: payloads `.cofre` de fases anteriores sin el campo nuevo se deserializan con
  default `false` (retrocompatibilidad de lectura).

---

### T-00-02 — Estado de settings en `AppState` y transición a `settings`

**Descripción**

- `src/app.rs`: extender `AppState` con el estado de la pantalla `settings`:
  - `settings_draft: SettingsDraft` (copias editables de los 3 campos).
  - `settings_dirty: bool` (hay cambios sin guardar).
  - `message` reutilizado para errores de validación/guardado.
- Transición: `t` desde `list` abre `settings` (atajo sugerido en DEV-FASE-1 §2.5),
  cargando el draft desde `Settings` de la sesión; `Esc`/`q` vuelve a `list`
  conservando la selección (transición de fase 1).
- La máquina de pantallas de fases 1–3 no cambia.

**Tests a crear**

- `mod test_settings_state`:
  - `fn t_desde_list_abre_settings_con_draft_actual()`
  - `fn settings_sin_cambios_no_dirty()`
  - `fn esc_desde_settings_vuelve_a_list_con_seleccion()`
  - `fn t_en_pantalla_incorrecta_se_ignora()`

**Casos cubiertos**

- Criterio de aceptación (US-1, DoD): `t` abre `settings` con los valores actuales y
  `Esc` descarta sin persistir (enlazado a T-SET-01).
- Edge: atajos presionados en pantallas no permitidas → ignorados sin ruido.

---

## 3. Concepto A — Configuración (`config.rs`)

### T-CFG-01 — Defaults y validación de rangos

**Descripción**

- `src/config.rs` (**lógica pura**, sin TTY):
  - Constantes de defaults: `DEFAULT_AUTO_LOCK_MINUTES = 5`,
    `DEFAULT_CLIPBOARD_SECONDS = 15`, `DEFAULT_REQUIRE_PASSWORD_ON_DELETE = false`.
  - `fn validate_auto_lock_minutes(value: u32) -> Result<()>`: rango **1–120**
    (SPEC §4.8); fuera → error de validación legible.
  - `fn validate_clipboard_seconds(value: u32) -> Result<()>`: rango **5–120**.
  - `fn validate_require_password_on_delete(value: bool) -> Result<()>`: siempre
    válido (toggle).
  - `fn default_settings() -> Settings` con los defaults.
- Los errores de validación se mapean a `CofreError` (sin `panic`).

**Tests a crear**

- `mod test_config`:
  - `fn defaults_coinciden_spec()`
  - `fn auto_lock_1_y_120_ok()`
  - `fn auto_lock_0_y_121_error()`
  - `fn clipboard_5_y_120_ok()`
  - `fn clipboard_4_y_130_error()`
  - `fn require_password_on_delete_bool_siempre_ok()`

**Casos cubiertos**

- Normal: los rangos del SPEC §4.8 se validan (US-1, DoD).
- Criterio de aceptación (US-1): valores fuera de rango producen error legible, sin
  `panic` ni guardado.
- Edge: límites exactos (1/120 y 5/120) → válidos.

---

### T-CFG-02 — Aplicar los ajustes a la sesión en curso

**Descripción**

- `src/app.rs` (lógica pura): `fn apply_settings(state, settings) -> AppState` que
  hace efectivos los ajustes en la sesión:
  - `auto_lock_minutes` → umbral del contador de auto-lock de fase 3 (T-LCK-01).
  - `clipboard_seconds` → duración del auto-clear del portapapeles de fase 3
    (T-CLP-03).
  - `require_password_on_delete` → activa/desactiva el borrado protegido (T-DEL-01).
- Los nuevos valores aplican **de inmediato**, sin reabrir el vault.

**Tests a crear**

- `mod test_apply_settings`:
  - `fn aplicar_auto_lock_minutos_actualiza_umbral()`
  - `fn aplicar_clipboard_segundos_actualiza_auto_clear()`
  - `fn aplicar_require_password_on_delete_activa_borrado_protegido()`
  - `fn sin_cambios_no_altera_la_sesion()`

**Casos cubiertos**

- Criterio de aceptación (US-1, DoD): los nuevos valores aplican a la sesión en curso.
- Edge: `auto_lock_minutes = 1` → el auto-lock aplica al minuto (enlazado a fase 3).

---

## 4. Concepto B — Pantalla `settings`

### T-SET-01 — Pantalla `settings` funcional con guardado explícito

**Descripción**

- `src/ui/screens.rs` + `src/app.rs`: reemplazar el placeholder de `settings` por la
  pantalla funcional del §2.3 de `DEV-FASE-4.md`:
  - Campos editables: `auto_lock_minutes`, `clipboard_seconds`,
    `require_password_on_delete` (toggle), con validación en vivo (T-CFG-01).
  - `Esc` descarta los cambios sin persistir (draft no aplica).
  - Guardado **explícito** (SPEC §4.8): al guardar se muta el estado, se aplica a la
    sesión (T-CFG-02), se actualiza `updated_at` y se ejecuta el ciclo de guardado de
    fase 2 (`.tmp` + `rename`).
- La contraseña de la sesión no cambia; el vault no se re-cifra en esta pantalla.

**Tests a crear**

- `mod test_settings_screen`:
  - `fn editar_y_guardar_persiste_y_aplica()`
  - `fn validacion_en_vivo_muestra_error_sin_guardar()`
  - `fn esc_descarta_sin_persistir()`
  - `fn guardado_actualiza_updated_at()`
  - `fn guardado_con_error_de_disco_informa_y_conserva_memoria()`
  - `fn teclas_no_mapeadas_se_ignoran()`

**Casos cubiertos**

- Criterio de aceptación (US-1, DoD): `settings` funcional con rangos validados,
  guardado explícito y aplicación a la sesión.
- Edge: guardado con error de disco → mensaje claro; el archivo previo no se corrompe
  (guardado atómico de fase 2); los cambios quedan en memoria y se informa.

---

## 5. Concepto C — Cambio de master password

### T-PWD-01 — Verificación de la password actual y validación de la nueva

**Descripción**

- `src/app.rs` / `src/crypto.rs` (**lógica pura**):
  - `fn verify_master_password(master_password, salt, params, session_key) -> bool`:
    re-derivar con el salt del vault y **comparar con la clave en memoria** (sin I/O
    adicional) para verificar la password actual (§2.4 de `DEV-FASE-4.md`).
  - Validación de la **nueva**: mínimo 8+ caracteres (aviso, no bloquea) y confirmación
    que coincida; si no coinciden → error local, no se cambia nada.
- Al fallar la actual: error "Credenciales inválidas" y el **backoff de 3 intentos de
  fase 2** aplica (contador y cuenta atrás).

**Tests a crear**

- `mod test_verify_master_password`:
  - `fn password_actual_correcta_verifica()`
  - `fn password_actual_incorrecta_no_verifica()`
  - `fn nueva_y_confirmacion_no_coinciden_error()`
  - `fn nueva_menor_de_8_avisa_no_bloquea()`
  - `fn tras_3_fallidos_activa_backoff()`

**Casos cubiertos**

- Criterio de aceptación (US-2, DoD): con la actual incorrecta se muestra error y no se
  continúa; nueva + confirmación válidas completan el cambio.
- Edge: cancelar con `Esc` en cualquier paso → no se modifica el vault.

---

### T-PWD-02 — Re-salt, re-cifrado y escritura atómica

**Descripción**

- `src/crypto.rs` + `src/storage.rs` + `src/app.rs`:
  1. Generar **nuevo salt** (16 B, `OsRng`).
  2. Re-derivar la clave con el nuevo salt y la **nueva** master password
     (`derive_key`, fase 2).
  3. **Re-cifrar** toda la payload (SPEC §3.4) con la nueva clave: `seal()` con nuevo
     nonce (24 B).
  4. Reescribir el archivo de forma **atómica** (`.tmp` + `rename`, fase 2), con el
     nuevo salt y los parámetros Argon2 en el header.
  5. Sustituir la clave en memoria por la nueva; la sesión sigue desbloqueada.
- La master password anterior deja de funcionar de inmediato.
- `zeroize` de la master password nueva y la anterior al terminar el flujo.

**Tests a crear**

- `mod test_master_password_change`:
  - `fn cambio_reescribe_con_nuevo_salt()`
  - `fn reabrir_con_password_nueva_funciona()`
  - `fn password_anterior_ya_no_funciona()`
  - `fn escritura_atomica_no_corrompe_archivo_en_fallo()`
  - `fn sesion_sigue_desbloqueada_tras_cambio()`
  - `fn claves_se_limpian_con_zeroize_al_terminar()`

**Casos cubiertos**

- Criterio de aceptación (US-2, DoD): re-salt, re-cifrado completo y guardado atómico;
  la password anterior deja de funcionar.
- Edge: fallo de disco durante el re-cifrado → no corrompe el archivo previo (guardado
  atómico); no se sustituye la clave en memoria.

---

## 6. Concepto D — Borrado con `require_password_on_delete`

### T-DEL-01 — Borrado protegido con verificación de master password

**Descripción**

- `src/app.rs`: extender el flujo de borrado de fase 2 (confirmación inline `y`/`n`,
  T-CRUD-04):
  - Con `require_password_on_delete` **activo** y tras `y`: pedir la master password
    (input oculto) y verificarla re-derivando y comparando con la clave en memoria
    (`verify_master_password`, T-PWD-01).
  - Correcta → borrar la entry, persistir (ciclo fase 2) y volver a `list`.
  - Incorrecta → error, la entry permanece, se permanece en `detail`.
  - `Esc` durante la petición → cancela, no se borra nada.
- Con el ajuste **desactivado**, el borrado se comporta igual que en fases 2–3
  (solo `y`/`n`).
- **No** aplica backoff de intentos aquí (no es unlock; §2.5 de `DEV-FASE-4.md`).
- La master password introducida se limpia con `zeroize` al terminar.

**Tests a crear**

- `mod test_protected_delete`:
  - `fn activo_pide_master_password_tras_y()`
  - `fn password_correcta_borra_y_persiste()`
  - `fn password_incorrecta_error_y_entry_permanece()`
  - `fn esc_cancela_sin_borrar()`
  - `fn desactivado_borra_solo_con_yn()`
  - `fn sin_backoff_en_la_verificacion()`
  - `fn master_password_limpia_con_zeroize()`

**Casos cubiertos**

- Criterio de aceptación (US-3, DoD): con el ajuste activo se pide y verifica la
  master password; desactivado conserva el flujo de fases 2–3.
- Edge: error de disco al persistir el borrado → se informa; el archivo no queda
  truncado (fase 2).
- Edge: borrado de la entry seleccionada → selección re-posicionada sin `panic`
  (fase 2).

---

## 7. Concepto E — Revisión final de errores

### T-ERR-01 — Mensajes de error claros y consistentes (SPEC §8)

**Descripción**

- `src/errors.rs` + `src/app.rs`: repaso final de `user_message` para todos los
  caminos del SPEC §8:
  - Contraseña incorrecta / archivo corrupto → mensaje ambiguo (no distingue).
  - `magic`/`version` inválida → "No es un vault de Cofre".
  - Error de disco → mensaje claro + archivo previo intacto (guardado atómico).
  - API de portapapeles no disponible (headless) → función desactivada con aviso
    (fase 3).
  - Validación de ajustes fuera de rango → error legible.
- Ningún mensaje revela secretos; ningún error controlable produce `panic` ni deja la
  terminal en modo raw.
- Mensajes largos se muestran legibles sin romper el layout (resize de fase 1).

**Tests a crear**

- `mod test_errors_final`:
  - `fn mensajes_consistentes_entre_pantallas()`
  - `fn no_revela_secretos_en_ningun_mensaje()`
  - `fn error_de_disco_no_sale_del_tui()`
  - `fn validacion_out_of_range_mensaje_legible()`
  - `fn sin_panic_en_ninguna_variante()`

**Casos cubiertos**

- Criterio de aceptación (US-4, DoD): errores claros y consistentes, sin `panic`, sin
  secretos.
- Edge: disco lleno / sin permisos durante un guardado → mensaje claro, archivo previo
  intacto.

---

## 8. Concepto F — Pruebas de integración y documentación

### T-INT-01 — Suite de integración de flujos completos

**Descripción**

- Rellenar `tests/integration.rs` (stub de T-00-01): flujos completos con **TUI
  simulado** (inputs simulados) o **headless**, sobre los flujos de esta fase usando
  fixtures `tests/fixtures/*.cofre` (fase 2):
  - Cambio de master password: reabrir con la nueva, rechazar la anterior.
  - Borrado protegido: activo con password correcta/incorrecta; desactivado.
  - Settings: guardado, persistencia tras reabrir y aplicación a la sesión.
- La suite no requiere TTY ni red.

**Tests a crear**

- `mod test_integration_flows`:
  - `fn flujo_cambio_master_password_completo()`
  - `fn flujo_borrado_protegido_completo()`
  - `fn flujo_settings_persiste_tras_reabrir()`
  - `fn fixtures_corruptos_validan_mensajes_sin_panic()`

**Casos cubiertos**

- Criterio de aceptación (US-5, DoD): tests de integración de los flujos completos de
  esta fase, que pasan en CI/headless.
- Edge: fixtures `.cofre` corruptos → validan los mensajes de error sin `panic`.

---

### T-DOC-01 — Documentación final

**Descripción**

- Crear el **README** del proyecto: uso del producto, atajos definitivos (SPEC §5) y
  requisitos de entorno (SPEC §12 del PRD).
- Alinear la documentación existente (PRD M4: "documentación final") con la
  implementación real de las cuatro fases.
- Revisar que los atajos y flujos documentados coinciden con el código.

**Tests a crear**

- Check estático: `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo build`.

**Casos cubiertos**

- Criterio de aceptación (US-5, DoD): el README documenta el uso y los atajos
  definitivos (SPEC §5).
- Edge: la documentación no promete funcionalidad fuera de alcance (§1.2).

---

## 9. Orden de ejecución sugerido

| # | Tarea | Depende de |
|---|---|---|
| 1 | T-00-01 | — |
| 2 | T-00-02 | T-00-01 |
| 3 | T-CFG-01 | T-00-01 |
| 4 | T-CFG-02 | T-CFG-01 |
| 5 | T-SET-01 | T-CFG-01, T-CFG-02, T-00-02 |
| 6 | T-PWD-01, T-PWD-02 | T-CFG-01 |
| 7 | T-DEL-01 | T-CFG-02, T-PWD-01 |
| 8 | T-ERR-01 | T-SET-01, T-PWD-02, T-DEL-01 |
| 9 | T-INT-01 | T-SET-01, T-PWD-02, T-DEL-01 |
| 10 | T-DOC-01 | T-INT-01 |

Racional: primero se fijan tipos y estructura (Concepto 0) y la configuración pura
(Concepto A), base testeable sin TTY. La pantalla `settings` (B) y el cambio de master
password (C) dependen solo de esa base; el borrado protegido (D) los reutiliza. La
revisión de errores (E) y las pruebas de integración (F) se cierran sobre los flujos
ya integrados, sin fijar mensajes ni estructura de tests prematuramente.

## 10. Mapa con el Definition of Done de `DEV-FASE-4.md`

| DoD (§4) | Tareas que lo satisfacen |
|---|---|
| `settings` funcional con rangos validados y guardado explícito (SPEC §4.8) | T-CFG-01, T-SET-01, T-00-02 |
| Cambio de master password: nuevo salt, re-cifrado completo y guardado atómico | T-PWD-01, T-PWD-02 |
| `require_password_on_delete` pide y verifica la master password; desactivado conserva el flujo | T-DEL-01, T-CFG-02 |
| Errores del SPEC §8 claros y consistentes sin `panic` | T-ERR-01 |
| `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test` en verde | todos (DoD por tarea), explícito T-00-01 |
| Tests: validación de rangos, cambio de master password (re-salt/re-cifrado), verificación en borrado e integración | T-CFG-01, T-PWD-01..02, T-DEL-01, T-INT-01 |
| Sin funcionalidad fuera de alcance (§1.2) | T-00-01, T-DOC-01 |
| Máquina de pantallas de fases 1–3 conservada | T-00-02, T-SET-01, T-DEL-01 |
| Documentación final (README) | T-DOC-01 |