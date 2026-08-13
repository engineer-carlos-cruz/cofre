# TASK-FASE-1 — Tareas de desarrollo: Fundación del proyecto (Skeleton)

> Desglose atómico de las tareas de implementación de la fase 1 de **Cofre**.
> Derivado de `docs/PRD.md` (M1 Skeleton), `docs/SPEC.md` y `docs/specs/fase-1/DEV-FASE-1.md`.
> Estado: **planificado**.

## 1. Propósito y convenciones

Este documento descompone la fase 1 en **tareas atómicas agrupadas por concepto**.
Cada tarea incluye:

- **Descripción**: qué se implementa y en qué módulos (`src/…`).
- **Tests a crear**: casos que deben cubrirse.
- **Casos cubiertos**: comportamiento normal, errores, criterios de aceptación (según
  las historias de usuario de `DEV-FASE-1.md`) y edge cases.

Convenciones:

- **Naming de tests**: `mod test_<modulo>`, casos `fn <comportamiento>_<condición>`.
- **Sin `panic`**: el código de la app no debe hacer `panic` en ninguna condición
  controlable; los tests deben verificar ausencia de `panic` en estados límite.
- **Testabilidad**: la máquina de pantallas, el layout, la confirmación de salida y el
  mapeo de errores se implementan como **funciones puras** (`app.rs`, `ui/layout.rs`,
  `errors.rs`) para poder testearlas sin TTY.
- **DoD por tarea**: `cargo fmt --check` sin diferencias, `cargo clippy -- -D warnings`
  sin warnings y `cargo test` verde.
- **Fuera de alcance**: ninguna tarea incluye crypto, storage, CRUD real, generador,
  portapapeles ni auto-lock (sección §1.2 de `DEV-FASE-1.md`).

## 2. Concepto 0 — Proyecto Cargo y estructura

### T-SK-01 — Crear el crate y la estructura de módulos

**Descripción**

- Crear crate binario `cofre` (Rust edition **2021+**).
- `Cargo.toml` con dependencias **solo** de terminal para esta fase:
  - `crossterm` (backend, eventos).
  - `ratatui` (framework TUI).
  - Versiones estables actuales, compatibles entre sí; fijadas en el `Cargo.lock`.
- Crear la estructura de módulos especificada en §2.2 de `DEV-FASE-1.md`:

  ```text
  cofre/
  ├── Cargo.toml
  └── src/
      ├── main.rs            # Entry point, orquestación init/teardown
      ├── app.rs             # Estado global, máquina de pantallas (enum Screen)
      ├── terminal.rs        # Setup ratatui/crossterm, raw mode, teardown
      ├── ui/
      │   ├── mod.rs         # Render del frame según la pantalla activa
      │   └── screens.rs     # Funciones de dibujo por pantalla (placeholder)
      └── errors.rs          # Tipos de error base de la aplicación
  ```

- **No** crear los módulos de negocio (`crypto`, `storage`, `models`, `password`,
  `clipboard`, `config`) — son de fases posteriores.

**Tests a crear**

- `src/main.rs` compila y arranca con placeholders.
- Check estático: `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo build`.

**Casos cubiertos**

- Normal: compilación limpia con la estructura declarada.
- Criterio de aceptación (DoD): no se incluyen dependencias de crypto/storage/clipboard.
- Edge: versión de edición compatible con las dependencias instaladas.

---

### T-SK-02 — Esqueleto de `main.rs` (orquestación)

**Descripción**

- `main.rs` orquesta `init → event loop → teardown`.
- Devuelve el `exit code` al SO; ante error de arranque propaga por `stderr`.

**Tests a crear**

- Smoke headless: ejecutar el binario con `stdout` redirigido (no-TTY) → error legible
  por `stderr`, exit ≠ 0 (comparte caso con T-ERR-ST-01).

**Casos cubiertos**

- Normal: orquestación de los tres pasos en orden.
- Criterio de aceptación (US-1, DoD): el shell no recupera el control hasta salir.
- Edge: arranque sin TTY no cuelga ni hace `panic`.

---

## 3. Concepto A — Tipos de error base

