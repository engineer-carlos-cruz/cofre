# SPEC — Cofre

> Especificación técnica del gestor de contraseñas de terminal (TUI) en Rust.
> Derivada de `docs/PRD.md`. Estado: **borrador v1**.

## 1. Alcance

Implementar **Cofre**: un gestor de contraseñas de escritorio, offline-first, de un solo binario, con interfaz TUI (`ratatui` + `crossterm`). Contiene cifrado en reposo, desbloqueo con contraseña maestra, CRUD de credenciales, generador de contraseñas, búsqueda, portapapeles con borrado automático y auto-lock por inactividad.

```text
cofre/
├── Cargo.toml
├── src/
│   ├── main.rs            # Entry point, init/teardown del terminal
│   ├── app.rs             # Estado global, máquina de pantallas
│   ├── terminal.rs        # Setup ratatui/crossterm, eventos, raw mode
│   ├── ui/                # Widgets por pantalla (mod.rs, list.rs, …)
│   ├── crypto.rs          # Argon2id + XChaCha20-Poly1305
│   ├── storage.rs         # Lectura/escritura del archivo .cofre
│   ├── models.rs          # Entry, Vault, Settings, dos de sero sero
│   ├── password.rs        # Generador + análisis de fortaleza
│   ├── clipboard.rs       # Arboard wrapper + auto-clear
│   ├── config.rs          # Configuración y valores por defecto
│   └── errors.rs          # Tipos de error y mensajes al usuario
├── tests/
│   ├── integration.rs
│   └── fixtures/          # Archivos .cofre de prueba
└── docs/
    ├── PRD.md
    └── SPEC.md
```

## 2. Terminología

| Término | Definición |
|---|---|
| **Vault** | Archivo `.cofre` cifrado con las credenciales y ajustes |
| **Master password** | Contraseña maestra; nunca se almacena, solo se deriva una clave |
| **Entry** | Credencial individual (título, usuario, contraseña, URL, notas, tags) |
| **Session** | Periodo de tiempo entre unlock y lock/exit |

## 3. Especificación de seguridad y cifrado

### 3.1 Derivación de clave (KDF)

- **Algoritmo**: Argon2id.
- **Parámetros por defecto** (configurables, almacenados en el header):
  - `m_cost = 19456 KiB` (19 MiB)
  - `t_cost = 2`
  - `p_cost = 1`
- **Salt**: 16 bytes aleatorios por vault, generados con `rand::rngs::OsRng`. Se almacenan en claro en el header (no es secreto).
- **Largo de clave derivada**: 32 bytes.
- La clave **nunca se escribe en disco**; vive solo en memoria durante la sesión.

### 3.2 Cifrado autenticado

- **Cifrado**: XChaCha20-Poly1305 (`chacha20poly1305` crate).
- **Nonce**: 24 bytes aleatorios por operación de guardado, generados con `OsRng`.
- **Tag**: 16 bytes Poly1305, anexado al ciphertext.
- El descifrado sirve **además como verificación de contraseña**: un error de autenticación implica "contraseña incorrecta **o** archivo corrupto".

### 3.3 Formato del archivo `.cofre` (v1)

Layout binario, todos los enteros big-endian:

```text
│ Offset  │ Tamaño │ Campo                                          │
│ 0       │ 7      │ Magic: "COFRE1"                                │
│ 7       │ 1      │ Format version (u8) = 1                        │
│ 8       │ 4      │ argon2 m_cost (u32, en KiB)                    │
│ 12      │ 4      │ argon2 t_cost (u32)                            │
│ 16      │ 4      │ argon2 p_cost (u32)                            │
│ 20      │ 16     │ Salt                                            │
│ 36      │ 24     │ XChaCha nonce                                   │
│ 60      │ var    │ Ciphertext (JSON cifrado de la payload)        │
│ +len    │ 16     │ Poly1305 tag                                    │
```

Reglas:
- `magic` o `version` inválidos → error "archivo no es un vault de Cofre".
- Cualquier campo con largo insuficiente → error de formato, no `panic`.
- La versión habilita migraciones futuras (registro `version_migrations`).

### 3.4 Payload descifrada (JSON)

```json
{
  "version": 1,
  "updated_at": "2026-08-11T17:18:00Z",
  "settings": {
    "auto_lock_minutes": 5,
    "clipboard_seconds": 15
  },
  "entries": [
    {
      "id": "uuid-v4",
      "title": "GitHub",
      "username": "carlos",
      "password": "s3cret!",
      "url": "https://github.com",
      "notes": "",
      "tags": ["dev", "trabajo"]
    }
  ]
}
```

- `updated_at` se actualiza en cada guardado exitoso.
- Campos opcionales (`url`, `notes`) se serializan como string vacío si no existen.
- `tags` es un array, puede ir vacío.

