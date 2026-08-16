# DEV-FASE-3 — Definición: Experiencia de usuario (UX)

> Definición de la tercera fase de desarrollo de **Cofre**.
> Derivada de `docs/PRD.md` (M3 UX) y de `docs/SPEC.md`.
> Estado: **definido**. Las tareas de implementación se detallarán en una etapa posterior.

## 1. Objetivo y alcance

La tercera fase de desarrollo de Cofre convierte el **CRUD funcional** de la fase 2
en una aplicación con las **funciones de confort de UX** del producto: generador de
contraseñas con análisis de fortaleza, búsqueda incremental y filtrado por tag,
portapapeles con auto-clear y auto-lock por inactividad.

Esta fase **no implementa aún** la pantalla de ajustes funcional, el cambio de master
password ni la exigencia de master password al borrar: todo eso corresponde a M4
(fase 4). La pantalla `generator` de las fases anteriores se hace **funcional**; las
pantallas `unlock`, `list` y `detail` se extienden con búsqueda, filtros y acciones de
portapapeles, manteniendo el cifrado, el almacenamiento y el CRUD de la fase 2.

### 1.1 Incluido en esta fase

- Módulos nuevos de negocio: `password.rs` (generador + análisis de fortaleza) y
  `clipboard.rs` (wrapper de `arboard` + auto-clear); extensión de `app.rs` (estado de
  búsqueda, filtros y contador de inactividad) y de `ui/` (pantalla `generator`
  funcional, barra de búsqueda y tags en `list`, indicador de copiado).
- Dependencia nueva: `arboard` (portapapeles, con fallback headless). `rand` ya está
  disponible desde la fase 2 para `OsRng`.
- **Generador de contraseñas** (SPEC §4.3): pantalla `generator` funcional con opciones
  en vivo (largo 4–128 con defecto 20, toggles de charsets, evitar ambiguos, al menos
  una de cada clase), regeneración con `r`, copia con `Enter` y guardado como nueva
  entry con `s`.
- **Análisis de fortaleza** (SPEC §7): estimación por entropía (`pool_size ^ length`),
  clasificación débil/media/fuerte y visual de barra + colores, sin diccionarios.
- **Búsqueda y filtrado** (SPEC §4.4): input incremental con `/` en `list`, campos
  título/username/URL/tags (case-insensitive, substring), prefijos `t:`/`u:` y filtro
  por tag con `f` combinable con la búsqueda.
- **Portapapeles** (SPEC §4.5): `c` copia password y `C` (shift) copia username desde
  `list`/`detail`; indicador visual "copied" con cuenta atrás; auto-clear tras
  `clipboard_seconds` (defecto 15 s) y **siempre** al salir o lockear.
- **Auto-lock** (SPEC §4.6): contador de inactividad reseteado con cualquier tecla,
  transición a `unlock` al alcanzar `auto_lock_minutes` (defecto 5), descarte con
  `zeroize` de la clave y la payload descifrada.
- Integración del generador con el CRUD de fase 2: `s` abre `form` pre-cargado con la
  contraseña generada; el guardado persiste con el ciclo de fase 2.
- Tests: garantías del generador por charset/ambigüedad/"una de cada clase", estimador
  de fortaleza, lógica de búsqueda/filtros (incluidos prefijos), auto-clear del
  portapapeles y auto-lock por inactividad.

### 1.2 Excluido de esta fase (fases posteriores)

- Pantalla `settings` funcional y cambio de master password → M4 (fase 4).
- `require_password_on_delete` (depende de settings) → M4 (fase 4); el borrado se
  confirma inline con `y`/`n` como en fase 2.
- Detección de contraseñas débiles/reutilizadas (basada en diccionarios) → extensiones
  futuras del PRD.
- Import/export (CSV/kdbx) y TOTP/2FA → extensiones futuras del PRD.
- Migraciones de versión del formato (solo se soporta `version = 1`).
- Fuzzing del parseo binario (nice-to-have del SPEC §11, fuera de esta fase).

## 2. Requerimientos detallados

### 2.1 Dependencias nuevas

Se añade a las de fases 1 y 2 (`crossterm`, `ratatui`, `argon2`, `chacha20poly1305`,
`rand`, `zeroize`, `serde`/`serde_json`, `uuid`, `time`):

| Crate | Uso |
|---|---|
| `arboard` | Portapapeles del sistema con fallback si no hay display |

- Versión estable actual, fijada en el lock al implementar; compatible con las
  dependencias previas.
