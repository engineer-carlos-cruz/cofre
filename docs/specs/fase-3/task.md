# TASK-FASE-3 — Tareas de desarrollo: Experiencia de usuario (UX)

> Desglose atómico de las tareas de implementación de la fase 3 de **Cofre**.
> Derivado de `docs/PRD.md` (M3 UX), `docs/SPEC.md` y
> `docs/specs/fase-3/DEV-FASE-3.md`.
> Estado: **planificado**.

## 1. Propósito y convenciones

Este documento descompone la fase 3 en **tareas atómicas agrupadas por concepto**.
Cada tarea incluye:

- **Descripción**: qué se implementa y en qué módulos (`src/…`).
- **Tests a crear**: casos que deben cubrirse.
- **Casos cubiertos**: comportamiento normal, errores, criterios de aceptación (según
  las historias de usuario de `DEV-FASE-3.md`) y edge cases.

Convenciones:

- **Naming de tests**: `mod test_<modulo>`, casos `fn <comportamiento>_<condición>`.
- **Sin `panic`**: el código de la app no debe hacer `panic` en ninguna condición
  controlable; los tests deben verificar ausencia de `panic` en estados límite.
- **Testabilidad**: la generación, el análisis de fortaleza, la lógica de
  búsqueda/filtros y el auto-lock se implementan como **funciones puras**
  (`password.rs`, `app.rs`) para poder testearlas sin TTY. El UI solo dibuja;
  la decisión está en `app.rs`. `clipboard.rs` encapsula solo la interacción con el
  portapapeles del sistema.
- **DoD por tarea**: `cargo fmt --check` sin diferencias, `cargo clippy -- -D warnings`
  sin warnings y `cargo test` verde.
- **Fuera de alcance**: ninguna tarea incluye `settings` funcional, cambio de master
  password ni `require_password_on_delete` (sección §1.2 de `DEV-FASE-3.md`).
- La máquina de pantallas de fases 1 y 2 se **conserva**: las tareas de esta fase
  integran estado/lógica en `app.rs` y `ui/` sin cambiar las transiciones existentes.

## 2. Concepto 0 — Proyecto y dependencias

### T-00-01 — Añadir `arboard` y crear los módulos `password.rs` y `clipboard.rs`

**Descripción**

- Añadir a `Cargo.toml` (se suma a las de fases 1 y 2):
  - `arboard` (portapapeles del sistema, con fallback headless).
- Versión estable actual, fijada en el `Cargo.lock`; compatible con las dependencias
  previas. Sin dependencias de red (todo local).
- Crear los módulos de negocio del §2.2 de `DEV-FASE-3.md`:
  - `src/password.rs` (generador + análisis de fortaleza, lógica pura sin TTY).
  - `src/clipboard.rs` (wrapper de `arboard` + auto-clear).
- Los módulos nuevos deben compilar como stubs declarados (sin lógica todavía).

**Tests a crear**

- Check estático: `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo build`.

**Casos cubiertos**

- Normal: compilación limpia con la nueva dependencia y la estructura declarada.
- Criterio de aceptación (DoD): no se incluyen dependencias fuera de la lista
  (sin nada de settings/cambio de master password).
- Edge: versión de `arboard` compatible con las dependencias de fases 1 y 2.

---

### T-00-02 — Estado de UX en `AppState`

**Descripción**

- `src/app.rs`: extender `AppState` con el estado de UX de esta fase:
  - **Búsqueda**: `search_query: String` y flag de input activo.
  - **Filtro de tag**: `tag_filter: Option<String>`.
  - **Copiado**: `copied: Option<CopiedKind>` (password/username) y
    `clipboard_deadline: Option<Instant>` para el indicador y el auto-clear.
  - **Inactividad**: `last_key_activity: Instant` para el auto-lock.
  - **Generador**: `password_options: PasswordOptions` y `generated: String`.