### T-ERR-01 — Definir `CofreError` con mensajes legibles

**Descripción**

- `src/errors.rs`: `enum CofreError` con variantes útiles para esta fase
  (p. ej. `NotTty`, `TermNotSet`, `RawMode`, `AlternateScreen`, `Teardown`) y
  mensajes `Display` legibles en español.
- Implementar `std::error::Error`.
- Nunca usar `panic` para condiciones controladas.

**Tests a crear**

- `mod test_errors`:
  - `fn display_<variante>_mensaje_legible()` → el `Display` de cada variante coincide
    con el mensaje definido.
  - `fn error_se_puede_encadenar_source()` → `source()` devuelve `None` (o el origen
    correcto si se encadena).

**Casos cubiertos**

- Normal: cada variante produce un mensaje entendible.
- Criterio de aceptación (US-5): el usuario entiende la causa antes de un cierre.
- Edge: mensajes sin fugas de datos sensibles.

---

### T-ERR-02 — Mapeo de error → exit code

**Descripción**

- Función pura `fn exit_code(error: &CofreError) -> i32` en `errors.rs`.
- Códigos acordados:
  - `0` salida normal.
  - `1` error de arranque.
  - `2` error de terminal (raw mode / alternate screen).

**Tests a crear**

- `mod test_exit_codes`:
  - `fn normal_es_cero()`
  - `fn arranque_es_uno()`
  - `fn terminal_es_dos()`
  - `fn no_se_traga_error_con_exit_0()` → ningún `CofreError` mapea a `0`.

**Casos cubiertos**

- Normal: cada categoría de error devuelve el código correcto.
- Criterio de aceptación (US-5, DoD): la salida por error usa códigos definidos; no se
  traga el error con `exit 0`.
- Edge: error no contemplado → código por defecto no-`0`.

---

## 4. Concepto B — Ciclo de vida de la terminal

### T-TERM-01 — Init: raw mode + alternate screen + cursor oculto

**Descripción**

- `src/terminal.rs`: `init()` encapsula el ciclo de vida de la terminal.
  - Habilitar raw mode.
  - Entrar en la pantalla alternativa.
  - Ocultar el cursor.
- Si **cualquier** paso falla: devolver `CofreError` (no `panic`), sin dejar la
  terminal en modo raw.
- Devolver un guard/estructura `Terminal` que recuerde el estado para teardown.

**Tests a crear**

- `mod test_terminal_init`:
  - `fn init_falla_devuelve_error()` → en entorno sin TTY o con terminal no disponible
    devuelve `Err(...)`, no `panic`.
  - `fn init_no_deja_raw_mode_en_fallo()` → tras un init fallido el estado del guard
    indica que no hay limpieza pendiente.

**Casos cubiertos**

- Normal: init completo habilita los tres pasos en orden.
- Criterio de aceptación (US-1): si el arranque falla, mensaje claro y exit ≠ 0.
- Edge: `Ctrl+C` durante el init → teardown seguro (cubierto en T-EXIT-02).
- Edge: double init → sin efectos visibles (segundo init restaura/renueva limpiamente).

---

### T-TERM-02 — Teardown idempotente

**Descripción**

- `teardown()`: muestra el cursor, sale de la pantalla alternativa y desactiva raw mode.
- Debe ser **idempotente**: ejecutarlo dos veces no produce errores ni `panic`.

**Tests a crear**

- `mod test_terminal_teardown`:
  - `fn teardown_doble_no_falla()` → invocar dos veces seguidas no devuelve error.
  - `fn teardown_sin_init_no_falla()` → llamar teardown sin haber hecho init.
  - `fn teardown_tras_error_parcial_no_panic()` → teardown con estado incompleto.

**Casos cubiertos**

- Normal: teardown restaura cursor, pantalla alternativa y modo cooked (US-3).
- Criterio de aceptación (US-3): invocarlo 2× consecutivas no produce error ni `panic`.
- Edge: teardown tras un fallo en init (estado parcial).
- Edge: salida con render interrumpido (contenido parcial) → sin artefactos.

---

