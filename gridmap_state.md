# GRIDMAP — State Tracker

> **Update this file after every prompt completion.**
> Mark status, record test results, note any deviations from the plan.

---

## Overall progress

```
Started:       ____-__-__
Current phase: NOT STARTED
Last prompt:   —
Next prompt:   00
Blockers:      none
```

---

## Prompt completion log

| # | Name | Status | Date | Tests | Notes |
|---|------|--------|------|-------|-------|
| 00 | Project scaffold | ⬜ NOT STARTED | — | — | — |
| 01 | Types + constants | ⬜ NOT STARTED | — | — | — |
| 02 | CellStore SoA | ⬜ NOT STARTED | — | — | — |
| 03 | Spatial primitives | ⬜ NOT STARTED | — | — | — |
| 04 | Feature computation | ⬜ NOT STARTED | — | — | — |
| 05 | Candidates + classify | ⬜ NOT STARTED | — | — | — |
| 06 | Region detection | ⬜ NOT STARTED | — | — | — |
| 07 | Detection (inline/formula/comment) | ⬜ NOT STARTED | — | — | — |
| 08 | Scoring engine | ⬜ NOT STARTED | — | — | — |
| 09 | Spatial inference | ⬜ NOT STARTED | — | — | — |
| 10 | Pipeline + Rayon | ⬜ NOT STARTED | — | — | — |
| 11 | PyO3 FFI boundary | ⬜ NOT STARTED | — | — | — |
| 12 | Python extraction | ⬜ NOT STARTED | — | — | — |
| 13 | Python API | ⬜ NOT STARTED | — | — | — |
| 14 | Test fixtures | ⬜ NOT STARTED | — | — | — |
| 15 | Integration + harness | ⬜ NOT STARTED | — | — | — |
| 16 | Docs + metadata | ⬜ NOT STARTED | — | — | — |

```
Status legend:
  ⬜ NOT STARTED
  🔨 IN PROGRESS
  ✅ COMPLETE
  🔴 BLOCKED
  🟡 NEEDS REVISION
```

---

## Module implementation status

### Rust crate: gridmap-core

| Module | File | Structs | Functions | Tests | Clippy | Status |
|--------|------|---------|-----------|-------|--------|--------|
| types | types.rs | CellType, Relationship, Region | — | ⬜ | ⬜ | ⬜ |
| store | store.rs | RawCell, CellStore | from_raw, len, is_empty | ⬜ | ⬜ | ⬜ |
| spatial | spatial.rs | DistanceTable | new, score, query_radius | ⬜ | ⬜ | ⬜ |
| features | features.rs | — | normalize, compute_entropy, precompute_features | ⬜ | ⬜ | ⬜ |
| candidates | candidates.rs | — | reduce_candidate_space, classify_cells, is_section_title | ⬜ | ⬜ | ⬜ |
| regions | regions.rs | — | detect_regions | ⬜ | ⬜ | ⬜ |
| detection | detection.rs | — | detect_inline_credentials, analyze_formulas, analyze_comments | ⬜ | ⬜ | ⬜ |
| scoring | scoring.rs | — | score_candidate | ⬜ | ⬜ | ⬜ |
| inference | inference.rs | — | infer_relationships, detect_split_password | ⬜ | ⬜ | ⬜ |
| pipeline | pipeline.rs | — | deduplicate, process_sheet, process_workbook | ⬜ | ⬜ | ⬜ |

### Rust crate: gridmap-py

| Module | File | Functions | Tests | Status |
|--------|------|-----------|-------|--------|
| bindings | lib.rs | process_sheet, process_workbook, version | ⬜ | ⬜ |

### Python package: gridmap

| Module | File | Functions / Classes | Tests | Status |
|--------|------|---------------------|-------|--------|
| extract | extract.py | extract_workbook, extract_single_sheet, collect_merge_origins, clean_comment | ⬜ | ⬜ |
| api | api.py | load, GridDoc, Relationship | ⬜ | ⬜ |
| init | __init__.py | public exports | ⬜ | ⬜ |

### Bench + tests