- Modelar funciones puras auxiliares: `fn visible_entries(&self) -> Vec<&Entry>`
  (aplica búsqueda + filtro de tag) y `fn is_locked(&self) -> bool`.
- La máquina de pantallas de fases 1 y 2 no cambia.

**Tests a crear**

- `mod test_ux_state`:
  - `fn estado_ux_defaults_iniciales()` → búsqueda vacía, sin filtro, sin copiado,
    sin inactividad inicial definida.
  - `fn visible_entries_sin_filtros_es_todas()`
  - `fn visible_entries_con_busqueda_filtra()`
  - `fn visible_entries_con_tag_filtra()`

**Casos cubiertos**

- Normal: el estado de UX se modela en `app.rs`, testeable sin TTY.
- Criterio de aceptación (DoD): la decisión (búsqueda, filtro, copiado, auto-lock)
  vive en `app.rs`; el UI solo dibuja.
- Edge: `visible_entries` con búsqueda y tag a la vez (combinación, ver T-SRC-03).

---

## 3. Concepto A — Generador de contraseñas (`password.rs`)

### T-GEN-01 — `PasswordOptions` y validación

**Descripción**

- `src/password.rs`:
  - `struct PasswordOptions { length: u8, lowercase: bool, uppercase: bool,
    digits: bool, symbols: bool, avoid_ambiguous: bool, one_of_each_class: bool }`
    con defaults: `length = 20`, los 4 charsets activos, resto en `false`.
  - Constantes de charsets del SPEC §4.3: minúsculas, mayúsculas, dígitos y símbolos
    `!@#$%^&*()-_=+[]{};:,.?`.
  - `fn validate(&self) -> Result<()>`:
    - `length` fuera de 4–128 → error de validación legible.
    - Ningún charset activo → error legible.
- Los errores de validación se mapean a `CofreError` (sin `panic`).

**Tests a crear**

- `mod test_password_options`:
  - `fn defaults_largo_20_y_cuatro_charsets()`
  - `fn largo_menor_de_4_error()`
  - `fn largo_mayor_de_128_error()`
  - `fn sin_charsets_activos_error()`
  - `fn rango_valido_ok()`

**Casos cubiertos**

- Normal: las opciones del SPEC §4.3 se modelan con validación.
- Criterio de aceptación (US-1, DoD): largo fuera de rango y cero charsets producen
  error legible, sin `panic`.
- Edge: largo en límites 4 y 128 → válido.

---

### T-GEN-02 — Generación con Fisher-Yates, ambiguos y "una de cada clase"

**Descripción**

- `src/password.rs`: `generate_password(options: &PasswordOptions) -> Result<String>`:
  1. Construir el pool según los charsets activos (aplicando "evitar ambiguos"
     `0O1lI` sobre el pool).
  2. Si `one_of_each_class`: garantizar 1 carácter de cada clase activa **antes** de
     filtrar los ambiguos (para que nunca falle por quedarse un charset sin
     caracteres) y rellenar el resto del pool.
  3. Barajar con `OsRng` (Fisher-Yates).
- Llamar a `validate()` al inicio; cualquier error propaga sin generar.

**Tests a crear**

- `mod test_password_gen`:
  - `fn largo_generado_coincide_con_opcion()`
  - `fn evitar_ambiguos_no_incluye_0O1lI()`
  - `fn una_de_cada_clase_garantiza_minimo_por_clase()`
  - `fn una_de_cada_clase_con_una_sola_clase_trivial()`
  - `fn una_de_cada_clase_mas_ambiguos_nunca_falla()` → casos con charsets que quedan
    sin caracteres tras el filtro.
  - `fn sin_charsets_error_y_no_genera()`
  - `fn resultados_distintos_en_generaciones_seguidas()` → dos llamadas no son
    idénticas (probabilísticamente).

**Casos cubiertos**

- Normal: el resultado respeta las opciones del SPEC §4.3 (US-1, DoD).
- Criterio de aceptación (US-1): con "evitar ambiguos" no aparece `0O1lI`; con "una
  de cada clase" hay al menos un carácter de cada clase activa.
