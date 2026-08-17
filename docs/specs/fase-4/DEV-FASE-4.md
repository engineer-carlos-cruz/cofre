# DEV-FASE-4 — Definición: Polish (M4)

> Definición de la cuarta y última fase de desarrollo de **Cofre**.
> Derivada de `docs/PRD.md` (M4 Polish) y de `docs/SPEC.md`.
> Estado: **definido**. Las tareas de implementación se detallarán en una etapa posterior.

## 1. Objetivo y alcance

La cuarta fase de desarrollo de Cofre cierra el producto: convierte el TUI funcional
de las fases 1–3 en la versión final con **configuración real** (pantalla `settings`
funcional), el **cambio de master password**, el borrado con confirmación de master
password y la revisión final de **errores claros, pruebas de integración y
documentación**.

Esta fase **no añade funcionalidad nueva de negocio** más allá de la configuración:
crypto, storage, CRUD, generador, búsqueda, portapapeles y auto-lock ya están
implementados (fases 2–3). Aquí se hacen funcionales las capacidades que dependían de
los ajustes y se cierra la calidad del producto.

### 1.1 Incluido en esta fase

- **Pantalla `settings` funcional** (SPEC §4.8): acceso desde `list` (atajo `t`,
  sugerido en DEV-FASE-1 §2.5); edición y persistencia de `auto_lock_minutes`
  (1–120), `clipboard_seconds` (5–120) y `require_password_on_delete` (bool).
- **Cambio de master password** (SPEC §4.8): pedir la actual (verificar), nueva +
  confirmación (mínimo 8+); al guardar se re-deriva la clave con **nuevo salt**,
  se re-cifra y se reescribe todo el archivo de forma **atómica**.
- **`require_password_on_delete`** (SPEC §4.2): cuando está activo, el borrado de una
  entry pide y verifica la master password antes de eliminar; el flujo de confirmación
  `y`/`n` de fases 2–3 se conserva.
- **Revisión final de errores** (SPEC §8): mensajes claros y consistentes en todos los
  caminos (disco, headless, contraseña incorrecta), sin `panic`.
- **Pruebas finales** (SPEC §11): tests de integración de flujos completos con TUI
  simulado o headless, además de los unit tests por módulo.
- **Documentación final**: README y cierre de la documentación del producto
  (PRD M4), alineados con la implementación real.
- Módulo nuevo `config.rs` (defaults y validación de rangos, lógica pura);
  `models.rs` extiende `Settings` con `require_password_on_delete`.

### 1.2 Excluido de esta fase (extensiones futuras)

- Detección de contraseñas débiles/reutilizadas (basada en diccionarios) → extensiones
  futuras del PRD.
- Import/export (CSV/kdbx) y TOTP/2FA → extensiones futuras del PRD.
- Backup vía Git/archivos cifrados, FIDO2, recovery kit y múltiples vaults/perfiles →
  extensiones futuras del PRD.
- Migraciones de versión del formato (solo se soporta `version = 1`; la
  infraestructura de versión ya existe en el formato).
- Fuzzing del parseo binario (nice-to-have del SPEC §11, fuera de esta fase).

## 2. Requerimientos detallados

### 2.1 Dependencias

- **Ninguna dependencia nueva**: se reutilizan las de fases 1–3 (`crossterm`,
  `ratatui`, `argon2`, `chacha20poly1305`, `rand`, `zeroize`, `serde`/`serde_json`,
  `uuid`, `time`, `arboard`).
- Ninguna dependencia de red; todo es local.

### 2.2 Estructura de módulos

```text
cofre/
├── Cargo.toml
├── src/
│   ├── main.rs            # Entry point, orquestación init/teardown (fase 1)
│   ├── app.rs             # Estado global + máquina de pantallas (extendido: settings)
│   ├── terminal.rs        # Setup ratatui/crossterm (fase 1)
│   ├── ui/                # Render por pantalla (extendido: pantalla settings)
│   ├── crypto.rs          # Argon2id + XChaCha20-Poly1305 + formato (fase 2)
│   ├── storage.rs         # Lectura/escritura del archivo .cofre (fase 2)
│   ├── models.rs          # Entry, Vault, Settings (extendido: require_password_on_delete)
│   ├── password.rs        # Generador + análisis de fortaleza (fase 3)
│   ├── clipboard.rs       # Wrapper arboard + auto-clear (fase 3)
│   ├── config.rs          # Defaults + validación de rangos de Settings (nuevo)
│   └── errors.rs          # Tipos de error (fase 2)
└── tests/
    ├── integration.rs     # Flujos completos con TUI simulado / headless (nuevo)
    └── fixtures/          # Archivos .cofre de prueba (fase 2)
```

