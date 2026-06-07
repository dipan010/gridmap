# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-06-08

### Added

- Core detection engine in Rust with spatial indexing, feature computation, candidate selection, region detection, scoring, and inference phases
- Python API: `gridmap.load()`, `GridDoc`, `Relationship`
- Extraction layer via openpyxl supporting formulas, comments, merged cells, hidden sheets, and multi-sheet workbooks
- PyO3 bindings (`gridmap._core`) bridging Python and Rust
- Detection pathways: adjacent cells, inline same-cell, formula concatenation, comments on headers, inline comments, split passwords, multilingual keywords
- 14 synthetic test fixtures with ground truth
- Precision/recall benchmark harness
- Integration test suite