- Edge: "una de cada clase" con una sola clase activa → se cumple trivialmente.
- Edge: "una de cada clase" + "evitar ambiguos" → nunca falla (la garantía se
  resuelve antes de filtrar).

---

## 4. Concepto B — Análisis de fortaleza (`password.rs`)

### T-STR-01 — `estimate_strength` (entropía y clasificación)

**Descripción**

- `src/password.rs`:
  - `enum Strength { Weak, Medium, Strong }` con `fn label() -> &'static str`
    ("débil"/"media"/"fuerte").
  - `fn estimate_strength(password: &str) -> Strength` basado en entropía estimada
    `pool_size ^ length` (SPEC §7), donde `pool_size` se estima por los caracteres
    presentes (dígitos, minúsculas, mayúsculas, símbolos).
  - Umbrales: **débil** < ~48 bits, **media** 48–80, **fuerte** > 80.
- Cálculo de entropía en bits sin desbordamiento (`f64`), incluido largo máximo 128.

**Tests a crear**

- `mod test_strength`:
  - `fn corta_un_charset_debil()`
  - `fn media_entre_48_y_80()`
  - `fn larga_varios_charsets_fuerte()`
  - `fn password_vacia_neutro_sin_error()` → estado neutro o sin evaluar, sin error.
  - `fn largo_128_no_desborda()` → no `panic`, clasifica fuerte.

**Casos cubiertos**

- Normal: la clasificación sigue el SPEC §7 (US-2, DoD).
- Criterio de aceptación (US-2): muy corta con un charset = débil; la estimación no
  depende de diccionarios.
- Edge: password vacía → sin error; largo máximo → sin desbordamiento.

---

### T-STR-02 — Visual de fortaleza en el TUI

**Descripción**

- `src/ui/screens.rs`: junto al resultado del generador se muestra la barra del SPEC §7
  (`▓▓▓▓░░░░`) + color + texto de la clasificación.
- Se recalcula con cada regeneración o cambio de opciones.

**Tests a crear**

- `mod test_strength_ui`:
  - `fn barra_y_texto_visibles_en_generator()`
  - `fn clasificacion_se_recalcula_al_regenerar()` → el estado expone la fortaleza
    actualizada tras un cambio.

**Casos cubiertos**

- Criterio de aceptación (US-2, DoD): el generador muestra clasificación + barra
  visual para cualquier resultado y la recalcula en vivo.
- Edge: password vacía → estado neutro sin error (enlazado a T-STR-01).

---

## 5. Concepto C — Pantalla `generator` (UI y acciones)

### T-UI-GEN-01 — Pantalla `generator` funcional con opciones en vivo

**Descripción**

- `src/ui/screens.rs` + `src/app.rs`: reemplazar el placeholder de `generator` por la
  pantalla funcional del §2.5 de `DEV-FASE-3.md`:
  - Largo (input/stepper), toggles de los 4 charsets, "evitar ambiguos" y "al menos
    una de cada clase".
  - Cada cambio regenera en vivo (`generate_password`) y actualiza la fortaleza.
  - `r` regenera un resultado nuevo.
  - `Esc`/`q` vuelve a `list` conservando la selección (transición de fase 1).
- La máquina de pantallas no cambia; `g` desde `list` sigue abriendo `generator`.

**Tests a crear**

- `mod test_generator_screen`:
  - `fn g_desde_list_abre_generator_con_defaults()`
  - `fn cambiar_opcion_regenera_resultado()`
  - `fn r_regenera_resultado()`
  - `fn esc_vuelve_a_list_con_seleccion()`
  - `fn teclas_no_mapeadas_se_ignoran()`
  - `fn largo_invalido_muestra_error_sin_panic()`

**Casos cubiertos**

- Criterio de aceptación (US-1, DoD): generador funcional con regeneración en vivo.
- Edge: largo fuera de rango o sin charsets → error legible, sin `panic` (enlazado a
  T-GEN-01).