### T-TERM-03 — Hook de panic con teardown

**Descripción**

- Registrar un `panic hook` que, si se está dentro del raw mode, ejecute `teardown()`
  (sin enmascarar el mensaje de panic) antes de re-propagarlo.
- El hook solo limpia si hay limpieza pendiente.

**Tests a crear**

- `mod test_panic_hook`:
  - `fn hook_ejecuta_teardown_en_estado_activo()` → con una señal/hook de prueba, el
    teardown se invoca ante un `panic` controlado.
  - `fn hook_no_pane_en_teardown_fallido()` → si el teardown falla dentro del hook, no
    se encadena otro `panic`.

**Casos cubiertos**

- Normal: un `panic` inesperado deja la terminal restaurada (US-3, US-5).
- Criterio de aceptación (US-5, DoD): un `panic` en runtime deja la terminal usable y
  muestra el mensaje de error.
- Edge: fallo en el teardown tras un `panic` → se intenta reportar sin encadenar otro
  `panic`.

---

## 5. Concepto C — Bucle de eventos

### T-EVT-01 — Event loop con poll, KeyEvent y Resize

**Descripción**

- `src/terminal.rs` (o módulo del event loop): `poll` de crossterm con timeout corto.
- Procesar solo `KeyEvent` y `Resize`; ignorar el resto de eventos sin romper el bucle.
- Refrescar el frame de ratatui (tick/frame) tras cada evento relevante.

**Tests a crear**

- `mod test_event_loop`:
  - `fn keys_y_resize_se_procesan()` → de una cola simulada de eventos, solo
    `KeyEvent`/`Resize` producen acción.
  - `fn eventos_no_soportados_se_ignoran()` → eventos extra no generan ruido ni error.
  - `fn timeout_sin_evento_no_rompe_bucle()` → un poll que expira sigue el bucle.

**Casos cubiertos**

- Normal: el bucle se mantiene hasta orden de salida (US-1).
- Criterio de aceptación (DoD): el shell no recupera el control hasta salir.
- Edge: eventos desconocidos o `Resize` sin mediar tecla → render estable.

---

### T-EVT-02 — `Ctrl+C` como salida normal

**Descripción**

- `Ctrl+C` se mapea a orden de salida normal (no termina abruptamente).
- Definir `enum AppEvent { Key(KeyEvent), Resize(u16,u16), Quit }` y una función pura de
  conversión.

**Tests a crear**

- `mod test_app_event`:
  - `fn ctrl_c_es_quit()` → `Ctrl+C` genera `Quit`.
  - `fn otras_teclas_son_key()` → el resto de teclas generan `Key`.
  - `fn ctrl_c_en_no_termina_sin_teardown()` → `Quit` dispara siempre el flujo de salida.

**Casos cubiertos**

- Normal: `Ctrl+C` sale por el flujo normal (US-1, US-3).
- Criterio de aceptación (US-1): el prompt del shell vuelve tras salir.
- Edge: `Ctrl+C` durante un `poll` activo (timeout no atendido) → se procesa como salida
  normal.
- Edge: `Ctrl+C` durante el arranque → teardown seguro (US-1 edge).

---

## 6. Concepto D — Máquina de pantallas (`app.rs`)

### T-APP-01 — Enum `Screen` y estado global

**Descripción**

- `src/app.rs`:
  - `enum Screen { Unlock, List, Detail, Form, Generator, Settings }`.
  - `struct AppState { screen: Screen, selected: Option<usize>, entries: Vec<EntryDemo>,
    message: Option<String>, confirming_exit: bool, … }`.
  - Modelar también el estado vacío de `list`.

**Tests a crear**

- `mod test_app_state`:
  - `fn estado_inicial_es_unlock()`
  - `fn lista_vacia_se_detecta()` → `entries` vacío ⇒ estado vacío.

**Casos cubiertos**

- Normal: arranca siempre en `unlock` (US-1, US-2).
- Criterio de aceptación (DoD): existe representación del estado vacío y de selección.

---

### T-APP-02 — Transiciones válidas e inválidas (función pura)

**Descripción**

