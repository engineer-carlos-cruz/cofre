# DEV-FASE-1 — Definición: Fundación del proyecto (Skeleton)

> Definición de lo primero que se debe desarrollar para **Cofre**.
> Derivada de `docs/PRD.md` (M1 Skeleton) y de `docs/SPEC.md`.
> Estado: **definido**. Las tareas de implementación se detallarán en una etapa posterior.

## 1. Objetivo y alcance

La primera fase de desarrollo de Cofre consiste en construir el **esqueleto funcional
del proyecto**: la estructura del crate, el arranque de la terminal TUI
(`ratatui` + `crossterm`), la máquina de pantallas y la navegación entre las seis
vistas del producto (**placeholder**), la restauración limpia de la terminal al salir
y el manejo base de errores y de tamaños de ventana.

Esta fase **no implementa ninguna funcionalidad de negocio**: no hay cifrado, no hay
archivo `.cofre`, no hay CRUD de credenciales, ni generador, ni portapapeles, ni
auto-lock. Todas las pantallas muestran contenido esqueleto
identificable para validar la estructura y la navegación antes de integrar lógica.

### 1.1 Incluido en esta fase

- Creación del proyecto Cargo con la estructura de módulos propuesta.
- Inicialización y destrucción (teardown) del terminal TUI.
- Bucle de eventos de teclado (event loop).
- Máquina de pantallas con los tres módulos base: `unlock`, `list`, y los flujos
  `detail`, `form`, `generator`, `settings`.
- Navegación por teclado entre las pantallas (atajos básicos).
- Manejo de ventanas pequeñas (< 80×24) y resize en caliente.
- Confirmación visual de salida desde `list` y comportamiento `Esc`/`q` en pantallas hijas.
- Salida limpia con `q`, `Esc`, `Ctrl+C` y restauración del terminal.
- Manejo de errores de arranque con mensaje legible y exit code ≠ 0.
- Mensaje informativo en listado vacío.

### 1.2 Excluido de esta fase (fases posteriores)

- Cifrado (Argon2id + XChaCha20-Poly1305) y formato `.cofre` → M2.
- Persistencia (lectura/escritura del archivo) → M2.
- CRUD real de entradas (crear, editar, borrar, ver detalles) → M2.
- Validación de master password y creación de vault → M2.
- Generador de contraseñas, búsqueda/filtros, portapapeles, auto-lock → M3.
- Settings funcionales y cambio de master password → M4.

## 2. Requerimientos detallados

### 2.1 Proyecto Cargo

- Crate binario `cofre`, Rust edition **2021+**.
- Dependencias para esta fase:
  - `crossterm` (backend de terminal, eventos).
  - `ratatui` (framework TUI).
  - Sin dependencias de crypto, almacenamiento ni portapapeles aún.
- Versiones: fijar las versiones estables actuales en el bloqueo al momento de
  implementar; deben ser compatibles entre sí (`ratatui` + `crossterm`).
- Perfil por defecto (`dev`) usables para desarrollo; no se requiere optimización
  específica en esta fase.

### 2.2 Estructura de módulos

```text
cofre/
├── Cargo.toml
└── src/
    ├── main.rs            # Entry point, orquestación init/teardown
    ├── app.rs             # Estado global, máquina de pantallas (enum Screen)
    ├── terminal.rs        # Setup ratatui/crossterm, rodaje de raw mode, teardown
    ├── ui/
    │   ├── mod.rs         # Render del frame según la pantalla activa
    │   └── screens.rs     # Funciones de dibujo por pantalla (placeholder)
    └── errors.rs          # Tipos de error base de la aplicación
```

- Los módulos de negocio del SPEC §1 (`crypto`, `storage`, `models`, `password`,
  `clipboard`, `config`) **no se crean aún**; se añadirán en sus fases.
- Separación clara: `app.rs` modela el estado; `terminal.rs` gestiona el ciclo de
  vida de ratatui/crossterm; `ui/` sólo dibuja; `main.rs` los orquesta.