- Edge: teclas no mapeadas → ignoradas sin efecto ni ruido.

---

### T-UI-GEN-02 — Acciones `Enter` (copiar) y `s` (guardar como nueva entry)

**Descripción**

- `Enter` en `generator` → copia la contraseña generada al portapapeles con indicador
  "copied" (integra T-CLP-02).
- `s` en `generator` → abre `form` de **nueva entry** con el campo `password`
  pre-cargado con la contraseña generada (§2.9 de `DEV-FASE-3.md`).
  - El usuario completa `title`/`username` y guarda con el ciclo de fase 2.
  - `Esc` descarta sin persistir.
  - La validación de `title`/`username` no vacíos de fase 2 aplica.
- `src/app.rs`: al salir del `form` sin guardar, la contraseña generada se descarta con
  `zeroize` (enlazado a T-SEC-01).

**Tests a crear**

- `mod test_generator_actions`:
  - `fn enter_copia_al_portapapeles_con_indicador()`
  - `fn s_abre_form_con_password_prerellenada()`
  - `fn guardar_tras_s_persiste_la_entry()`
  - `fn esc_en_form_descarta_sin_persistir()`
  - `fn validacion_de_titulo_usuario_aplica()`
  - `fn password_generada_se_descarta_si_no_se_guarda()`

**Casos cubiertos**

- Criterio de aceptación (US-6, DoD): `s` guarda la contraseña generada como nueva
  entry con el CRUD de fase 2.
- Edge: guardado con error de disco → la entry queda en memoria y se informa; el
  archivo previo no se corrompe (guardado atómico de fase 2).

---

## 6. Concepto D — Búsqueda y filtrado

### T-SRC-01 — Búsqueda incremental con `/`

**Descripción**

- `src/app.rs` + `src/ui/screens.rs`: `/` en `list` abre un input de búsqueda
  **incremental**.
- `fn matches_query(entry, query) -> bool`: case-insensitive, substring sobre
  **título, username, URL y tags** (SPEC §4.4).
- Cada pulsación filtra en vivo (`visible_entries`); la selección se re-posiciona al
  primer resultado.
- `Esc` o vaciar el input restaura la lista completa y conserva la selección.
- Sin resultados → mensaje informativo "sin resultados" (distinto del estado vacío
  global).

**Tests a crear**

- `mod test_search`:
  - `fn slash_abre_input_de_busqueda()`
  - `fn busqueda_filtra_por_titulo()`
  - `fn busqueda_filtra_por_username_url_tags()`
  - `fn case_insensitive()`
  - `fn substring_no_parcial_exacto()`
  - `fn esc_o_vacio_restaura_lista()`
  - `fn seleccion_se_reposiciona_al_primer_resultado()`
  - `fn sin_resultados_mensaje_distinto_del_vacio()`

**Casos cubiertos**

- Criterio de aceptación (US-3, DoD): búsqueda incremental sobre los 4 campos,
  case-insensitive, con re-posición de selección y restauración con `Esc`.
- Edge: búsqueda sobre 10.000 entries → < 50 ms (SPEC §9).

---

### T-SRC-02 — Prefijos `t:` y `u:`

**Descripción**

- `src/app.rs`: si la consulta comienza por `t:` → filtra por **tag** (substring del
  tag); si comienza por `u:` → filtra por **username** (case-insensitive, substring).
- Prefijo incompleto ("t:" sin valor) → se trata como búsqueda de texto literal o
  estado de filtro vacío, sin error (comportamiento documentado).

**Tests a crear**

- `mod test_search_prefixes`:
  - `fn t_dev_filtra_por_tag()`
  - `fn u_github_filtra_por_username()`
  - `fn prefijo_incompleto_sin_error()`
  - `fn prefijo_case_insensitive()`

**Casos cubiertos**

- Criterio de aceptación (US-3, DoD): `t:dev` y `u:GitHub` filtran por tag y username
  respectivamente.