- Función pura `fn transition(state: AppState, event: AppEvent) -> AppEvent/Action`
  que aplica la máquina de estados de §2.5 de `DEV-FASE-1.md`:
  `unlock → list → {detail, form, generator, settings}`; hijas → `list`; `list → salir`.
- Las **transiciones no permitidas se ignoran**: sin `panic`, sin efecto visible.

**Tests a crear**

- `mod test_transitions`:
  - `fn todas_las_transiciones_validas()` → cada transición permitida cambia de estado.
  - `fn transiciones_invalidas_ignoradas()` → `Enter`/`Esc` desde `unlock`, saltos a
    `detail` desde `generator`/`settings`, etc. no cambian nada.
  - `fn teclas_no_mapeadas_ignoradas()` → teclas sin acción en cualquier pantalla.
  - `fn sin_efecto_ni_ruido()` → una transición inválida no altera mensajes ni
    selección.

**Casos cubiertos**

- Normal: navegación completa `list → {detail, form, generator, settings}` y vuelta
  (US-2, DoD).
- Criterio de aceptación (US-2): se respeta la máquina de estados y nada rompe por
  teclas no mapeadas o estados incompletos.
- Edge: re-entradas rápidas (varias teclas seguidas) no corrompen el estado.

---

### T-APP-03 — `Enter` en `list`: detalle o estado vacío

**Descripción**

- `Enter` en `list`:
  - Con selección activa → abre `detail` del elemento seleccionado.
  - Con lista vacía → muestra el mensaje informativo "Sin entradas" y **no navega**.

**Tests a crear**

- `mod test_enter_list`:
  - `fn enter_con_seleccion_abre_detail()`
  - `fn enter_con_lista_vacia_no_navega()` → permanece en `list`.
  - `fn enter_con_lista_vacia_muestra_mensaje()` → `message` = "Sin entradas".

**Casos cubiertos**

- Criterio de aceptación (US-2): con lista vacía, `Enter` no navega y muestra el estado
  vacío.
- Edge: seleccionar sin haber movido el cursor (selección inicial) → comportamiento
  definido.

---

### T-APP-04 — Atajos `n`, `g`, `e`, `t` (placeholders)

**Descripción**

- Desde `list`: `n` abre `form` (nueva), `g` abre `generator`.
- Desde `detail`: `e` abre `form` (editar).
- Ajustes: atajo `t` desde `list` abre `settings` (placeholder, propuesta de
  implementación).

**Tests a crear**

- `mod test_shortcuts`:
  - `fn n_desde_list_abre_form()`
  - `fn g_desde_list_abre_generator()`
  - `fn e_desde_detail_abre_form()`
  - `fn t_desde_list_abre_settings()`
  - `fn atajos_rechazados_en_pantalla_incorrecta()` → p. ej. `n` desde `detail` se
    ignora.

**Casos cubiertos**

- Criterio de aceptación (US-2): cada atajo funciona desde la pantalla correcta.
- Edge: atajos presionados en pantallas no permitidas → ignorados sin ruido.

---

### T-APP-05 — `Esc`/`q` en hijas conservando selección

**Descripción**

- `Esc` (y `q`) desde `detail`, `form`, `generator` o `settings` vuelve a `list`
  **conservando la selección previa**.
- `q` desde `list` inicia la confirmación de salida (no sale directo).

**Tests a crear**

- `mod test_back`:
  - `fn esc_desde_hija_vuelve_a_list()`
  - `fn seleccion_se_conserva_al_volver()` → la selección previa permanece.
  - `fn q_desde_hija_equivale_a_esc()`
  - `fn q_desde_list_inicia_confirmacion()` → no sale directo.

**Casos cubiertos**

- Criterio de aceptación (US-2): `Esc` desde hijas vuelve a `list` conservando selección.
- Edge: `q` desde pantallas hijas equivale a `Esc`.

---

## 7. Concepto E — Render y placeholders (`ui/`)

### T-UI-01 — Dispatch de render por pantalla

**Descripción**

- `src/ui/mod.rs`: función `draw(frame, state)` que delega en `screens.rs` según
  `state.screen`.