### 3.5 Flujo de guardado

1. Mutar estado en memoria.
2. `updated_at = now`.
3. Serializar payload → JSON.
4. Envolver en `seal()`: `ciphertext + tag` con nuevo nonce.
5. Escribir header + salt (fijo del vault) + nonce + ciphertext.

### 3.6 Flujo de apertura (unlock)

1. Leer archivo → validar magic/version → parámetros Argon2 y salt.
2. Derivar clave del master password.
3. `decrypt()`: si falla la autenticación → "Contraseña incorrecta".
4. Parsear JSON → cargar `Vault` en memoria.

## 4. Especificación funcional

### 4.1 Desbloqueo / creación de vault

- **Sin archivo**: pantalla `unlock`, sección "crear nuevo vault". Se pide:
  - Master password (input oculto, sin echo).
  - Confirmación. Si no coinciden → error local, no crea nada.
- **Con archivo**: solo pedir master password.
- Regla de mínimo: la master password debe tener **8+ caracteres** (configurable). Se avisa, no se bloquea.
- Tras 3 intentos fallidos consecutivos (configurable) → 5 s de backoff con cuenta atrás visible; evita fuerza bruta en el TUI.

### 4.2 CRUD de entradas

| Operación | Detalle |
|---|---|
| **Crear** | Pantalla `form`; campos title*, username*, password*, url, notes, tags. Valida que title y username no estén vacíos. |
| **Listar** | Pantalla `list`, una fila por entry (título + username). |
| **Ver** | Pantalla `detail`; password oculta por defecto, toggle con `Tab`/`p`. |
| **Editar** | Misma pantalla `form` pre-rellenada; `Esc` descarta, guardado confirmado en `detail`. |
| **Borrar** | Confirmación inline (`y`/`n`) en `detail`; irreversible. Pide confirmación de la master password si `require_password_on_delete` está activo. |

### 4.3 Generador de contraseñas

- Pantalla `generator`, opciones en vivo (se regenera al pulsar `r` o cambiar una opción):
  - **Largo**: 4–128, defecto **20**.
  - **Charsets** (toggle): minúsculas, mayúsculas, dígitos, símbolos `!@#$%^&*()-_=+[]{};:,.?`.
  - **Evitar ambiguos** `0O1lI` (bool).
  - **Al menos una de cada clase** (bool).
- Algoritmo: construir pool según charsets; si "una de cada clase", garantizar 1 símbolo de cada clase activa y rellenar el resto del pool; barajar con `OsRng` (Fisher-Yates).
- Acciones: `Enter` copiar al portapapeles (con auto-clear), `s` guardar como nueva entry (abre `form` pre-cargado con la contraseña).
- Acompaña el resultado con una nota de fortaleza (ver §7).

### 4.4 Búsqueda y filtrado

- Input incremental en `list` con `/`. Cada pulsación filtra en vivo.
- Campos buscados: **título, username, URL, tags** (case-insensitive, substring).
- `t:`, `u:` prefijos filtran por tag / username ("u:GitHub").
- Filtro por tag: barra de tags en `list`; selección con `f`. Combinable con la búsqueda.

### 4.5 Portapapeles

- `c` copia password, `C` (shift) copia username desde `list`/`detail`.
- Tras copiar: indicador visual "copied" + cuenta atrás.
- **Auto-clear** tras `clipboard_seconds` (defecto 15 s) y **siempre** al salir/lockear (se limpia el portapapeles de la sesión al terminar).
- En el lock, la copia en el portapapeles se borra igualmente.

### 4.6 Auto-lock

- Contador de inactividad: se resetea con **cualquier tecla**.
- Al llegar a `auto_lock_minutes` (defecto 5) → transición a pantalla `unlock` (sin cerrar app).
- La clave y la payload descifrada se descartan de memoria (zeroize).
- No aplica durante inputs de texto en `form`/búsqueda … Aplica también (regla simple, de implementación: se cuenta cualquier inactividad real de teclas).

### 4.7 Pantallas (máquina de estados)

```text
unlock -> list -> detail -> form
                -> generator
                -> settings
cualquiera -> unlock   (auto-lock / Ctrl+L)
cualquiera -> exit     (q)
```

| Pantalla | Persistencia | Contenido |
|---|---|---|
| `unlock` | — | pedir master password / crear vault / mensajes de error |
| `list` | — | búsqueda, filtros, tabla de entries |
| `detail` | — | campos, copiar, editar, borrar |
| `form` | — | edición/creación de una entry |
| `generator` | — | generador en vivo |
| `settings` | sí | cambiar master password, auto-lock, clipboard timeout |

### 4.8 Settings