- Edge: prefijo incompleto → sin error, comportamiento definido y documentado.

---

### T-SRC-03 — Filtro por tag con `f`, combinable

**Descripción**

- `src/ui/screens.rs` + `src/app.rs`: barra de tags en `list`; `f` selecciona o
  des-selecciona un tag (`tag_filter`).
- El filtro de tag se **combina** con la búsqueda (`visible_entries` aplica ambos).
- La selección se re-posiciona al filtrar; sin resultados → mensaje informativo.

**Tests a crear**

- `mod test_tag_filter`:
  - `fn f_selecciona_y_des-selecciona_tag()`
  - `fn filtro_tag_combina_con_busqueda()` → búsqueda + tag a la vez.
  - `fn sin_resultados_con_tag_mensaje_informativo()`
  - `fn seleccion_se_reposiciona_al_filtrar()`
  - `fn tags_con_espacios_y_acentos_coinciden()` → substring exacto case-insensitive.

**Casos cubiertos**

- Criterio de aceptación (US-3, DoD): filtro por tag con `f` combinable con la
  búsqueda.
- Edge: filtrar y borrar la entry seleccionada (fase 2) → selección re-posicionada sin
  `panic`.

---

## 7. Concepto E — Portapapeles (`clipboard.rs`)

### T-CLP-01 — Wrapper de `arboard` con fallback headless

**Descripción**

- `src/clipboard.rs`: encapsular la API de `arboard`:
  - `struct Clipboard` que se inicializa al arrancar la sesión desbloqueada.
  - `fn copy(&mut self, text: &str) -> Result<()>` y `fn clear(&mut self)`.
- Si la API no está disponible (entorno headless, sin display):
  - La función se **desactiva con aviso** al usuario (SPEC §8), sin romper la app.
- `errors.rs`: variante de error para el portapapeles no disponible (mensaje legible,
  sin revelar secretos).

**Tests a crear**

- `mod test_clipboard`:
  - `fn copia_exitosa_cuando_disponible()` (en entorno con display o mock).
  - `fn headless_desactiva_con_aviso()` → el estado del clipboard queda desactivado y
    se emite el aviso.
  - `fn clear_no_falla_si_desactivado()` → idempotente.

**Casos cubiertos**

- Criterio de aceptación (US-4, DoD): en headless la función se desactiva con aviso y
  la app sigue funcionando.
- Edge: `clear` sobre un clipboard ya desactivado → sin error.

---

### T-CLP-02 — Acciones `c`/`C` con indicador "copied"

**Descripción**

- `src/app.rs` + `src/ui/screens.rs`:
  - `c` copia la **password** y `C` (shift) copia el **username**, desde `list` y
    `detail` (SPEC §4.5, §5).
  - Tras copiar: `copied = Some(...)` y `clipboard_deadline = now + clipboard_seconds`.
  - Indicador visual "copied" con cuenta atrás del auto-clear.
  - Copiar dos veces seguidas **reinicia** el temporizador de la última copia.
- `C` con username vacío → copia cadena vacía con aviso, sin error.
- Copiar desde una entry con password oculta → se copia el valor real, no la máscara.

**Tests a crear**

- `mod test_copy`:
  - `fn c_copia_password_en_list_y_detail()`
  - `fn c_mayuscula_copia_username()`
  - `fn indicador_copied_y_cuenta_atras_en_estado()`
  - `fn copiar_dos_veces_reinicia_deadline()`
  - `fn username_vacio_avisa_sin_error()`
  - `fn copia_valor_real_no_la_mascara()`

**Casos cubiertos**

- Criterio de aceptación (US-4, DoD): `c`/`C` copian con indicador y cuenta atrás.
- Edge: recopiar → el auto-clear se reinicia con la última copia.

---

### T-CLP-03 — Auto-clear del portapapeles

**Descripción**

- `src/app.rs` / event loop: al vencer `clipboard_deadline` (vencido
  `clipboard_seconds`, defecto 15 s desde `Settings`), se ejecuta `Clipboard::clear()`.