- Cada pantalla muestra su nombre en la cabecera (`[unlock]`, `[list]`, `[detail]`,
  `[form]`, `[generator]`, `[settings]`).

**Tests a crear**

- `mod test_ui_dispatch`:
  - `fn cada_pantalla_tiene_header(...)` → para los 6 estados el render incluye su
    cabecera (mediante layout/estado testeable).
  - `fn dispatch_no_panic_para_todos_los_estados()`.

**Casos cubiertos**

- Criterio de aceptación (US-2): cada pantalla tiene encabezado con su nombre y
  contenido esqueleto distinguible.

---

### T-UI-02 — Esqueletos identificables por pantalla (sin simular)

**Descripción**

- `src/ui/screens.rs`: funciones de dibujo placeholder por pantalla.
- Los placeholders **no simulan funcionalidad inexistente**: no inventar passwords, ni
  botones de copiado activos (§2.9 de `DEV-FASE-1.md`).

**Tests a crear**

- `mod test_screens`:
  - `fn placeholders_identificables()` → contenido distinguible por pantalla.
  - `fn sin_funcionalidad_falsa()` → ningún placeholder invoca copiar/generar/CRUD.

**Casos cubiertos**

- Criterio de aceptación (DoD): placeholders visibles e identificables.
- Edge: no se implementa funcionalidad fuera de alcance (§1.2).

---

### T-UI-03 — Estado vacío en `list`

**Descripción**

- Dibujo del estado vacío en `list`: mensaje "Sin entradas" + barra de atajos.

**Tests a crear**

- `mod test_empty_state`:
  - `fn lista_vacia_muestra_mensaje()`
  - `fn barra_de_atajos_visible()`

**Casos cubiertos**

- Criterio de aceptación (US-2, DoD): `list` muestra el estado vacío con mensaje
  informativo.

---

### T-UI-04 — Entries ficticias de demo (opcional y desactivable)

**Descripción**

- Pequeño set de entries ficticias solo para validar navegación/scroll, controlado por
  un flag en `AppState`.
- El estado vacío debe seguir existiendo y ser alcanzable (flag off).

**Tests a crear**

- `mod test_demo_entries`:
  - `fn con_demo_hay_entries()`
  - `fn sin_demo_estado_vacio()`

**Casos cubiertos**

- Criterio de aceptación (§2.9): la demo es opcional; el estado vacío siempre existe.

---

## 8. Concepto F — Flujo de salida

### T-EXIT-01 — Confirmación de salida desde `list`

**Descripción**

- Función pura `fn resolve_exit(key) -> ExitResolution { Confirm, Cancel, Proceed }`.
- Desde `list`, `q` muestra confirmación "¿salir? y/n"; `y` confirma, `Esc`/`n` cancela
  y la app sigue funcionando.

**Tests a crear**

- `mod test_exit_confirm`:
  - `fn q_abre_confirmacion()`
  - `fn y_confirma()`
  - `fn n_cancela_y_sigue()`
  - `fn esc_cancela_y_sigue()`
  - `fn otras_teclas_durante_confirmacion_no_la_cierran()`

**Casos cubiertos**

- Criterio de aceptación (US-3): `q` pide confirmación; `Esc`/`n` cancela.
- Edge: resize durante la confirmación → no cancela ni duplica la confirmación (US-4).

---

### T-EXIT-02 — `Ctrl+C` directo desde cualquier pantalla

**Descripción**

- `Ctrl+C` (evento `Quit`) sale directamente desde cualquier pantalla ejecutando
  teardown normal.

**Tests a crear**

- `mod test_quit`:
  - `fn ctrl_c_sale_desde_cualquier_pantalla()`
  - `fn salida_por_ctrl_c_ejecuta_teardown()` → verificación vía hook/flag de test.

**Casos cubiertos**

- Criterio de aceptación (US-3): `Ctrl+C` ejecuta el mismo teardown que la salida
  normal y termina con el código correcto.
- Edge: `Ctrl+C` durante render o evento de teclado → teardown seguro.

---

### T-EXIT-03 — Todos los caminos de salida pasan por teardown