- Ninguna dependencia de red; todo es local.

### 2.2 Estructura de módulos

```text
cofre/
├── Cargo.toml
├── src/
│   ├── main.rs            # Entry point, orquestación init/teardown (fase 1)
│   ├── app.rs             # Estado global + máquina de pantallas (extendido)
│   ├── terminal.rs        # Setup ratatui/crossterm (fase 1)
│   ├── ui/                # Render por pantalla (extendido: generator, búsqueda, tags)
│   ├── crypto.rs          # Argon2id + XChaCha20-Poly1305 + formato (fase 2)
│   ├── storage.rs         # Lectura/escritura del archivo .cofre (fase 2)
│   ├── models.rs          # Entry, Vault, Settings (fase 2)
│   ├── password.rs        # Generador + análisis de fortaleza (nuevo)
│   ├── clipboard.rs       # Wrapper arboard + auto-clear (nuevo)
│   └── errors.rs          # Tipos de error (extendido)
└── tests/
    └── fixtures/          # Archivos .cofre de prueba (fase 2)
```

- `password.rs` es **lógica pura** (sin TTY): recibe opciones y devuelve contraseña /
  estimación; `clipboard.rs` encapsula solo la interacción con el portapapeles del
  sistema; `app.rs`/`ui/` orquestan el estado (búsqueda, filtros, copiado, inactividad).
- La separación de responsabilidades de fases 1 y 2 se mantiene; no se acopla
  crypto/storage a UI.

### 2.3 Generador de contraseñas (`password.rs`)

- `generate_password(options: PasswordOptions) -> Result<String>` construye el pool
  según los charsets activos y baraja con `OsRng` (Fisher-Yates).
- Opciones (SPEC §4.3):
  - **Largo**: 4–128, defecto **20**; fuera de rango → error de validación, no `panic`.
  - **Charsets** (toggle, al menos uno activo): minúsculas, mayúsculas, dígitos y
    símbolos `!@#$%^&*()-_=+[]{};:,.?`. Si ninguno está activo → error legible.
  - **Evitar ambiguos**: elimina `0O1lI` del pool.
  - **Al menos una de cada clase**: garantiza 1 carácter de cada clase activa y rellena
    el resto del pool. Si solo hay una clase activa, se cumple trivialmente.
  - Regla de orden: la garantía "una de cada clase" se resuelve antes de filtrar los
    ambiguos, para que no pueda fallar por un charset quedarse sin caracteres.
- El resultado se regenera **en vivo**: al cambiar cualquier opción o al pulsar `r`.
- La contraseña generada se trata como dato sensible (ver §2.10).

### 2.4 Análisis de fortaleza (SPEC §7)

- `estimate_strength(password) -> Strength { weak, medium, strong }` basado en
  entropía estimada `pool_size ^ length` (pool estimado por los caracteres presentes).
- Clasificación: **débil** (< ~48 bits), **media** (48–80), **fuerte** (> 80 bits).
- Visual en el TUI: barra `▓▓▓▓░░░░` + color + texto. No depende de diccionarios.
- Acompaña al resultado del generador (§2.5) y se recalcula con cada regeneración.

### 2.5 Pantalla `generator`

- Opciones en vivo: largo (input/stepper), toggles de los 4 charsets, "evitar
  ambiguos" y "al menos una de cada clase". Cada cambio regenera.
- Acciones:
  - `Enter` → copiar al portapapeles (con auto-clear, §2.7) + indicador "copied".
  - `s` → abrir `form` pre-cargado con la contraseña generada (integra §2.9).
  - `r` → regenerar.
  - `Esc`/`q` → volver a `list` conservando la selección (máquina de fases 1–2).
- Nota de fortaleza visible junto al resultado.

### 2.6 Búsqueda y filtrado

- `/` en `list` abre un input de búsqueda **incremental**: cada pulsación filtra en
  vivo sobre las entries en memoria.
- Campos buscados: **título, username, URL, tags** (case-insensitive, substring).
- Prefijos: `t:` filtra por tag ("t:dev"), `u:` por username ("u:GitHub").
- Filtro por tag: barra de tags en `list`; `f` selecciona/des-selecciona un tag.
  **Combinable** con la búsqueda.
- Al filtrar, la selección se re-posiciona (primer resultado o estado vacío filtrado);
  sin resultados se muestra un mensaje informativo (no "Sin entradas" del estado
  vacío global). `Esc`/borrar el input restaura la lista completa.
