# CHECKLIST-TASK — Checklist global de tareas de Cofre

> Checklist de seguimiento de todas las tareas de desarrollo de **Cofre**, derivada de
> los `docs/specs/<fase-N>/task.md` de las fases 1–4. Marca cada tarea con `[x]` cuando
> la consideres terminada (DoD: `cargo fmt --check`, `cargo clippy -- -D warnings`,
> `cargo test` en verde).
> Estado: **borrador v1**.

## Fase 1 — Fundación del proyecto (Skeleton)

- [ ] T-SK-01 — Crear el crate y la estructura de módulos
- [ ] T-SK-02 — Esqueleto de `main.rs` (orquestación)
- [ ] T-ERR-01 — Definir `CofreError` con mensajes legibles
- [ ] T-ERR-02 — Mapeo de error → exit code
- [ ] T-TERM-01 — Init: raw mode + alternate screen + cursor oculto
- [ ] T-TERM-02 — Teardown idempotente
- [ ] T-TERM-03 — Hook de panic con teardown
- [ ] T-EVT-01 — Event loop con poll, KeyEvent y Resize
- [ ] T-EVT-02 — `Ctrl+C` como salida normal
- [ ] T-APP-01 — Enum `Screen` y estado global
- [ ] T-APP-02 — Transiciones válidas e inválidas (función pura)
- [ ] T-APP-03 — `Enter` en `list`: detalle o estado vacío
- [ ] T-APP-04 — Atajos `n`, `g`, `e`, `t` (placeholders)
- [ ] T-APP-05 — `Esc`/`q` en hijas conservando selección
- [ ] T-UI-01 — Dispatch de render por pantalla
- [ ] T-UI-02 — Esqueletos identificables por pantalla (sin simular)
- [ ] T-UI-03 — Estado vacío en `list`
- [ ] T-UI-04 — Entries ficticias de demo (opcional y desactivable)
- [ ] T-EXIT-01 — Confirmación de salida desde `list`
- [ ] T-EXIT-02 — `Ctrl+C` directo desde cualquier pantalla
- [ ] T-EXIT-03 — Todos los caminos de salida pasan por teardown
- [ ] T-RES-01 — Cálculo de layout mínimo y scroll (función pura)
- [ ] T-RES-02 — Estado conservado al redimensionar
- [ ] T-ERR-ST-01 — stdout no-TTY
- [ ] T-ERR-ST-02 — `$TERM` ausente o inadecuado
- [ ] T-ERR-ST-03 — Fallo de raw mode / alternate screen y teardown

## Fase 2 — Cifrado y almacenamiento (Crypto & storage)

- [ ] T-00-01 — Añadir dependencias de crypto, modelos y storage
- [ ] T-00-02 — Estructura de módulos nuevos y fixtures
- [ ] T-MOD-01 — Tipos `Entry`, `Settings` y `Vault`
- [ ] T-MOD-02 — Serialización JSON de la payload (SPEC §3.4)
- [ ] T-ERR-01 — Variantes de error de crypto/storage
- [ ] T-ERR-02 — Manejo sin `panic` y mensajes de usuario
- [ ] T-CRY-01 — Derivación de clave Argon2id
- [ ] T-CRY-02 — Cifrado autenticado `seal`/`open`
- [ ] T-CRY-03 — Discriminación de clave errónea
- [ ] T-STO-01 — Escritura del archivo `.cofre` v1
- [ ] T-STO-02 — Lectura y validación del archivo
- [ ] T-STO-03 — Guardado atómico (`.tmp` + `rename`)
- [ ] T-STO-04 — Fixtures de prueba (`tests/fixtures/`)
- [ ] T-UNL-01 — Crear un vault nuevo
- [ ] T-UNL-02 — Apertura con master password (unlock)
- [ ] T-UNL-03 — Backoff de intentos fallidos
- [ ] T-CRUD-01 — Crear una entrada (`form`)
- [ ] T-CRUD-02 — Listar y ver entradas
- [ ] T-CRUD-03 — Editar una entrada
- [ ] T-CRUD-04 — Borrar una entrada
- [ ] T-CRUD-05 — Ciclo de guardado y `updated_at`
- [ ] T-SEC-01 — `zeroize` de master password y clave derivada
- [ ] T-SEC-02 — `zeroize` de passwords de entries

## Fase 3 — Experiencia de usuario (UX)

- [ ] T-00-01 — Añadir `arboard` y crear los módulos `password.rs` y `clipboard.rs`
- [ ] T-00-02 — Estado de UX en `AppState`
- [ ] T-GEN-01 — `PasswordOptions` y validación
- [ ] T-GEN-02 — Generación con Fisher-Yates, ambiguos y "una de cada clase"
- [ ] T-STR-01 — `estimate_strength` (entropía y clasificación)
- [ ] T-STR-02 — Visual de fortaleza en el TUI
- [ ] T-UI-GEN-01 — Pantalla `generator` funcional con opciones en vivo
- [ ] T-UI-GEN-02 — Acciones `Enter` (copiar) y `s` (guardar como nueva entry)
- [ ] T-SRC-01 — Búsqueda incremental con `/`
- [ ] T-SRC-02 — Prefijos `t:` y `u:`
- [ ] T-SRC-03 — Filtro por tag con `f`, combinable
- [ ] T-CLP-01 — Wrapper de `arboard` con fallback headless
- [ ] T-CLP-02 — Acciones `c`/`C` con indicador "copied"
- [ ] T-CLP-03 — Auto-clear del portapapeles
- [ ] T-LCK-01 — Contador de inactividad y transición a `unlock`
- [ ] T-LCK-02 — `zeroize` y limpieza del portapapeles en el lock
- [ ] T-SEC-01 — `zeroize` de datos sensibles de UX

## Fase 4 — Polish (M4)

- [ ] T-00-01 — Extender `Settings` y crear `config.rs` (sin dependencias nuevas)
- [ ] T-00-02 — Estado de settings en `AppState` y transición a `settings`
- [ ] T-CFG-01 — Defaults y validación de rangos
- [ ] T-CFG-02 — Aplicar los ajustes a la sesión en curso
- [ ] T-SET-01 — Pantalla `settings` funcional con guardado explícito
- [ ] T-PWD-01 — Verificación de la password actual y validación de la nueva
- [ ] T-PWD-02 — Re-salt, re-cifrado y escritura atómica
- [ ] T-DEL-01 — Borrado protegido con verificación de master password
- [ ] T-ERR-01 — Mensajes de error claros y consistentes (SPEC §8)
- [ ] T-INT-01 — Suite de integración de flujos completos
- [ ] T-DOC-01 — Documentación final

---

## Progreso

| Fase | Tareas | Completadas |
|---|---|---|
| Fase 1 | 26 | 0/26 |
| Fase 2 | 23 | 0/23 |
| Fase 3 | 17 | 0/17 |
| Fase 4 | 11 | 0/11 |
| **Total** | **77** | **0/77** |