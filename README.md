# 📍 Geographic Distance Calculator (ektaon)

> 🌐 [ektaon.ethal.fr](https://ektaon.ethal.fr)

![Rust](https://img.shields.io/badge/Rust-1.95.0-c5a059?logo=rust&style=flat-square) ![Rust Edition](https://img.shields.io/badge/edition-2024-orange?style=flat-square) ![License](https://img.shields.io/badge/License-MIT-gray?style=flat-square) ![Platform](https://img.shields.io/badge/Platform-Linux%20%7C%20macOS%20%7C%20Windows-black?logo=windows&style=flat-square)

This CLI tool calculates the distance between two geographic points from a CSV file using the **Haversine** formula.

The coordinates are read in a **unique and explicit** format, defined by the command-line options.

Internally, all coordinates are normalized to decimal degrees before any computation or output generation.

---

## 🎯 Features

- Reads a CSV file containing coordinates A → B
- Supports multiple coordinate formats (via CLI options)
- Internal conversion to decimal degrees (DD)
- Distance calculation:
  - kilometers
  - miles
- Writes an enriched CSV, JSON or GeoJSON as per RFC 7946
- Strict or permissive mode
- Ignore or block on invalid lines
- Unicode support for DMS/DDM formats (`° ′ ″`)
- Average Earth radius: **6,371 km (spherical approximation)**

---

## 📦 Supported Formats

| Type | Supported |
|---|---|
| Coordinate formats | DD, DDM, DMS |
| Input formats | CSV, JSON, JSONL |
| Output formats | CSV, GeoJSON |

---

## 🧭 Supported Coordinate Formats

The **coordinate format is global** and must be specified via the CLI.

The Input files **cannot contain mixed coordinate formats**.

### 1️⃣ Decimal Degrees (DD)

Option:
```
--coord-format=dd
```

Valid examples:
```
48.858056
-2.294500
```

- latitude: `[-90 ; +90]`
- longitude: `[-180 ; +180]`

### 2️⃣ Degrees / Minutes / Seconds (DMS)

Option:
```
--coord-format=dms
```

Valid examples:
```
48°51'29.6"N
48° 51′ 29″ N
2°17'40"W
2°17'40"O
```

✔ Accepted directions:
- longitude: `E`, `W`, `O`
- latitude: `N`, `S`

✔ Accepted Unicode symbols:

- degree: `°`
- minutes: `'` or `′`
- seconds: `"` or `″`

### 3️⃣ Degrees / Decimal Minutes (DDM)

Option:
```
--coord-format=ddm
```

Valid examples:
```
48°51.4'N
48° 51.4′ N
2°17.3'W
2°17.3' O
```

✔ Accepted directions:
- longitude: `E`, `W`, `O`
- latitude: `N`, `S`

✔ Accepted Unicode symbols:

- degree: `°`
- minutes: `'` or `′`

---

## 📄 Input CSV/JSON/JSONL File Format

The file must contain **at least** the following data:

| Data | Description |
|------|-----------|
| `name_a` | Name of point A |
| `lat_a` | Latitude of point A |
| `lon_a` | Longitude of point A |
| `name_b` | Name of point B |
| `lat_b` | Latitude of point B |
| `lon_b` | Longitude of point B |

The CSV file **shall contain headers** matching the expected column names.

👉 The `lat_*` and `lon_*` fields must conform to the **coordinate format chosen via the CLI**.

---

## 📤 Output CSV file

The output file contains:

- all input datas
- a unique identifier (`id`)
- normalized coordinates
- calculated distances
- near-equality flags

Additional fields:

| Data | Description |
|------|-----------|
| `id` | Line ID |
| `distance_km` | Distance in kilometers |
| `distance_miles` | Distance in miles |
| `lat_a_dd` | Latitude A in decimal degrees |
| `lon_a_dd` | Longitude A in decimal degrees |
| `lat_b_dd` | Latitude B in decimal degrees |
| `lon_b_dd` | Longitude B in decimal degrees |
| `lat_a_dms` | Latitude A in degrees minutes seconds |
| `lon_a_dms` | Longitude A in degrees minutes seconds |
| `lat_b_dms` | Latitude B in degrees minutes seconds |
| `lon_b_dms` | Longitude B in degrees minutes seconds |
| `nearly_lat` | Latitude A and B are almost identical|
| `nearly_lon` | Longitude A and B are almost identical |
| `nearly_both` | Point A and B are almost identical |

Tolerance for nearly is **1e-6** (~11 cm at the equator) 

---

## 📤 Output GeoJSON file as per RFC 7946 

The `type` of the GeoJSON is `FeatureCollection`.

The `feature` contains: 
  - `id`, a unique identifier(number), 
  - `properties`, 
    - all input data
    - calculated distances
    - near-equality flags
  - `geometry`,
    - `type` is `LineString` 
    - `coordinates` as `[[lon_a_dd, lat_a_dd],[lon_b_dd,lat_b_dd]]`

Propertiess added:

| Properties | Description |
|------|-----------|
| `distance_km` | Distance in kilometers |
| `distance_miles` | Distance in miles |
| `nearly_lat` | Latitude A and B are almost identical|
| `nearly_lon` | Longitude A and B are almost identical |
| `nearly_both` | Point A and B are almost identical |

Tolerance for nearly is **1e-6** (~11 cm at the equator) 

---

## 📐 Distance calculation

The calculation uses the **Haversine formula**:

- Earth modeled as a sphere
- Average radius: 6,371 km
- Results rounded to **2 decimal places**

---

## 🚦 Validation Modes

### Permissive Mode (default)

- Invalid lines are ignored
- Processing continues
- A final summary indicates the number of lines ignored

---

### Strict Mode

Option:
```
--strict
```

- The program stops at the **first invalid line**
- A detailed error message is displayed:
  - line number
  - exact cause (format, minutes, seconds, direction…)

- Strict mode is recommended for data validation pipelines.
- Permissive mode is useful for large heterogeneous datasets.

---

## ❌ Policy on mixed formats

⚠️ Mixed formats in the same file **are not supported**.

Invalid example:
```
--input-format=dms
48.858056
48°51'29"N
```

➡️ Error in strict mode
➡️ Line ignored in permissive mode

---

## 🏁 Usage

```bash
Usage: ektaon [OPTIONS] --input <INPUT> --output <OUTPUT> --coord-format <COORD_FORMAT> --input-format <INPUT_FORMAT>

Options:
  -i, --input <INPUT>                   Input file path
  -o, --output <OUTPUT>                 Output file path
  -c, --coord-format <COORD_FORMAT>     Coordinate format [possible values: dd, dms, ddm]
      --input-format <INPUT_FORMAT>     Input format [possible values: csv, json, jsonl]
      --output-format <OUTPUT_FORMAT>   Output format [possible values: csv, geo(GeoJSON compliant with RFC 7946.)]
  -s  --strict                          Strict mode: stop on first error
  -h, --help                            Print help
  -V, --version                         Print version
```

- Example of use

```bash

cargo run -- \
  --input examples/dms.json \
  --output output.geojson \
  --coord-format=dms \
  --input-format=json \
  --output-format=geo
```

- Example of an output csv file (from the input file: examples/dd_edge_case.csv)

```csv
id,name_a,lat_a_in,lon_a_in,lat_a_dd,lon_a_dd,lat_a_dms,lon_a_dms,name_b,lat_b_in,lon_b_in,lat_b_dd,lon_b_dd,lat_b_dms,lon_b_dms,distance_km,distance_miles,nearly_lat,nearly_lon,nearly_both
1,Paris,48.858056,2.2945,48.858056,2.2945,"48°51'29.00""N","2°17'40.20""E",Lyon,45.764043,4.835659,45.764043,4.835659,"45°45'50.55""N","4°50'8.37""E",393.73,244.65,false,false,false
2,Eiffel Tower,48.858056,2.2945,48.858056,2.2945,"48°51'29.00""N","2°17'40.20""E",Copy of Eiffel,48.858056,2.2945,48.858056,2.2945,"48°51'29.00""N","2°17'40.20""E",0.0,0.0,true,true,true
3,Point A,48.858056,2.2945,48.858056,2.2945,"48°51'29.00""N","2°17'40.20""E",Point A Micro,48.8580560001,2.2945000001,48.858056,2.2945,"48°51'29.00""N","2°17'40.20""E",0.0,0.0,true,true,true
4,Equator Start,0,0,0.0,0.0,"0°0'0.00""N","0°0'0.00""E",Equator End,0,1,0.0,1.0,"0°0'0.00""N","1°0'0.00""E",111.19,69.09,true,false,false
5,North Pole,90,0,90.0,0.0,"90°0'0.00""N","0°0'0.00""E",South Pole,-90,0,-90.0,0.0,"90°0'0.00""S","0°0'0.00""E",20015.09,12436.8,false,true,false
6,Greenwich,0,0,0.0,0.0,"0°0'0.00""N","0°0'0.00""E",Pacific Antipode,0,180,0.0,180.0,"0°0'0.00""N","180°0'0.00""E",20015.09,12436.8,true,false,false
7,East Side,10,179,10.0,179.0,"10°0'0.00""N","179°0'0.00""E",West Side,10,-179,10.0,-179.0,"10°0'0.00""N","179°0'0.00""W",219.01,136.09,true,false,false
```

- Example of an output GeoJSON file (from the input file: examples/dms.csv)

```json
{
  "type": "FeatureCollection",
  "features": [
    {
      "id": "1",
      "properties": {
        "distance_km": 392.93,
        "distance_miles": 244.16,
        "lat_a_in": "48°51'24.12\"N",
        "lat_b_in": "45°45'0.00\"N",
        "lon_a_in": "2°20'54.96\"E",
        "lon_b_in": "4°50'0.00\"E",
        "name_a": "Paris",
        "name_b": "Lyon",
        "nearly_both": false,
        "nearly_lat": false,
        "nearly_lon": false
      },
      "geometry": {
        "type": "LineString",
        "coordinates": [
          [
            2.3486,
            48.8567
          ],
          [
            4.833333,
            45.75
          ]
        ]
      }
    },
    {...}
  ]
}
```

---

## 📁 Example files

The repository provides example CSV files in:

```text
examples/
```

| File | Purpose |
|---|---|
| `dd.csv` | Standard decimal degree examples |
| `ddm.csv` | Standard DDM examples |
| `dms.csv` | Standard DMS examples |
| `dms.json` | Standard DMS examples |
| `dms.jsonl` | Standard DMS examples |
| `dd_edge_case.csv` | Geographic boundary edge cases |
| `dms_mixed.csv` | Mixed valid/invalid rows for strict/permissive mode testing |
| `empty.csv` | For header requirement testing |

---