**Descripción**

- Centralizar la salida en una única ruta que siempre invoque `teardown()`.
- Verificable mediante un hook de test que registre las invocaciones.

**Tests a crear**

- `mod test_exit_paths`:
  - `fn q_desde_list_usa_teardown()`
  - `fn esc_desde_list_usa_teardown()`
  - `fn ctrl_c_usa_teardown()`
  - `fn error_de_arranque_usa_teardown()`

**Casos cubiertos**

- Criterio de aceptación (US-3, DoD): todo camino de salida ejecuta teardown completo.

---

## 9. Concepto G — Resize y ventanas pequeñas

### T-RES-01 — Cálculo de layout mínimo y scroll (función pura)

**Descripción**

- Función pura `fn layout(size: (u16,u16), screen: Screen) -> LayoutResult`
  (en `ui/layout.rs` o `ui/mod.rs`).
- Mínimo soportado: **80×24** (§2.7 de `DEV-FASE-1.md`).
- Debajo del mínimo: redibujar con scroll sin cortar contenido de forma ilegible ni
  provocar `panic`.
- Sin espacio para todo: priorizar la región principal y mantener la barra de estado
  cuando sea físicamente posible.

**Tests a crear**

- `mod test_layout`:
  - `fn layout_minimo_80x24()`
  - `fn menor_de_80x24_usar_scroll()`
  - `fn menor_que_header_mas_barra_prioriza_principal()`
  - `fn muy_estrecho_sin_panic()`
  - `fn muy_alto_sin_panic()`
  - `fn areas_de_layout_validas()` → áreas resultantes dentro de los límites del frame
    para todos los tamaños probados.

**Casos cubiertos**

- Criterio de aceptación (US-4): sin `panic` con tamaños extremos; debajo de 80×24 hay
  scroll; contenido principal accesible.
- Edge: resize más chico que header + barra de estado → prioriza región principal, sin
  `panic`.

---

### T-RES-02 — Estado conservado al redimensionar

**Descripción**

- Al recibir `Resize`, la UI se redibuja con el nuevo tamaño **conservando el estado**
  de la pantalla (selección en `list`, mensajes, pantalla activa).
- Aplica en cualquier pantalla (`detail`, `form`, `generator`, `settings`).

**Tests a crear**

- `mod test_resize_state`:
  - `fn seleccion_se_conserva_en_resize()`
  - `fn mensaje_vacio_se_conserva_en_resize()`
  - `fn pantalla_activa_se_mantiene_en_resize()`
  - `fn resize_en_pantalla_hija_no_panic()`
  - `fn resize_frecuente_render_estable()` → serie de `Resize` sin error ni corrupción.

**Casos cubiertos**

- Criterio de aceptación (US-4): el estado se conserva al redimensionar; resize funciona
  en cualquier pantalla.
- Edge: `Resize` muy frecuente → render estable.
- Edge: resize durante confirmación de salida → no cancela ni duplica (enlazado a
  T-EXIT-01).

---

## 10. Concepto H — Errores de arranque y exit codes

### T-ERR-ST-01 — stdout no-TTY

**Descripción**

- Detectar en `main.rs` que `stdout` no es un TTY (redirección/CI) → mensaje claro en
  `stderr` (p. ej. "no es una terminal interactiva"), salida con exit code ≠ 0, sin
  `panic` y sin alterar la terminal.

**Tests a crear**

- `mod test_startup` (`#[ignore]` o prueba headless con output redirigido):
  - `fn stdout_no_tty_es_error()`
  - `fn mensaje_via_stderr()`
  - `fn exit_distinto_de_cero()`
  - `fn sin_panic()`
  - `fn stdout_y_stderr_en_pipes_lo_mismo()` → mensaje siempre a `stderr`.

**Casos cubiertos**

- Criterio de aceptación (US-1, US-5, DoD): no-TTY → mensaje claro en `stderr`, exit ≠ 0.
- Edge: CI sin TTY → exit code correcto para no colgar.
- Edge: `stdout` y `stderr` en contextos distintos (pipes).

---