| Component | File | Status |
|-----------|------|--------|
| Fixture generator | bench/fixtures/generate_fixtures.py | ⬜ |
| Ground truth | bench/fixtures/ground_truth.json | ⬜ |
| Integration tests | python/tests/test_integration.py | ⬜ |
| P/R harness | bench/harness/run_harness.py | ⬜ |

### Docs

| File | Status |
|------|--------|
| README.md | ⬜ |
| CHANGELOG.md | ⬜ |
| CONTRIBUTING.md | ⬜ |
| LICENSE-MIT | ⬜ |
| LICENSE-APACHE | ⬜ |
| pyproject.toml metadata | ⬜ |

---

## Spec fix verification checklist

Mark each fix as verified when the relevant test passes.

| Fix | Description | Implemented in | Test exists | Verified |
|-----|-------------|----------------|-------------|----------|
| FIX 1 | VecDeque not list.pop(0) | regions.rs | ⬜ | ⬜ |
| FIX 2 | Spatial index over ALL cells | store.rs | ⬜ | ⬜ |
| FIX 3 | Comment merge by coordinate | store.rs | ⬜ | ⬜ |
| FIX 4 | Single workbook open | extract.py | ⬜ | ⬜ |
| WIN 1 | deque.popleft → VecDeque | regions.rs | ⬜ | ⬜ |
| WIN 2 | Single normalization pass | features.rs | ⬜ | ⬜ |
| WIN 3 | Compiled regex (lazy_static) | candidates.rs | ⬜ | ⬜ |
| GAP 1 | No shared file writes | pipeline.rs | ⬜ | ⬜ |
| GAP 2 | Distance table offset fix | spatial.rs | ⬜ | ⬜ |
| GAP 3 | Entropy pre-filter bitmask | features.rs | ⬜ | ⬜ |
| GAP 4 | Single-pass extraction | store.rs + extract.py | ⬜ | ⬜ |

---

## Test fixture status

| # | Fixture | Tests pathway | Expected findings | Status |
|---|---------|--------------|-------------------|--------|
| 01 | adjacent_right.xlsx | Spatial: right neighbor | 1 credential | ⬜ |
| 02 | adjacent_below.xlsx | Spatial: below neighbor | 1 credential | ⬜ |
| 03 | inline_same_cell.xlsx | Phase 7: inline detection | 1 credential | ⬜ |
| 04 | formula_concat.xlsx | Phase 8: formula analysis | 1 credential | ⬜ |
| 05 | comment_on_header.xlsx | Phase 9: comment mode 1 | 1 credential | ⬜ |
| 06 | comment_inline.xlsx | Phase 9: comment mode 2 | 1 credential | ⬜ |
| 07 | split_password.xlsx | Phase 10: split detection | 1 credential | ⬜ |
| 08 | multi_sheet.xlsx | Rayon parallel | 2 credentials | ⬜ |
| 09 | hidden_sheet.xlsx | Hidden sheet extraction | 1 credential | ⬜ |
| 10 | with_context.xlsx | Scoring: username/URL context | 1 credential (high conf) | ⬜ |
| 11 | noise.xlsx | Candidate reduction | 1 credential | ⬜ |
| 12 | multilingual.xlsx | Multilingual headers | 3 credentials | ⬜ |
| 13 | no_credentials.xlsx | False positive check | 0 credentials | ⬜ |
| 14 | merged_cells.xlsx | Merged cell handling | 1 credential | ⬜ |

---

## Benchmark results

```
Last run: not yet run
Corpus size: 0 fixtures

Aggregate:
  Precision: —
  Recall:    —
  F1:        —

Per-fixture breakdown:
  (populated after Prompt 15)
```

---

## Deviations from plan

> Record any deliberate departures from the prompt plan or implementation
> reference here. Each entry should explain WHAT changed, WHY, and which
> files were affected.

```
(none yet)
```

---

## Blockers and open questions

> Track anything that's blocking progress or needs a decision.

```
(none yet)
```

---

## Environment

```
Rust version:    (fill after install)
Python version:  (fill after install)
maturin version: (fill after install)
OS:              (fill)
```