- Rendimiento: listado/filtrado sobre 10.000 entries < 50 ms (SPEC §9).

### 2.7 Portapapeles (`clipboard.rs`)

- Wrapper de `arboard`:
  - `c` copia la **password** y `C` (shift) copia el **username**, desde `list` y
    `detail` (SPEC §4.5, §5).
  - Tras copiar: indicador visual "copied" + cuenta atrás del auto-clear.
- **Auto-clear** tras `clipboard_seconds` (defecto 15 s, desde `Settings` de la
  payload), y **siempre** al salir o al lockear (incluido el lock por auto-lock).
- Si la API del portapapeles no está disponible (entorno headless) → la función se
  **desactiva con aviso** al usuario (SPEC §8), sin romper el resto de la app.

### 2.8 Auto-lock

- Contador de inactividad: se resetea con **cualquier tecla**, incluidos los inputs de
  texto de `form`/búsqueda (regla simple del SPEC §4.6: se cuenta cualquier
  inactividad real de teclas).
- Solo corre mientras la sesión está desbloqueada.
- Al alcanzar `auto_lock_minutes` (defecto 5, desde `Settings`) → transición a
  pantalla `unlock` **sin cerrar la app**.
- En el lock:
  - Se descartan de memoria la clave derivada y la payload descifrada (`zeroize`).
  - Se limpia el portapapeles (aunque la copia siguiera en curso).
- El backoff de 3 intentos de la fase 2 se conserva y no colisiona con el auto-lock.

### 2.9 Integración con el CRUD de fase 2

- `s` desde `generator` abre `form` (nueva entry) con el campo `password` pre-cargado;
  el usuario completa `title`/`username` y guarda con el ciclo de guardado de fase 2
  (§2.8 de `DEV-FASE-2.md`).
- `Esc` desde ese `form` descarta sin persistir.
- No se modifican `crypto`, `storage` ni `models`; se reutilizan tal cual.

### 2.10 Seguridad de memoria

- La contraseña generada, las passwords copiadas y los buffers de búsqueda que
  contienen datos de entries se tratan como sensibles: `zeroize` al descartarlos.
- La clave derivada y la payload viven solo en RAM (SPEC §9), ya garantizado en fase 2.
- No hay logs ni volcados con contraseñas en claro.

### 2.11 Requisitos no funcionales

- Sin red, sin telemetría, sin logs de contraseñas (SPEC §9).
- Rendimiento: búsqueda/filtrado sobre 10.000 entries < 50 ms (SPEC §9).
- El copiado/auto-clear no bloquea el render (se gestiona en el event loop).

## 3. Historias de usuario

### US-1 — Generar contraseñas con opciones en vivo

> **Como** usuario,
> **quiero** generar contraseñas configurando largo y charsets y verlas regenerarse al
> instante,
> **para** crear credenciales seguras según mis necesidades.

**Detalle:** la pantalla `generator` muestra las opciones del SPEC §4.3 y regenera el
resultado en vivo con cada cambio o con `r`. `Enter` copia y `s` guarda como nueva
entry.

**Criterios de aceptación:**

- [ ] `g` desde `list` abre `generator` con las opciones por defecto (largo 20, los 4
      charsets activos) y un resultado válido.
- [ ] Cambiar el largo (4–128) o un charset regenera el resultado de inmediato.
- [ ] Con "evitar ambiguos" activo, el resultado no contiene `0O1lI`.
- [ ] Con "al menos una de cada clase" activo, el resultado contiene al menos un
      carácter de cada clase activa.
- [ ] `r` regenera un resultado nuevo; `Enter` copia al portapapeles con indicador.
- [ ] `Esc` desde `generator` vuelve a `list` conservando la selección.

**Edge cases:**

- Largo fuera de rango (0, 3, 129) → error de validación legible, sin `panic`.
- Ningún charset activo → error legible, no genera nada.
- "Al menos una de cada clase" con una sola clase activa → se cumple trivialmente.
- "Al menos una de cada clase" + "evitar ambiguos" → nunca falla (la garantía se
  resuelve antes de filtrar ambiguos).
- Teclas no mapeadas en `generator` → se ignoran sin efecto ni ruido.

### US-2 — Evaluar la fortaleza de una contraseña

> **Como** usuario,
> **quiero** ver una estimación de la fortaleza de la contraseña generada,
> **para** decidir si es suficientemente segura antes de usarla.