- **Cambiar master password**: pedir actual (verificar), nueva + confirmar. Al guardar: re-deriva clave con **nuevo salt**, re-cifra y reescribe todo el archivo.
- **auto_lock_minutes** (1–120).
- **clipboard_seconds** (5–120).
- Los cambios persisten en la payload (`settings`) y requieren guardado explícito.

## 5. Atajos de teclado (definitivos)

| Tecla | Pantalla de lista | Pantalla de detalle |
|---|---|---|
| `↑/↓`, `j/k` | Navegar | — |
| `Enter` | Abrir detalle | — |
| `/` | Buscar | — |
| `f` | Filtrar por tag | — |
| `c` / `C` | Copiar password / username | igual |
| `n` | Nueva entry | — |
| `e` | — | Editar |
| `d` | — | Borrar |
| `g` | Abrir generador | — |
| `–` | al generador: `r` regenerar, `s` guardar, `Enter` copiar | — |
| `q`/`Esc` | Volver / salir | Volver a lista |
| `Ctrl+L` | Lock | Lock |
| `Tab` / `p` | — | Mostrar/ocultar password |

## 6. Diseño TUI (wireframes)

```text
┌ COFRE ──────────────────────────────────────────┐
│ [unlock]                                        │
│   Master password: ••••••••••    (Enter)        │
│   3 attempts remaining                          │
└─────────────────────────────────────────────────┘

┌ COFRE ──────────────────────────────────────────┐
│ [list]  search: /github   tag: [dev ✕]          │
│  • GitHub          carlos        🔒             │
│  » GitHub Actions  ci-bot        🔒             │
│  • Email           carlos@x.com  🔒             │
├─────────────────────────────────────────────────┤
│  n new · g generator · c copy · q quit          │
└─────────────────────────────────────────────────┘
```

- Layout mínimo de 80×24, con fallback para terminales menores (scroll).
- Colores: tema claro/oscuridad por defecto de ratatui; indicador de estado en la barra inferior.

## 7. Análisis de fortaleza

- Estimador basado en entropía estimada: `pool_size ^ length`.
- Clasificación: **débil** (< ~48 bits), **media** (48–80), **fuerte** (> 80).
- Visual: `▓▓▓▓░░░░` más barra de colores + texto. No depende de diccionarios.

## 8. Manejo de errores

| Caso | Comportamiento |
|---|---|
| Vault no encontrado | Ofrecer crear nuevo vault |
| Contraseña incorrecta / archivo corrupto | "Credenciales inválidas o archivo dañado" (no revela cuál) |
| Magic/version inválida | "No es un vault de Cofre" |
| Error de disco (permisos, full) | Mensaje claro + no sobreescribir el archivo anterior (guardado atómico) |
| Puerto de teclado no disponible | Mensaje y salida limpia |
| API de portapapeles no disponible (headless) | Desactivar la función con aviso |

**Guardado atómico**: se escribe a `<archivo>.tmp` y se renombra (`rename`) sobre el original, para evitar truncar el vault si falla el guardado.

## 9. Requisitos no funcionales

- **Un solo binario** estático, sin dependencias runtime.
- **Rendimiento**: apertura de vault < 500 ms (Argon2id incluido); listado con 10.000 entries < 50 ms.
- **Memoria**: payload descifrada solo en RAM; `zeroize` de claves y passwords en drop.
- **Portabilidad**: Linux/macOS/Windows (crossterm); `arboard` con fallback.
- **Privacidad**: sin telemetría, sin red, sin logs de contraseñas.

## 10. Criterios de aceptación (testeables)

- [ ] Crear vault, abrir con la contraseña correcta y rechazar la incorrecta.
- [ ] El archivo `.cofre` sin master password no puede leerse (cifrado autenticado).
- [ ] CRUD completo (crear, listar, buscar, editar, borrar).
- [ ] Generador con todas las opciones del §4.3 respetadas.
- [ ] Copia/borrado automático del portapapeles (15 s y al salir).
- [ ] Auto-lock tras N min de inactividad.
- [ ] Guardado atómico: fallo de disco no corrompe el vault.
- [ ] Cambio de master password: re-cifrado completo y re-salt.

## 11. Plan de pruebas

- **Unitarias**: crypto round-trip, discriminación de clave errónea, formato binario con fixtures `tests/fixtures/*.cofre`, generador (garantías por charset/ambigüedad), estimador de fortaleza.
- **Integración**: flujos completos con `faux` TUI (inputs simulados) o headless sobre `list`/CRUD.
- **Fuzz (nice-to-have)**: parseo del header binario `cargo-fuzz`.

## 12. Mapeo con hitos del PRD

| PRD | Especificación |
|---|---|
| M1 Skeleton | §1, §5, §6 TUI |
| M2 Crypto & storage | §3, §4.1–4.2 |
| M3 UX | §4.3–4.6 |
| M4 Polish | §4.7–4.8, §8, §10, §11 |