- El portapapeles se limpia **siempre**:
  - Al salir de la aplicación (se integra en el flujo de teardown de fase 1).
  - Al lockear, incluido el auto-lock (T-LCK-02).
- `clipboard_seconds` se lee de `Settings` (fase 2), con defecto 15.

**Tests a crear**

- `mod test_auto_clear`:
  - `fn vencido_deadline_limpia_portapapeles()`
  - `fn antes_de_deadline_no_limpia()`
  - `fn salida_limpia_portapapeles()`
  - `fn lock_limpia_portapapeles()`
  - `fn clipboard_seconds_default_15()`

**Casos cubiertos**

- Criterio de aceptación (US-4, DoD): auto-clear tras `clipboard_seconds` y siempre al
  salir o lockear.
- Edge: salir durante la cuenta atrás → se limpia igualmente en el teardown.

---

## 8. Concepto F — Auto-lock

### T-LCK-01 — Contador de inactividad y transición a `unlock`

**Descripción**

- `src/app.rs` / event loop:
  - Cualquier tecla resetea `last_key_activity` (incluidos los inputs de `form` y de
    búsqueda — regla simple del SPEC §4.6).
  - Eventos que no son de teclado (p. ej. `Resize`) **no** resetean el contador.
  - El contador solo corre mientras la sesión está desbloqueada.
  - Al superar `auto_lock_minutes` (defecto 5, desde `Settings`) desde la última
    tecla → transición a pantalla `unlock` **sin cerrar la app**.
- `fn check_auto_lock(now) -> bool` (función pura testeable).

**Tests a crear**

- `mod test_auto_lock`:
  - `fn inactividad_supera_minutos_dispara_lock()`
  - `fn cualquier_tecla_resetea_contador()`
  - `fn resize_no_resetea_contador()`
  - `fn contador_no_corre_bloqueado()`
  - `fn auto_lock_minutes_default_5()`
  - `fn con_1_minuto_lock_al_minuto()`
  - `fn auto_lock_durante_confirmacion_de_borrado_la_descarta()` → vuelve a `unlock`.

**Casos cubiertos**

- Criterio de aceptación (US-5, DoD): la app pasa a `unlock` sin cerrarse tras
  `auto_lock_minutes` de inactividad real de teclas.
- Edge: inactividad durante inputs de texto → aplica igualmente; resize no resetea.

---

### T-LCK-02 — `zeroize` y limpieza del portapapeles en el lock

**Descripción**

- Al producirse el lock (auto-lock o `Ctrl+L`):
  - Se descartan de memoria la clave derivada y la payload descifrada (`zeroize`),
    reutilizando los tipos de fase 2.
  - Se limpia el portapapeles aunque hubiera una copia en curso (integra T-CLP-03).
  - Se descarta el estado de búsqueda/filtro/copiado de la sesión.
- El backoff de 3 intentos de la fase 2 se conserva y **no colisiona** con el auto-lock
  (un auto-lock no resetea el contador de intentos fallidos del unlock).

**Tests a crear**

- `mod test_lock_cleanup`:
  - `fn lock_descarta_clave_y_payload_con_zeroize()`
  - `fn lock_limpia_portapapeles()`
  - `fn lock_limpia_estado_ux()` → búsqueda/filtro/indicador descartados.
  - `fn re-desbloqueo_funciona_con_master_password()`
  - `fn backoff_de_fase2_no_se_resetea_por_auto_lock()`

**Casos cubiertos**

- Criterio de aceptación (US-5, DoD): en el lock se descarta clave/payload con
  `zeroize` y se limpia el portapapeles.
- Edge: auto-lock con una copia en curso → portapapeles limpio igualmente.

---

## 9. Concepto G — Seguridad de memoria

### T-SEC-01 — `zeroize` de datos sensibles de UX

**Descripción**

