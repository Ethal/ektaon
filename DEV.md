# Development

## 🏗️ Architecture
```
           ┌────────────┐
           │   DMS str  │
           └─────┬──────┘
                 │
             dms_to_dd
                 │
┌────────────┐   ▼    ┌────────────┐
│  DDM str   │ ────▶  │  f64 (DD)  │ ◀── decimal input
└─────┬──────┘        └─────┬──────┘
      │                     │
  ddm_to_dd             dd_to_dms
                            │
                       DMS output
```

---

## 📥 Input Format

| Format  | Example        | Meaning                     |
| ------- | -------------- | --------------------------- |
| **DD**  | `48.858056`    | decimal degrees             |
| **DDM** | `48°51.483'N`  | degrees + minutes decimal   |
| **DMS** | `48°51'29.7"N` | degrees + minutes + seconds  |

---

## 🧪 Robustness & validation

- Strict validation of geographic boundaries
- Validation of minutes and seconds (`[0; 60[`)
- Validation of directions (`N/S/E/W/O`)
- Unit tests covering:
  - valid formats
  - format errors
  - Unicode
  - boundaries
  - calculations
- Automatic coordinate format detection is intentionally not supported.
  > Reason:
    - ambiguous inputs
    - deterministic parsing
    - predictable validation
    - simpler error handling

---

## 🧠 Design philosophy

- **Single internal format:** Decimal degrees (DD)
- Formats are **input/output**
- No ambiguous auto-detection
- The CLI decides, the engine calculates

---

## 📁 Source tree

```text
src/
├── geo/       # Coordinate parsing & conversion
├── models/    # Input/output normalized structures
├── pipeline/  # Parsing and processing pipeline
├── util.rs    # Math & helper utilities
├── cli.rs     # Clap CLI definitions
└── error.rs   # Application errors
```

## 🚨 Error strategy

The pipeline supports two modes:

- permissive: invalid rows are skipped
- strict: first invalid row aborts processing

Errors are handled at row level to preserve batch processing behavior.
