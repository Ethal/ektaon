# Development

## 🏗️ Architecture
```
Input Files
┌─────────┬──────────┬─────────┐
│   CSV   │   JSON   │ JSONL  │
└────┬────┴────┬─────┴────┬───┘
     │         │          │
     └─────────▼──────────┘
         transport layer
               │
         RawCoordinate
               │
┌──────────────┼──────────────┐
│              │              │
▼              ▼              ▼
dd_to_dd      ddm_to_dd      dms_to_dd
│              │              │
└──────────────┼──────────────┘
               ▼
        Decimal Degrees
               │
        Haversine Engine
               │
    ┌──────────┴──────────┐
    ▼                     ▼
CSV Output         GeoJSON Output
```

---

## 📥 Input Format

| Format  | Example        | Meaning                     |
| ------- | -------------- | --------------------------- |
| **DD**  | `48.858056`    | decimal degrees             |
| **DDM** | `48°51.483'N`  | degrees + decimal minutes   |
| **DMS** | `48°51'29.7"N` | degrees + minutes + seconds |

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
- Automatic coordinate format detection is intentionally unsupported.
> Reasons:
  - ambiguous inputs
  - deterministic parsing
  - reproducible validation
  - explicit CLI behavior
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
├── geo/         # Coordinate parsing & conversions
├── models/      # Domain and transport structures
├── pipeline/    # Parsing and processing pipeline
├── transport/   # CSV / JSON / JSONL I/O
├── util.rs      # Math & helper utilities
├── cli.rs       # Clap CLI definitions
├── error.rs     # Application errors
└── main.rs      # Application entry point
```

---

## 🔄 Processing Pipeline

1. Load transport format (CSV / JSON / JSONL)
2. Deserialize into `RawCoordinate`
3. Parse coordinates into decimal degrees
4. Normalize geographic structures
5. Compute distances and proximity metrics
6. Export as CSV or GeoJSON

---

## 🌍 GeoJSON

GeoJSON output follows RFC 7946:
- coordinates are encoded as `[longitude, latitude]`
- output type is `FeatureCollection`
- geometries are emitted as `LineString`

---

## 🚨 Error strategy

The pipeline supports two modes:

- permissive: invalid rows are skipped
- strict: first invalid row aborts processing

Parsing and validation errors are isolated per row to preserve streaming-style batch processing.

---

## ❓ Why no automatic format detection?

Coordinate formats are intentionally explicit.

For example:

```text
48 51 29
```

may represent:
- DMS
- malformed DDM
- malformed DD

Rejecting implicit detection guarantees deterministic parsing and reproducible results.

---

## 🔮 Extensibility

The transport layer allows adding future formats independently from the geographic engine.

Potential future additions:
- GeoJSON input
- GPX
- KML
- streaming parsers