**Detalle:** el generador acompaña el resultado con el análisis del SPEC §7
(entropía estimada `pool_size ^ length`, clasificación débil/media/fuerte y barra
visual), recalculado en cada regeneración.

**Criterios de aceptación:**

- [ ] El generador muestra clasificación + barra visual para cualquier resultado.
- [ ] La clasificación se recalcula al regenerar o cambiar opciones.
- [ ] Una contraseña muy corta con un solo charset se clasifica como **débil**.
- [ ] La estimación no depende de diccionarios (solo de pool y largo).

**Edge cases:**

- Password vacía → no se evalúa ni se muestra (o se muestra estado neutro), sin error.
- Largo máximo (128) con varios charsets → clasificado fuerte, sin desbordamiento en el
  cálculo de entropía.

### US-3 — Buscar y filtrar entradas

> **Como** usuario,
> **quiero** buscar entre mis credenciales y filtrarlas por tag,
> **para** encontrar una entrada rápidamente en vaults con muchas credenciales.

**Detalle:** `/` en `list` abre la búsqueda incremental sobre título, username, URL y
tags (case-insensitive, substring), con prefijos `t:`/`u:`. `f` filtra por tag desde la
barra de tags. Búsqueda y filtro de tag se combinan.

**Criterios de aceptación:**

- [ ] `/` abre el input y cada pulsación filtra la lista en vivo.
- [ ] La búsqueda cubre título, username, URL y tags, sin distinguir mayúsculas.
- [ ] `t:dev` filtra solo entries con el tag "dev"; `u:GitHub` filtra por username.
- [ ] `f` selecciona/des-selecciona un tag de la barra; combinable con la búsqueda.
- [ ] `Esc`/vaciar el input restaura la lista completa y la selección se conserva.
- [ ] Sin resultados se muestra un mensaje informativo de "sin resultados" (distinto
      del estado vacío global).

**Edge cases:**

- Búsqueda con 10.000 entries → filtrado < 50 ms (SPEC §9).
- Prefijo incompleto ("t:" sin valor) → se trata como búsqueda de texto literal o
  estado de filtro vacío, sin error.
- Tags con espacios/acentos → el filtro coincide por substring exacto case-insensitive.
- Filtrar y luego borrar la entry seleccionada (fase 2) → la selección se re-posiciona
  sin `panic`.
- Cambiar el filtro durante una copia en curso → no interfiere con el indicador.

### US-4 — Copiar credenciales al portapapeles con auto-clear

> **Como** usuario,
> **quiero** copiar la password o el username con una tecla y que el portapapeles se
> limpie solo,
> **para** pegar mis credenciales sin dejarlas expuestas en el sistema.

**Detalle:** `c` copia la password y `C` el username desde `list`/`detail`. Al copiar
se muestra el indicador "copied" con cuenta atrás; el portapapeles se limpia tras
`clipboard_seconds` (defecto 15 s) y **siempre** al salir o al lockear.

**Criterios de aceptación:**

- [ ] `c` en `list`/`detail` copia la password; `C` copia el username.
- [ ] Tras copiar aparece el indicador "copied" con la cuenta atrás visible.
- [ ] El portapapeles se vacía transcurrido `clipboard_seconds` (15 s por defecto).
- [ ] Al salir o lockear (incluido auto-lock), el portapapeles queda vacío.
- [ ] En entorno headless (sin portapapeles disponible), la acción se desactiva con
      aviso y la app sigue funcionando.

**Edge cases:**

- Copiar dos veces seguidas → el temporizador del auto-clear se reinicia con la última
  copia.
- Salir durante la cuenta atrás → se limpia igualmente en el teardown.
- Copiar desde una entry cuyo password se muestra oculta → se copia el valor real, no
  la máscara.
- `C` en `list` con la entry sin username → copia cadena vacía con aviso, sin error.

### US-5 — Auto-lock por inactividad

> **Como** usuario,
> **quiero** que el vault se bloquee automáticamente tras un periodo sin usarlo,
> **para** proteger mis credenciales si me alejo del terminal.

**Detalle:** cualquier tecla resetea el contador de inactividad. Al alcanzar
`auto_lock_minutes` (defecto 5) la app pasa a `unlock` sin cerrarse, descarta la clave
y la payload con `zeroize` y limpia el portapapeles.

**Criterios de aceptación:**

- [ ] Sin pulsar ninguna tecla durante `auto_lock_minutes`, la app pasa a `unlock`
      automáticamente sin cerrarse.