### T-ERR-ST-02 — `$TERM` ausente o inadecuado

**Descripción**

- **Decisión documentada e implementada coherentemente**: ante `$TERM` ausente/vacío se
  muestra un **warning claro y se continúa** (la app funciona con los defaults).
  La decisión queda comentada/documentada en el código (§2.8 y DoD).

**Tests a crear**

- `mod test_term_env`:
  - `fn term_ausente_emite_warning()` → warning cuando la variable falta.
  - `fn term_ausente_continua()` → no aborta el arranque.
  - `fn term_erroneo_mismo_comportamiento()` → valor corrupto trata igual.

**Casos cubiertos**

- Criterio de aceptación (US-5, DoD): comportamiento definido (warning + continuar),
  implementado de forma coherente y documentado.
- Edge: `$TERM` vacío/corrupto.

---

### T-ERR-ST-03 — Fallo de raw mode / alternate screen y teardown

**Descripción**

- Fallo al habilitar raw mode o entrar a alternate screen → mensaje en `stderr`, exit ≠ 0,
  terminal sin alterar.
- Error en teardown → no enmascara la salida principal; se reporta si es posible.

**Tests a crear**

- `mod test_startup_terminal`:
  - `fn fallo_raw_mode_es_error_de_terminal()` → categoría terminal → exit `2`.
  - `fn fallo_alternate_screen_es_error_de_terminal()`
  - `fn fallo_no_deja_terminal_alterada()` → estado del guard sin limpieza pendiente.
  - `fn fallo_en_teardown_no_enmascara_salida_principal()`

**Casos cubiertos**

- Criterio de aceptación (US-5): en ningún caso de arranque erróneo se deja el terminal
  en modo raw.
- Edge: fallo en el teardown tras `panic` → se reporta sin encadenar otro `panic`
  (vinculado a T-TERM-03).

---

## 11. Orden de ejecución sugerido

| # | Tarea | Depende de |
|---|---|---|
| 1 | T-SK-01, T-SK-02 | — |
| 2 | T-ERR-01, T-ERR-02 | T-SK-01 |
| 3 | T-TERM-01, T-TERM-02, T-TERM-03 | T-ERR-01 |
| 4 | T-APP-01, T-APP-02, T-APP-03, T-APP-04, T-APP-05 | T-SK-01 |
| 5 | T-EVT-01, T-EVT-02 | T-TERM-01 |
| 6 | T-UI-01, T-UI-02, T-UI-03, T-UI-04 | T-APP-01..05 |
| 7 | T-EXIT-01, T-EXIT-02, T-EXIT-03 | T-APP-05, T-EVT-02 |
| 8 | T-RES-01, T-RES-02 | T-UI-01 |
| 9 | T-ERR-ST-01, T-ERR-ST-02, T-ERR-ST-03 | T-ERR-02, T-TERM-01 |

Racional: la máquina de pantallas (Concepto D) es el núcleo testeable y no depende de
terminal; se desarrolla antes que el render. Los errores de arranque (Concepto H) se
cierran al final para no fijar exit codes prematuramente.

## 12. Mapa con el Definition of Done de `DEV-FASE-1.md`

| DoD (§4) | Tareas que lo satisfacen |
|---|---|
| `cargo run` inicia el TUI, muestra `unlock`, shell no recupera control | T-SK-02, T-EVT-01 |
| Navegación `unlock → list → {detail, form, generator, settings}` con placeholders | T-APP-01..05, T-UI-01, T-UI-02 |
| `Esc`/`q`/`Ctrl+C` restauran siempre la terminal | T-TERM-02, T-EXIT-01..03 |
| Resize y terminales < 80×24 sin `panic` | T-RES-01, T-RES-02 |
| Errores de arranque: mensajes legibles y exit ≠ 0 | T-ERR-01, T-ERR-02, T-ERR-ST-01..03 |
| `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test` en verde | todos (DoD por tarea), explícito T-SK-01 |
| Sin funcionalidad fuera de alcance (§1.2) | T-SK-01, T-UI-02 |
| Decisión de `$TERM` documentada | T-ERR-ST-02 |