### 2.3 Ciclo de vida de la terminal (init / teardown)

- **Init**:
  - Envolver el directorio de trabajo previo de la terminal (enable raw mode).
  - Entrar en la pantalla alternativa (alternate screen).
  - Ocultar el cursor durante el render.
  - Si cualquier paso de init falla: mensaje claro en `stderr`, salida con exit code ≠ 0,
    **sin dejar** la terminal en modo raw.
- **Teardown**:
  - Mostrar el cursor, salir de la pantalla alternativa y desactivar raw mode.
  - Debe ser **idempotente** (ejecutarlo dos veces no produce errores ni panics).
  - Se ejecuta en la salida normal **y** en la salida por error/panic (hook de panic
    que hace teardown antes de re-propagar o mostrar el error).

### 2.4 Bucle de eventos

- `poll` de crossterm con timeout corto; por cada evento de tecla se re-arma la
  contabilidad que el ratatui necesita (tick/frame).
- En esta fase solo se procesan eventos de teclado (`KeyEvent`) y de cambio de
  tamaño (`Resize`). Otros eventos se ignoran sin romper el bucle.
- `Ctrl+C` tratado como orden de salida normal (no terminate abruptamente).

### 2.5 Máquina de pantallas

Estados y transiciones permitidas:

```text
unlock -> list -> detail -> form
                -> generator
                -> settings
list  -> salir (q)
cualquiera -> list (Esc / q en pantalla hija)
cualquiera -> teardown y salir (q desde list, o Ctrl+C)
```

| Pantalla | Rol en esta fase |
|---|---|
| `unlock` | Pantalla inicial; esqueleto con campo de master password (no funcional) y nota "vault" según exista o no archivo |
| `list` | Esqueleto: título, barra de búsqueda placeholder, región de entradas (ficticias para demo o vacía), barra de atajos |
| `detail` | Esqueleto del detalle de la entrada seleccionada en `list` |
| `form` | Esqueleto de formulario de creación/edición |
| `generator` | Esqueleto del generador |
| `settings` | Esqueleto de ajustes |

- Reglas de transición:
  - Las transiciones no permitidas se **ignoran** (no panic, no efecto visible).
  - `Enter` en `list` con selección abre `detail`. Con lista vacía muestra el estado
    vacío (mensaje informativo) y no navega.
  - `n` desde `list` abre `form` (acción "nueva").
  - `e` desde `detail` abre `form` (acción "editar"). (Ambos son placeholders).
  - `g` desde `list` abre `generator`.
  - Desde `settings` se accede por atajo predefinido (a definir en implementación;
    sugerido `t`), placeholder.
  - `Esc` desde una pantalla hija (`detail`, `form`, `generator`, `settings`) vuelve
    a `list` y **conserva la selección**.
  - `q` desde `list` inicia el flujo de salida (ver §2.6). `q` desde pantallas hijas
    equivale a `Esc`.

### 2.6 Salida

- Desde `list`, `q` confirma visualmente la salida (mensaje o estado "¿salir? y/n").
  Se puede cancelar con `Esc` o `n`.
- `Ctrl+C` sale directamente desde cualquier pantalla, resto del teardown normal.
- Todo camino de salida ejecuta el teardown completo (§2.3).

### 2.7 Tamaños de ventana y resize

- Tamaño mínimo soportado: **80×24** (SPEC §6).
- Debajo del mínimo, cada pantalla **debe** redibujarse con scroll y sin cortar
  contenido de forma ilegible ni provocar panic.
- Al recibir evento `Resize`, el frame de ratatui se redibuja con el nuevo tamaño.
- El estado de la pantalla (incluida la selección y el mensaje vacío) se conserva
  durante un resize.
- Rangos extremos (ventana muy estrecha o muy alta) no deben romper el layout; si no
  hay espacio para todo, priorizar la región principal y mantener la barra de estado
  cuando sea físicamente posible.

