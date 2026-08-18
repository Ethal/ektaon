# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

### Changed
- Made `Position` private in `models/geo.rs`.`
- Use of `std::sync::Lazylock` instead of the crate `once_cell` in `geo/dms.rs` and `geo/dds.rs`

---
## [1.2.0] — 2026-05-16

### Added
- Support for **JSON and JSONL input formats** in addition to CSV.
- Support for **GeoJSON (RFC 7946) output format** using `FeatureCollection`.
- New `transport` module for handling I/O (CSV / JSON / JSONL)
- Support for **Decimal Degrees (DD) parsing**.
- Unified input model with `InputRow` and `RawCoordinate`
- GeoJSON structures (`Feature`, `Geometry`, etc.)
- Example files for JSON and JSONL inputs

### Changed
- Major refactor of the processing pipeline:
  - Separation of I/O (`transport`) and processing logic (`pipeline`)
  - `process_geo` now returns an `OutputRecord` instead of writing directly to CSV
- CLI improvements:
  - `--coord-format` introduced to explicitly define coordinate format
  - `--input-format` and `--output-format` added for full format control
- Improved and simplified coordinate handling (`Coordinate.dir` is now optional)
- Refactored DD/DM/DMS parsing into clearer, separated modules
- Unified input handling across CSV, JSON, and JSONL

### Fixed
- Improved validation of coordinate direction values (N/S/E/W/O)
- Fixed inconsistent parsing behavior across DMS and DDM inputs
- Better error handling for malformed CSV/JSON rows with line tracking
- Fixed JSONL line-by-line parsing robustness
- Improved strict mode behavior for early failure on invalid input

### Removed
- direct CSV writing from process_geo
- single-format input assumption in pipeline
- legacy input structs tied only to CSV/decimal degrees workflow

---
## [1.1.0] — 2026-05-11

### Added
- Support for DDM coordinate parsing
- Strict/permissive processing modes
- Distance calculations in kilometers and miles
- Nearly-equal coordinate detection

### Changed
- Improved coordinate normalization pipeline
- Enhanced Unicode support for geographic formats

### Fixed
- Various parsing and validation edge cases

---
## [1.0.0] — 2026-01

### Added
- Initial release
- CSV input support
- DMS coordinate parsing
- Haversine distance calculation
- Normalized geographic output generation