- `config.rs` es **lógica pura** (sin TTY): define los defaults y valida los rangos
  de los ajustes (SPEC §1). `models.rs` solo declara los tipos; `app.rs`/`ui/`
  orquestan la pantalla `settings` y los flujos de cambio de master password y de
  borrado protegido.
- La separación de responsabilidades de fases 1–3 se mantiene; no se acopla
  crypto/storage a UI.

### 2.3 Pantalla `settings` funcional

- Acceso desde `list` con el atajo `t` (confirmado durante la implementación).
- Campos editables (SPEC §4.8), cada uno con validación:
  - **auto_lock_minutes**: entero 1–120; fuera de rango → error de validación legible,
    no `panic`. Default 5.
  - **clipboard_seconds**: entero 5–120; fuera de rango → error de validación legible.
    Default 15.
  - **require_password_on_delete**: toggle bool; default `false`.
- Los cambios **solo persisten tras guardado explícito** (SPEC §4.8): se muta el
  estado en memoria, se actualiza `updated_at` y se ejecuta el ciclo de guardado de
  fase 2 (§2.8 de `DEV-FASE-2.md`).
- `Esc` descarta los cambios sin persistir; se conserva la sesión desbloqueada.
- Los nuevos valores aplican de inmediato a la sesión en curso: auto-lock (fase 3),
  auto-clear del portapapeles (fase 3) y borrado protegido (esta fase).

### 2.4 Cambio de master password

- Flujo (SPEC §4.8):
  1. Pedir la **master password actual** y verificarla (re-derivando con el salt del
     vault y comparando con la clave en memoria, o descifrando una muestra).
  2. Si la actual no coincide → error "Credenciales inválidas" y no se continúa
     (el backoff de 3 intentos de fase 2 aplica aquí).
  3. Pedir la **nueva** y su confirmación; mínimo 8+ caracteres (aviso, no bloquea);
     si no coinciden → error local.
  4. Al confirmar: generar **nuevo salt**, re-derivar la clave, **re-cifrar** toda la
     payload y reescribir el archivo de forma **atómica** (`.tmp` + `rename`).
- Después del cambio, la clave en memoria se sustituye por la nueva; la sesión sigue
  desbloqueada.
- La master password anterior deja de funcionar de inmediato (el archivo se reescribe
  con el nuevo salt).

### 2.5 Borrado con `require_password_on_delete`

- Cuando el ajuste está activo, el flujo de borrado de fase 2 se extiende:
  1. `d` en `detail` → confirmación inline `y`/`n` (como en fases 2–3).
  2. Con `y`, si `require_password_on_delete` está activo → se pide la master
     password (input oculto).
  3. Verificación: re-derivar con el salt del vault y **comparar con la clave en
     memoria** (sin I/O adicional). Si no coincide → error, no se borra; el backoff
     de intentos no aplica aquí (no es unlock, se conserva el comportamiento de fase 2).
  4. Con `y`/verificación correcta → se borra y persiste con el ciclo de guardado.
- Con el ajuste desactivado, el borrado se comporta igual que en fases 2–3.
- La master password introducida se limpia con `zeroize` al terminar el flujo.

### 2.6 Revisión final de errores

| Caso | Comportamiento |
|---|---|
| Vault no encontrado | Ofrecer crear nuevo vault |
| Contraseña incorrecta / archivo corrupto | "Credenciales inválidas o archivo dañado" (no revela cuál) |
| `magic`/`version` inválida | "No es un vault de Cofre" |
| Error de disco (permisos, full) | Mensaje claro + no sobreescribir el archivo anterior (guardado atómico) |
| API de portapapeles no disponible (headless) | Desactivar la función con aviso (fase 3) |
| Ajustes fuera de rango | Error de validación legible, sin `panic` |

- Repaso final: todos los mensajes son consistentes entre pantallas, no revelan
  secretos y nunca producen `panic` por condiciones controlables.

### 2.7 Pruebas finales (SPEC §11)