### 2.8 Errores de arranque y exit codes

| Caso | Comportamiento |
|---|---|
| stdout no es un TTY (redirección/CI) | Mensaje legible en `stderr`, exit ≠ 0 |
| `$TERM` ausente o inadecuado | Mensaje/warning claro; decisión de fallar o continuar documentada e implementada coherentemente |
| Fallo al enable raw mode o al entrar a alternate screen | Mensaje en `stderr`, exit ≠ 0, terminal sin alterar |
| Error en teardown | No enmascara la salida principal; se reporta si es posible |

- Nunca `panic` por estas condiciones; se usan tipos `Result` de `errors.rs`.
- Exit codes: `0` salida normal; `1` error de arranque; codes diferenciados para
  errores de terminal (a convenir en implementación).

### 2.9 Mensajes y placeholders

- Cada pantalla muestra su nombre en la cabecera (ej. `[list]`, `[detail]`).
- Los placeholders deben ser identificables pero **no** simular funcionalidad
  inexistente (no inventar passwords, ni botones de copiado activos).
- `list` puede mostrar un pequeño set de entradas ficticias de demostración **solo**
  si ayuda a validar la navegación/scroll; debe existir también el estado vacío.

## 3. Historias de usuario

### US-1 — Iniciar la aplicación

> **Como** usuario,
> **quiero** ejecutar `cofre` desde la terminal,
> **para** que se abra la interfaz TUI mostrando la pantalla de desbloqueo y el
> shell no recupere el control hasta que yo salga.

**Detalle:** es el arranque de la aplicación. Debe inicializar la terminal en modo
raw sobre una pantalla alternativa, renderizar la primera pantalla (`unlock`) de
forma inmediata y mantener el bucle de eventos activo hasta una orden de salida.
El prompt del shell sólo reaparece tras la salida.

**Criterios de aceptación:**

- [ ] Ejecutar `cargo run` (o el binario `cofre`) muestra a pantalla completa un
      TUI con el encabezado de la pantalla `unlock` y un campo de master password.
- [ ] El prompt del shell no regresa hasta que el usuario sale con `q`, `Esc` o
      `Ctrl+C`.
- [ ] La pantalla inicial es siempre `unlock`, con la variante "nuevo vault" o
      "master password" según exista o no un archivo `.cofre` (por metadatos
      placeholder; **sin** leer el archivo).
- [ ] Si el arranque falla, se imprime un mensaje claro en `stderr` y se sale con
      exit code ≠ 0 sin dejar la terminal alterada.

**Edge cases:**

- Arranque en un terminal sin soporte TTY (stdout redirigido) → error claro, exit ≠ 0,
  sin `panic`.
- `Ctrl+C` presionado durante el arranque (antes de completar el init) → teardown
  seguro, terminal restaurada, exit code adecuado.
- `term` sin valor → warning claro; comportamiento coherente (fallar con mensaje o
  continuar), documentado en el código.
- Doble ejecución/secuestro de la terminal previa → sin efectos visibles (se
  restaura correctamente al salir).

### US-2 — Navegar entre pantallas

> **Como** usuario,
> **quiero** recorrer `list → detail → form`, `generator` y `settings` y volver a la
> lista,
> **para** validar la estructura del producto y la experiencia de navegación antes de
> implementar la lógica interna de cada módulo.

**Detalle:** todas las pantallas existen como esqueletos identificables. La
navegación con el teclado debe respetar la máquina de estados (§2.5) y no romper por
teclas no mapeadas o por estados incompletos (ej. entrar a `detail` sin selección).

**Criterios de aceptación:**

- [ ] Desde `list`, `↑/↓` (o `j/k`) cambian la selección y `Enter` abre `detail` de
      la entrada seleccionada.
- [ ] Desde `list`, `n` abre `form` (nueva entrada) y `g` abre `generator`.
- [ ] Desde `detail`, `e` abre `form` (edición).
- [ ] `Esc` desde `detail`, `form`, `generator` o `settings` vuelve a `list`
      conservando la selección previa.