- La contraseña generada, los valores copiados y los buffers de búsqueda que contienen
  datos de entries se tratan como sensibles y se limpian con `zeroize` al descartarse:
  - Contraseña generada al salir de `generator`/`form` sin guardar (enlazado a
    T-UI-GEN-02).
  - Valores copiados en el portapapeles tras el auto-clear o el lock.
  - Buffer de búsqueda al cerrar la sesión / lockear.
- No hay logs ni volcados con contraseñas en claro.

**Tests a crear**

- `mod test_zeroize_ux`:
  - `fn password_generada_limpia_al_descartar()`
  - `fn buffer_de_busqueda_limpio_al_lockear()`
  - `fn sin_logs_en_claro()` → los mensajes del copiado/búsqueda no incluyen secretos.

**Casos cubiertos**

- Criterio de aceptación (DoD): los datos sensibles de UX se limpian al descartarse.
- Edge: salida por `panic` → los `drop` de tipos `zeroize` siguen ejecutándose.

---

## 10. Orden de ejecución sugerido

| # | Tarea | Depende de |
|---|---|---|
| 1 | T-00-01, T-00-02 | — |
| 2 | T-GEN-01, T-GEN-02 | T-00-02 |
| 3 | T-STR-01, T-STR-02 | T-GEN-01, T-GEN-02 |
| 4 | T-CLP-01 | T-00-01 |
| 5 | T-UI-GEN-01, T-UI-GEN-02 | T-GEN-01..02, T-STR-01..02, T-CLP-02 |
| 6 | T-SRC-01, T-SRC-02, T-SRC-03 | T-00-02 |
| 7 | T-CLP-02, T-CLP-03 | T-CLP-01 |
| 8 | T-LCK-01, T-LCK-02 | T-00-02, T-CLP-03 |
| 9 | T-SEC-01 | T-UI-GEN-02, T-LCK-02 |

Racional: primero se fijan dependencias y estado (Concepto 0), luego la lógica pura de
`password.rs` (A y B), base testeable sin TTY. El portapapeles (E) se arranca con su
wrapper antes de integrarlo en las acciones del generador (C) y de la copia (E). La
búsqueda (D) y el auto-lock (F) dependen solo del estado; la seguridad de memoria (G)
se cierra al final sobre los flujos ya integrados.

## 11. Mapa con el Definition of Done de `DEV-FASE-3.md`

| DoD (§4) | Tareas que lo satisfacen |
|---|---|
| Generador funcional con todas las opciones del SPEC §4.3 y regeneración en vivo | T-GEN-01, T-GEN-02, T-UI-GEN-01 |
| Análisis de fortaleza del SPEC §7 con clasificación y visual de barra | T-STR-01, T-STR-02 |
| Búsqueda incremental `/`, prefijos `t:`/`u:` y filtro por tag `f` combinable | T-SRC-01, T-SRC-02, T-SRC-03 |
| `c`/`C` copian con indicador "copied"; auto-clear y limpieza al salir/lockear | T-CLP-01, T-CLP-02, T-CLP-03 |
| Auto-lock tras `auto_lock_minutes` con `zeroize` de clave/payload y limpieza de portapapeles | T-LCK-01, T-LCK-02 |
| En headless, el portapapeles se desactiva con aviso sin romper la app | T-CLP-01 |
| `s` desde `generator` guarda la contraseña como nueva entry con CRUD de fase 2 | T-UI-GEN-02 |
| Máquina de pantallas de fases 1–2 conservada | T-00-02, T-UI-GEN-01, T-SRC-01..03 |
| `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test` en verde | todos (DoD por tarea), explícito T-00-01 |
| Tests: garantías del generador, fortaleza, búsqueda/filtros, auto-clear, auto-lock | T-GEN-01..02, T-STR-01, T-SRC-01..03, T-CLP-03, T-LCK-01 |
| Sin funcionalidad fuera de alcance (§1.2) | T-00-01, T-00-02 |
| `zeroize` de datos sensibles de UX | T-SEC-01 |