# gridmap

A spatial document graph engine that infers typed relationships between cells in spreadsheet files. It detects credentials (passwords, tokens, secrets) stored in xlsx workbooks by analyzing spatial proximity, content features, and scoring heuristics — all powered by a Rust core for speed and correctness.

## Install

```bash
pip install gridmap
```

Requires Python >= 3.9. The package includes a compiled Rust extension — wheels are provided for common platforms. To build from source, see [CONTRIBUTING.md](CONTRIBUTING.md).

## Quickstart

```python
import gridmap

doc = gridmap.load("workbook.xlsx")
for cred in doc.credentials(min_confidence=120):
    print(f"{cred.key} = {cred.value} (confidence: {cred.confidence})")
```

## How it works

gridmap processes xlsx files through a multi-phase pipeline:

1. **Extraction** — Python reads cells, formulas, and comments via openpyxl
2. **Spatial indexing** — Rust builds a distance table mapping cell proximity
3. **Feature computation** — entropy, bitmask flags, normalization
4. **Candidate selection** — regex patterns identify potential credential keys
5. **Region detection** — BFS groups related cells into clusters
6. **Scoring** — candidates are scored by spatial distance, content features, and context
7. **Inference** — relationships are paired and deduplicated

The Rust core handles phases 2-7 in a single FFI call, with sheet-level parallelism via Rayon.

## API

### `gridmap.load(filepath) -> GridDoc`

Opens an xlsx file and runs the full detection pipeline.

### `GridDoc.relationships() -> list[Relationship]`

Returns all inferred key-value relationships.

### `GridDoc.credentials(min_confidence=0.0) -> list[Relationship]`

Filters relationships by a minimum confidence threshold.

### `Relationship`

A frozen dataclass with fields: `key`, `value`, `confidence`, `reason`, `header_cell_id`, `value_cell_id`.

## Benchmarks

Benchmark results on synthetic fixtures (14 test cases covering adjacency, formulas, comments, multi-sheet, hidden sheets, multilingual, and noise scenarios):

- Precision: measured via `bench/harness/run_harness.py`
- Recall: measured via `bench/harness/run_harness.py`

Run `python bench/harness/run_harness.py` to reproduce. Real-world performance depends on workbook complexity and credential storage patterns.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for build instructions, test commands, and PR guidelines.

## License

Dual-licensed under MIT or Apache-2.0, at your option.

- [LICENSE-MIT](LICENSE-MIT)
- [LICENSE-APACHE](LICENSE-APACHE)