- [ ] `q` desde `list` inicia la confirmación de salida; `q` desde pantallas hijas
      vuelve a `list`.
- [ ] Con la lista vacía, `Enter` no navega y muestra el mensaje informativo
      "Sin entradas" (estado vacío).
- [ ] Cada pantalla tiene un encabezado con su nombre (`[list]`, `[detail]`,
      `[form]`, `[generator]`, `[settings]`) y contenido esqueleto distinguible.

**Edge cases:**

- `Enter` con lista vacía → mensaje de estado vacío, sin cambio de pantalla, sin
  `panic`.
- Teclas no mapeadas en cualquier pantalla → se ignoran sin efecto ni ruido.
- Transiciones prohibidas (ej. `Enter`/`Esc` intentando saltar de `unlock` a
  pantallas hijas) → se ignoran.
- Re-entradas rápidas (varias teclas seguidas) → el estado no se corrompe.
- Navegar de `generator`/`settings` a `detail` no es posible (no existe en la
  máquina de estados) — se ignora.

### US-3 — Salir y restaurar la terminal

> **Como** usuario,
> **quiero** salir con `q`, `Esc` o `Ctrl+C` y que la terminal quede exactamente como
> estaba,
> **para** no perder mi sesión de shell ni dejar artefactos visuales.

**Detalle:** el teardown (§2.3) restaura cursor, pantalla alternativa y modo cooked.
Debe ejecutarse en todos los caminos de salida, incluyendo la interrupción por
`Ctrl+C` y cualquier salida por error/`panic`.

**Criterios de aceptación:**

- [ ] Tras salir, el prompt del shell funciona normalmente: sin modo raw residual, sin
      cursor oculto, sin caracteres fantasma en pantalla.
- [ ] `Ctrl+C` en cualquier pantalla ejecuta el mismo teardown que la salida normal y
      termina el proceso con código de salida correcto.
- [ ] El teardown es idempotente: invocarlo dos veces consecutivas no produce error
      ni `panic`.
- [ ] Ante una salida por error o `panic`, se restaura la terminal antes de propagar
      el mensaje.
- [ ] Confirmación de salida desde `list`: `q` pide confirmación; `Esc`/`n` cancela y
      la aplicación sigue funcionando.

**Edge cases:**

- `Ctrl+C` durante render o durante un evento de teclado → teardown seguro.
- Salida con contenido parcial en pantalla (render interrumpido) → sin artefactos.
- `Ctrl+C` mientras hay un `poll` activo (timeout no atendido) → se procesa como
  salida normal.
- Si un error `panic` se produce estando dentro del raw mode → el hook de panic hace
  teardown y deja el terminal usable, con el mensaje de panic visible.

### US-4 — Terminal pequeña y cambio de tamaño (resize)

> **Como** usuario,
> **quiero** que la interfaz se adapte al tamaño de mi ventana y no se rompa en
> terminales menores de 80×24,
> **para** poder usarla en entornos reducidos o al redimensionar la ventana.

**Detalle:** el layout mínimo es 80×24; por debajo se usa scroll. Al recibir un
evento de `Resize`, la pantalla se redibuja con el nuevo tamaño sin perder el estado
(selección, mensajes) y sin `panic`.

**Criterios de aceptación:**

- [ ] Ningún `panic` con tamaños extremos (muy estrecho, muy alto, muy pequeño).
- [ ] Al cambiar el tamaño de la ventana, la UI se redibuja limpia (sin glitches) y
      mantiene la pantalla activa.
- [ ] Debajo de 80×24 se muestra scroll; el contenido principal permanece accesible y
      la barra de estado se mantiene si físicamente hay espacio.
- [ ] El estado de la pantalla (selección en `list`, mensaje de estado vacío) se
      conserva al redimensionar.
- [ ] El resize funciona en cualquier pantalla (incluidas `detail`, `form`,
      `generator`, `settings`).