- **Unitarias** (por módulo): validación de rangos de `config.rs`, lógica de cambio de
  master password (re-salt + re-cifrado), verificación de master password en borrado.
- **Integración**: flujos completos con **TUI simulado** (inputs simulados) o
  **headless** sobre `settings`/cambio de master password/borrado protegido, usando
  fixtures `tests/fixtures/*.cofre`.
- Fuzz del parseo binario: **fuera de alcance** (nice-to-have, §1.2).

### 2.8 Requisitos no funcionales y documentación

- Sin red, sin telemetría, sin logs de contraseñas (SPEC §9).
- Apertura de vault < 500 ms y listado con 10.000 entries < 50 ms (SPEC §9) no se
  degradan con los cambios de esta fase.
- `zeroize` aplica a la master password nueva, la anterior y las introducidas en los
  flujos de esta fase.
- **Documentación final**: README del proyecto y cierre de la documentación alineado
  con la implementación (PRD M4).

## 3. Historias de usuario

### US-1 — Configurar los ajustes de la aplicación

> **Como** usuario,
> **quiero** ajustar el auto-lock, el tiempo del portapapeles y el borrado protegido,
> **para** adaptar el comportamiento del vault a mis preferencias.

**Detalle:** `t` desde `list` abre la pantalla `settings` funcional. Los cambios se
validan (rangos del SPEC §4.8) y solo persisten tras guardado explícito, aplicando a
la sesión en curso.

**Criterios de aceptación:**

- [ ] `t` desde `list` abre `settings` con los valores actuales (defaults 5 / 15 /
      false).
- [ ] Editar `auto_lock_minutes` y `clipboard_seconds` dentro de rango y guardar
      persiste en la payload y aplica a la sesión.
- [ ] Activar `require_password_on_delete` y guardar persiste el toggle.
- [ ] `Esc` descarta los cambios sin persistir.
- [ ] El guardado usa el ciclo de guardado de fase 2 y `updated_at` se actualiza.

**Edge cases:**

- Valores fuera de rango (0, 121, 4, 130) → error de validación legible, sin `panic`
  ni guardado.
- Guardado con error de disco → mensaje claro; el archivo previo no se corrompe
  (guardado atómico); los cambios quedan en memoria y se informa.
- Cambiar `auto_lock_minutes` a 1 → el auto-lock de fase 3 aplica al minuto.

### US-2 — Cambiar la master password

> **Como** usuario,
> **quiero** cambiar mi master password,
> **para** proteger el vault si la anterior se ha visto comprometida.

**Detalle:** se pide la actual (se verifica), la nueva y su confirmación. Al guardar,
se re-deriva la clave con un salt nuevo, se re-cifra toda la payload y se reescribe el
archivo atómicamente.

**Criterios de aceptación:**

- [ ] Con la actual incorrecta se muestra error y no se puede continuar.
- [ ] Con la actual correcta y nueva + confirmación válidas, el cambio se completa y
      el vault se reabre con la nueva password.
- [ ] Tras el cambio, la password anterior deja de funcionar y el archivo contiene el
      nuevo salt.
- [ ] La sesión queda desbloqueada tras el cambio.
- [ ] Un fallo de disco durante el re-cifrado no corrompe el archivo previo (guardado
      atómico).

**Edge cases:**

- Nueva password de menos de 8 caracteres → aviso, no bloquea.
- Nueva y confirmación no coinciden → error local, no se cambia nada.
- Cancelar con `Esc` en cualquier paso → no se modifica el vault.
- Master password introducida en el flujo → se limpia con `zeroize` al terminar.

### US-3 — Borrar una entrada con confirmación de master password

> **Como** usuario,
> **quiero** que el borrado de una entrada pida mi master password cuando el ajuste
> está activo,
> **para** evitar borrados accidentales incluso con la confirmación `y`/`n`.

**Detalle:** con `require_password_on_delete` activo, tras confirmar `y` se pide la
master password (input oculto). La verificación re-deriva la clave y la compara con la
de la sesión, sin I/O adicional.

**Criterios de aceptación:**

- [ ] Con el ajuste activo, `d` + `y` pide la master password antes de borrar.
- [ ] Con la master password correcta se borra la entry y persiste.
- [ ] Con la incorrecta se muestra error y la entry permanece.
- [ ] Con el ajuste desactivado, el borrado se comporta igual que en fases 2–3
      (solo `y`/`n`).