- [ ] Cualquier tecla (incluidos inputs de `form`/búsqueda) resetea el contador.
- [ ] Tras el lock, la clave derivada y la payload descifrada se descartan con
      `zeroize`.
- [ ] Tras el lock, el portapapeles queda vacío.
- [ ] Re-desbloquear tras el auto-lock funciona con la master password (fase 2) y el
      backoff de intentos se conserva.

**Edge cases:**

- Inactividad durante un input de texto en `form`/búsqueda → aplica igualmente (regla
  simple: cualquier inactividad real de teclas).
- Auto-lock con una copia en curso → el portapapeles se limpia igual.
- Auto-lock durante la confirmación de borrado → la confirmación se descarta y la app
  queda en `unlock`.
- Auto-lock con `auto_lock_minutes = 1` → el lock ocurre al minuto, sin desviación
  perceptible.
- Resize o eventos no de teclado → no resetean el contador.

### US-6 — Guardar una contraseña generada como nueva entrada

> **Como** usuario,
> **quiero** guardar una contraseña recién generada como nueva credencial,
> **para** no escribirla a mano ni perderla.

**Detalle:** `s` desde `generator` abre `form` con el campo `password` pre-cargado; el
usuario completa `title` y `username` y guarda con el ciclo de fase 2.

**Criterios de aceptación:**

- [ ] `s` desde `generator` abre `form` con la password generada ya rellenada.
- [ ] Al guardar, la entry aparece en `list` y persiste en el `.cofre` (reabrir la
      mantiene).
- [ ] `Esc` desde ese `form` descarta sin persistir.
- [ ] La validación de `title`/`username` no vacíos de fase 2 aplica también aquí.

**Edge cases:**

- Guardar con error de disco → la entry queda en memoria y se informa; el archivo
  previo no se corrompe (guardado atómico de fase 2).
- Volver al generador sin guardar → la contraseña anterior se descarta con `zeroize`.

## 4. Definition of Done (DoD)

La fase 3 se considera terminada cuando:

- [ ] El generador de contraseñas es funcional con todas las opciones del SPEC §4.3
      (largo 4–128, charsets, evitar ambiguos, al menos una de cada clase), con
      regeneración en vivo.
- [ ] El análisis de fortaleza del SPEC §7 se muestra junto al resultado con
      clasificación débil/media/fuerte y visual de barra.
- [ ] Búsqueda incremental con `/` (título, username, URL, tags, case-insensitive,
      substring), prefijos `t:`/`u:` y filtro por tag con `f` combinable.
- [ ] `c`/`C` copian password/username con indicador "copied"; el portapapeles se
      limpia tras `clipboard_seconds` y **siempre** al salir o lockear.
- [ ] Auto-lock tras `auto_lock_minutes` de inactividad con `zeroize` de clave y
      payload y limpieza del portapapeles.
- [ ] En entorno headless, el portapapeles se desactiva con aviso sin romper la app.
- [ ] `s` desde `generator` guarda la contraseña como nueva entry con el CRUD de fase 2.
- [ ] La máquina de pantallas de fases 1–2 se conserva; las transiciones existentes
      siguen siendo válidas.
- [ ] `cargo fmt --check` pasa sin diferencias.
- [ ] `cargo clippy -- -D warnings` no reporta `warnings`.
- [ ] `cargo test` pasa, incluyendo unit tests de: garantías del generador por
      charset/ambigüedad/"una de cada clase", estimador de fortaleza, lógica de
      búsqueda/filtros (incluidos prefijos y combinación con tags), auto-clear del
      portapapeles y auto-lock por inactividad.
- [ ] No se implementa funcionalidad fuera de alcance (§1.2): nada de `settings`
      funcional, cambio de master password ni `require_password_on_delete`.

## 5. Fase siguiente (preview, fuera de esta entrega)

Tras validar las funciones de UX, la siguiente fase corresponde a **M4 — Polish** del
PRD: pantalla `settings` funcional, cambio de master password (re-derivación con nuevo
salt y re-cifrado), `require_password_on_delete`, errores claros, tests y
documentación final, según `docs/SPEC.md` §4.7–4.8, §8, §10 y §11.

## 6. Nota sobre tareas

Este documento define **qué** se desarrolla (requerimientos, historias de usuario con
criterios de aceptación y edge cases) y **cuándo está terminado** (DoD). El desglose
en **tareas de implementación** (epics, subtareas, estimaciones y orden de trabajo) se
detallará en un documento de planificación posterior.