**Edge cases:**

- Resize a un tamaño más chico que el encabezado + barra de estado → la interfaz
  prioriza la región principal; sin `panic`.
- Resize muy frecuente o eventos `Resize` sin mediar tecla → render estable.
- Resize durante la confirmación de salida → no cancela ni duplica la confirmación.

### US-5 — Errores de arranque claros

> **Como** usuario,
> **quiero** ver un mensaje de error claro y salir con un código de error adecuado si
> la terminal no está disponible o falla el arranque,
> **para** entender qué pasó, en lugar de un cierre abrupto.

**Detalle:** los fallos de inicialización (no-TTY, `$TERM` inadecuado, fallo de raw
mode/alternate screen, etc.) se reportan por `stderr` con un mensaje legible y se
sale con exit code ≠ 0 sin `panic`.

**Criterios de aceptación:**

- [ ] Ejecutar con stdout no-TTY muestra un mensaje claro en `stderr` y exit code ≠ 0.
- [ ] El mensaje identifica la causa (ej. "no es una terminal interactiva") y sugiere
      el remedio si aplica.
- [ ] En ningún caso de arranque erróneo se deja el terminal en modo raw.
- [ ] La salida por error usa los códigos definidos (§2.8); no se traga el error con
      exit 0.
- [ ] Un `panic` inesperado en tiempo de ejecución deja la terminal restaurada y
      muestra el mensaje de error sin romper el shell.

**Edge cases:**

- `stdout` y `stderr` en contextos distintos (pipes) → mensaje siempre a `stderr`.
- `$TERM` vacío/corrupto → comportamiento definido (fallo claro con mensaje o
      warning + continuar), implementado de forma coherente.
- Fallo en el teardown tras un `panic` → se intenta reportar sin encadenar otro
      `panic`.
- Permisos/entorno de CI (sin TTY) → exit code correcto para que el CI no cuelgue
      esperando input.

## 4. Definition of Done (DoD)

La fase 1 se considera terminada cuando:

- [ ] `cargo run` inicia el TUI, muestra la pantalla `unlock` y el shell no recupera
      control hasta salir.
- [ ] Es posible recorrer `unlock → list → {detail, form, generator, settings}` y
      volver, con los placeholders visibles e identificables.
- [ ] `Esc`/`q`/`Ctrl+C` restauran siempre la terminal (cursor visible, modo cooked,
      pantalla alternativa restaurada, sin artefactos).
- [ ] Resize y terminales menores de 80×24 no producen `panic`.
- [ ] Los casos de error de arranque muestran mensajes legibles y exit code ≠ 0.
- [ ] `cargo fmt --check` pasa sin diferencias.
- [ ] `cargo clippy -- -D warnings` no reporta `warnings`.
- [ ] `cargo test` pasa, incluyendo unit tests de la máquina de pantallas
      (transiciones válidas e inválidas) y del cálculo de layout mínimo/scroll.
- [ ] No se implementa funcionalidad fuera de alcance (§1.2): nada de crypto,
      storage, CRUD real, generador, portapapeles ni auto-lock.
- [ ] Se documenta la decisión de comportamiento ante `$TERM` ausente (fallar o
      continuar) en el código.

## 5. Fase siguiente (preview, fuera de esta entrega)

Tras validar el skeleton, la siguiente fase corresponde a **M2 — Crypto & storage**
del PRD: derivación de clave Argon2id, cifrado XChaCha20-Poly1305, formato `.cofre`,
creación/apertura de vault y CRUD de entradas, según `docs/SPEC.md` §§3–4.1–4.2.

## 6. Nota sobre tareas

Este documento define **qué** se desarrolla (requerimientos, historias de usuario con
criterios de aceptación y edge cases) y **cuándo está terminado** (DoD). El desglose
en **tareas de implementación** (epics, subtareas, estimaciones y orden de trabajo) se
detallará en un documento de planificación posterior.