- [ ] La master password introducida se limpia con `zeroize` al terminar.

**Edge cases:**

- Cancelar con `Esc` durante la petición de master password → no se borra nada.
- Error de disco al persistir el borrado → se informa; el archivo no queda truncado.
- Borrado de la entry seleccionada → la selección se re-posiciona sin `panic` (fase 2).

### US-4 — Errores claros y consistentes

> **Como** usuario,
> **quiero** mensajes de error claros y coherentes en todos los flujos,
> **para** entender qué falló y qué puedo hacer sin perder la sesión.

**Detalle:** repaso final de los errores del SPEC §8 en todas las pantallas, sin
revelar secretos ni hacer `panic` por condiciones controlables.

**Criterios de aceptación:**

- [ ] Los mensajes de contraseña incorrecta / archivo corrupto no distinguen la causa.
- [ ] Los errores de disco muestran un mensaje claro y la app sigue con el estado en
      memoria.
- [ ] En entorno headless, el portapapeles se desactiva con aviso sin romper la app.
- [ ] Ningún error controlable produce `panic` ni deja la terminal en modo raw.

**Edge cases:**

- Disco lleno o sin permisos durante un guardado → mensaje claro, archivo previo
  intacto (guardado atómico).
- Mensajes largos → se muestran legibles sin romper el layout (resize de fase 1).

### US-5 — Confianza mediante pruebas de integración y documentación final

> **Como** usuaria o mantenedor,
> **quiero** una suite de integración que valide los flujos completos y una
> documentación alineada con la implementación,
> **para** entregar un producto fiable y mantenible.

**Detalle:** tests de integración de flujos completos (TUI simulado o headless) sobre
settings, cambio de master password y borrado protegido, más la documentación final
del README.

**Criterios de aceptación:**

- [ ] Existen tests de integración de cambio de master password (re-salt + re-cifrado)
      y de borrado protegido, que pasan en CI/headless.
- [ ] `cargo test` pasa, incluyendo los tests unitarios e de integración de esta fase.
- [ ] El README documenta el uso del producto y los atajos definitivos (SPEC §5).

**Edge cases:**

- Tests sobre fixtures `.cofre` corruptos → validan los mensajes de error sin `panic`.
- La suite completa no requiere TTY ni red.

## 4. Definition of Done (DoD)

La fase 4 se considera terminada cuando:

- [ ] La pantalla `settings` es funcional: `auto_lock_minutes` (1–120),
      `clipboard_seconds` (5–120) y `require_password_on_delete` se editan, validan y
      persisten con guardado explícito (SPEC §4.8).
- [ ] Cambio de master password: re-derivación con **nuevo salt**, re-cifrado completo
      y guardado atómico; la password anterior deja de funcionar.
- [ ] Con `require_password_on_delete` activo, el borrado pide y verifica la master
      password; desactivado, conserva el flujo de fases 2–3.
- [ ] Los errores del SPEC §8 muestran mensajes claros y consistentes sin `panic`.
- [ ] `cargo fmt --check` pasa sin diferencias.
- [ ] `cargo clippy -- -D warnings` no reporta `warnings`.
- [ ] `cargo test` pasa, incluyendo unit tests de: validación de rangos de `config.rs`,
      lógica de cambio de master password (re-salt + re-cifrado) y verificación de
      master password en borrado; e integración de flujos completos con TUI simulado
      o headless.
- [ ] No se implementa funcionalidad fuera de alcance (§1.2): nada de extensiones
      futuras del PRD ni fuzz.
- [ ] La máquina de pantallas de fases 1–3 se conserva; las transiciones existentes
      siguen siendo válidas.

## 5. Después de esta fase (extensiones futuras, fuera de esta entrega)

Con M4 completado el producto cubre todo el alcance del MVP. Las extensiones
futuras del PRD §11 (detección de contraseñas débiles/reutilizadas, import/export
CSV/kdbx, TOTP/2FA, backup vía Git, FIDO2, recovery kit, múltiples vaults/perfiles y
migraciones de versión del formato) quedan fuera de las cuatro fases.

## 6. Nota sobre tareas

Este documento define **qué** se desarrolla (requerimientos, historias de usuario con
criterios de aceptación y edge cases) y **cuándo está terminado** (DoD). El desglose
en **tareas de implementación** (epics, subtareas, estimaciones y orden de trabajo) se
detallará en un documento de planificación posterior.