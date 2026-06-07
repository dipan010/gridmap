# GRIDMAP — Implementation Reference

## How to use this file

This is the source of truth for what each module contains, how data flows
between them, and which spec rules govern each piece. When implementing a
prompt, open this file alongside the prompt plan and the original spec.

If a function signature here conflicts with the prompt plan, this file wins.
If either conflicts with the original spec, the spec wins.

---

## Repository layout (target state after all prompts)

```
gridmap/
├── Cargo.toml                          workspace root
├── pyproject.toml                      maturin + package metadata
├── README.md
├── CHANGELOG.md
├── CONTRIBUTING.md
├── LICENSE-MIT
├── LICENSE-APACHE
├── .gitignore
│
├── crates/
│   ├── gridmap-core/                   pure Rust, zero FFI
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs                  module declarations + version()
│   │       ├── types.rs                enums, structs, constants
│   │       ├── store.rs                CellStore SoA
│   │       ├── spatial.rs              DistanceTable, query_radius
│   │       ├── features.rs             normalize, bitmask, entropy, header sets
│   │       ├── candidates.rs           regex patterns, reduce, classify
│   │       ├── regions.rs              BFS region detection
│   │       ├── detection.rs            inline, formula, comment detection
│   │       ├── scoring.rs              score_candidate
│   │       ├── inference.rs            infer_relationships, split password
│   │       └── pipeline.rs             deduplicate, process_sheet, process_workbook
│   │
│   └── gridmap-py/                     PyO3 bindings only
│       ├── Cargo.toml
│       └── src/
│           └── lib.rs                  #[pymodule] + type conversions
│
├── python/
│   ├── gridmap/
│   │   ├── __init__.py                 public exports
│   │   ├── api.py                      GridDoc, Relationship, load()
│   │   └── extract.py                  openpyxl extraction
│   └── tests/
│       ├── test_smoke.py               toolchain smoke test
│       ├── test_extract.py             extraction unit tests
│       ├── test_api.py                 API unit tests
│       └── test_integration.py         fixture-based integration tests
│
└── bench/
    ├── fixtures/
    │   ├── generate_fixtures.py        creates all test xlsx files
    │   ├── ground_truth.json           expected findings per fixture
    │   └── *.xlsx                      generated fixture files
    └── harness/
        └── run_harness.py              precision/recall reporter
```

---

## Dependency graph (crate level)

```
gridmap-core          (ahash, regex, rayon, lazy_static)
     ↑
gridmap-py            (pyo3, gridmap-core)
     ↑
gridmap (Python)      (openpyxl, gridmap._core via FFI)
```

Rule: gridmap-core never depends on pyo3 or any FFI crate.
Rule: gridmap-py never contains logic — only type conversion.

---

## Module: types.rs

```
SPEC PHASE:  Constants and Data Structures
PROMPT:      01
DEPENDS ON:  nothing
DEPENDED BY: every other module in gridmap-core
```

### Constants

```rust
pub const NEIGHBOR_RADIUS: i32 = 3;
pub const RELATIONSHIP_THRESHOLD: f32 = 120.0;
pub const MIN_CANDIDATE_LENGTH: usize = 4;
pub const ENTROPY_THRESHOLD: f32 = 3.5;
pub const ENTROPY_MIN_LENGTH: usize = 8;

// Bitmask flags — u16 is sufficient (9 bits used)
pub const FLAG_HAS_UPPER: u16          = 0b0000_0000_0001;  // 1
pub const FLAG_HAS_LOWER: u16          = 0b0000_0000_0010;  // 2
pub const FLAG_HAS_DIGIT: u16          = 0b0000_0000_0100;  // 4
pub const FLAG_HAS_SPECIAL: u16        = 0b0000_0000_1000;  // 8
pub const FLAG_IS_PASSWORD_HEADER: u16 = 0b0000_0001_0000;  // 16
pub const FLAG_IS_USERNAME_HEADER: u16 = 0b0000_0010_0000;  // 32
pub const FLAG_IS_URL_HEADER: u16      = 0b0000_0100_0000;  // 64
pub const FLAG_HAS_FORMULA: u16        = 0b0000_1000_0000;  // 128
pub const FLAG_HAS_COMMENT: u16        = 0b0001_0000_0000;  // 256
```

### Enums

```rust
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellType {
    Empty   = 0,
    Header  = 1,
    Value   = 2,
    Section = 3,
}

impl Default for CellType {
    fn default() -> Self { CellType::Value }
}
```

### Structs

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct Relationship {
    pub header_cell_id: u32,
    pub value_cell_id: u32,
    pub key: String,
    pub value: String,
    pub confidence: f32,
    pub reason: String,
}

#[derive(Debug, Clone)]
pub struct Region {
    pub region_id: u32,
    pub cell_ids: Vec<u32>,
    pub header_ids: Vec<u32>,
    pub value_ids: Vec<u32>,
}
```

---

## Module: store.rs

```
SPEC PHASE:  Data Structures (CellStore) + Phase 1 construction
PROMPT:      02
DEPENDS ON:  types.rs
DEPENDED BY: every module except types.rs
```

### Input type (received from Python via FFI)

```rust
#[derive(Debug, Clone)]
pub struct RawCell {
    pub row: u32,
    pub col: u32,
    pub value: String,
    pub formula: String,
    pub comment: String,
    pub sheet_name: String,
    pub is_merged_origin: bool,
}
```

### CellStore (SoA layout)

```rust
pub struct CellStore {
    // Parallel arrays — index i is one cell
    pub rows: Vec<u32>,
    pub cols: Vec<u32>,
    pub values: Vec<String>,
    pub normalized_values: Vec<String>,     // filled by features.rs
    pub cell_types: Vec<CellType>,          // default Value, set by candidates.rs
    pub entropy: Vec<f32>,                  // filled by features.rs
    pub feature_flags: Vec<u16>,            // filled by features.rs
    pub region_ids: Vec<i32>,               // -1 until regions.rs
    pub formulas: Vec<String>,
    pub comments: Vec<String>,
    pub merged_flags: Vec<bool>,
    pub sheet_names: Vec<String>,

    // Spatial index — built during from_raw
    pub coord_to_id: AHashMap<(u32, u32), u32>,
}
```

### Critical implementation notes

```
FIX 3 — Comment merge:
  In from_raw(), if (row, col) already exists in coord_to_id:
    → update comments[existing_id] with new comment
    → set feature_flags[existing_id] |= FLAG_HAS_COMMENT
    → do NOT append a new row

FIX 4 — Single pass:
  Formula detection: if value.starts_with('='), store as formula.
  No second workbook open needed — openpyxl with data_only=False
  gives both the formula string and (if cached) the computed value.
  This is handled in Python extraction, not in Rust.
```

### Methods

```rust
impl CellStore {
    pub fn from_raw(cells: Vec<RawCell>) -> Self
    pub fn len(&self) -> usize
    pub fn is_empty(&self) -> bool
}
```

---

## Module: spatial.rs

```
SPEC PHASE:  build_distance_table (Constants) + Phase 4 (query_radius)
PROMPT:      03
DEPENDS ON:  store.rs (CellStore for coord_to_id)
DEPENDED BY: scoring.rs, inference.rs, regions.rs
```

### DistanceTable

```rust
pub struct DistanceTable {
    data: [[f32; 7]; 7],  // (2*RADIUS+1) x (2*RADIUS+1)
}

impl DistanceTable {
    pub fn new() -> Self          // builds from spec scoring rules
    pub fn score(&self, dr: i32, dc: i32) -> f32
}
```

```
GAP 2 FIX — Index offset:
  Access: self.data[(dr + RADIUS) as usize][(dc + RADIUS) as usize]
  Negative dr/dc are valid. Without the offset, negative indices
  silently wrap in array access giving wrong scores.

Scoring values (from spec):
  (dr=0, dc=1)  → 100  right adjacent
  (dr=0, dc=-1) →  80  left adjacent
  (dr=1, dc=0)  →  95  below adjacent
  (dr=-1, dc=0) →  70  above adjacent
  (dr=0, dc=0)  →   0  self (not used)
  |dr|==1, |dc|==1 → 70  diagonal
  dc=0, dr>1    → max(0, 90 - (dr-1)*15)
  dr=0, dc>1    → max(0, 90 - (dc-1)*15)
  else           → max(0, 60 - sqrt(dr²+dc²)*10)
```

### query_radius

```rust
pub fn query_radius(
    store: &CellStore,
    row: u32,
    col: u32,
    radius: i32,
) -> Vec<u32>
```

```
FIX 2 — Index over ALL cells:
  query_radius reads from store.coord_to_id which contains every cell,
  not just candidates. A plain VALUE cell adjacent to a password header
  must be discoverable even though it was never flagged suspicious.

  The spatial index IS coord_to_id, built during from_raw.
  No separate build step needed.
```

---

## Module: features.rs

```
SPEC PHASE:  Phase 2 — Feature precomputation
PROMPT:      04
DEPENDS ON:  store.rs, types.rs (constants + flags)
DEPENDED BY: candidates.rs (uses computed flags + entropy)
```

### Header keyword sets

```rust
// Built once at init, never rebuilt
lazy_static! {
    static ref PASSWORD_HEADERS: AHashSet<&'static str> = {
        // Multilingual set — at minimum:
        // English: password, pwd, pass, passwd, passphrase, secret, pin, passcode
        // German: passwort, kennwort
        // Spanish: contraseña, clave
        // French: motdepasse
        // Portuguese: senha
        // Russian: пароль
        // Japanese: パスワード
        // Chinese: 密码
        // Korean: 비밀번호
        // Dutch: wachtwoord
        // Polish: hasło
        // Swedish: lösenord
        // Italian: parola d'ordine → paroladordine (normalized)
        // + common normalized forms without separators
    };
    static ref USERNAME_HEADERS: AHashSet<&'static str> = { ... };
    static ref URL_HEADERS: AHashSet<&'static str> = { ... };
}
```

```
WIN 2 — Single normalization:
  normalize() runs exactly once per cell in precompute_features.
  The result is stored in normalized_values[i].
  No downstream code ever calls normalize() again.

GAP 3 FIX — Entropy pre-filter:
  Before computing entropy (float math with log2), check bitmask:
    flags & FLAG_HAS_UPPER AND
    flags & FLAG_HAS_LOWER AND
    flags & FLAG_HAS_DIGIT AND
    len >= ENTROPY_MIN_LENGTH
  Most cells fail this check. Skip entropy entirely for them.
```

### Functions

```rust
pub fn normalize(value: &str) -> String
    // lowercase, strip, collapse [\s_\-:=]+ to empty string

pub fn compute_entropy(text: &str) -> f32
    // Shannon entropy: -Σ (count/len) * log2(count/len)

pub fn precompute_features(store: &mut CellStore)
    // Single pass over all cells:
    //   1. normalize → store in normalized_values
    //   2. header detection (lookup in PASSWORD/USERNAME/URL sets)
    //   3. character class flags (upper/lower/digit/special)
    //   4. formula/comment flags (check non-empty strings)
    //   5. entropy (only if pre-filter passes)
```

---

## Module: candidates.rs

```
SPEC PHASE:  Phase 3 (reduce) + Phase 5 (classify)
PROMPT:      05
DEPENDS ON:  features.rs (flags computed), types.rs
DEPENDED BY: regions.rs, detection.rs, inference.rs
```

### Compiled regex patterns

```rust
lazy_static! {
    pub(crate) static ref INLINE_CREDENTIAL_REGEX: Regex =
        Regex::new(r"(?i)(?:password|pwd|pass|passwd|passwort|contraseña|motdepasse|senha|пароль|secret|pin)[\s:=]+(.+)").unwrap();

    pub(crate) static ref FORMULA_STRING_REGEX: Regex =
        Regex::new(r#""([^"]*)""#).unwrap();

    pub(crate) static ref FORMULA_KEYWORD_REGEX: Regex =
        Regex::new(r"(?i)(?:password|pwd|pass|passwd|passwort|contraseña|motdepasse|senha|пароль|secret|pin)[\s:=]*$").unwrap();
}
```

```
WIN 3 — Vectorized regex:
  Compile patterns once (lazy_static), apply in a single loop.
  Never compile inside a function body.
```

### Functions

```rust
pub fn reduce_candidate_space(store: &CellStore) -> Vec<u32>
    // Returns cell_ids that are suspicious:
    //   - FLAG_IS_PASSWORD_HEADER set
    //   - FLAG_HAS_FORMULA or FLAG_HAS_COMMENT set
    //   - INLINE_CREDENTIAL_REGEX matches cell value
    //   - entropy > ENTROPY_THRESHOLD and len >= MIN_CANDIDATE_LENGTH

pub fn classify_cells(store: &mut CellStore, candidate_ids: &[u32])
    // Sets cell_types on candidate cells:
    //   - FLAG_IS_PASSWORD_HEADER → CellType::Header
    //   - is_section_title() → CellType::Section
    //   - else → CellType::Value (already default, but explicit)

fn is_section_title(store: &CellStore, cell_id: u32, row_pop: &AHashMap<u32, u32>) -> bool
    // row_population[row] <= 2
    // len(value) <= 60
    // no special chars [=@#$%^&*{}[\]]
    // alpha chars > 50% of total
```

---

## Module: regions.rs

```
SPEC PHASE:  Phase 6 — Region detection
PROMPT:      06
DEPENDS ON:  store.rs, spatial.rs (query_radius), candidates.rs
DEPENDED BY: inference.rs (same_region scoring)
```

### Functions

```rust
pub fn detect_regions(
    store: &mut CellStore,
    candidate_ids: &[u32],
) -> Vec<Region>
```

```
FIX 1 — VecDeque not Vec:
  BFS queue uses std::collections::VecDeque.
  VecDeque::pop_front() is O(1).
  Vec::remove(0) is O(n) — this was a bug in the original spec.

Algorithm:
  For each unvisited candidate cell:
    1. Create new Region with next region_id
    2. Seed BFS queue with cell_id
    3. While queue not empty:
       a. Pop front
       b. Set store.region_ids[cell_id] = region_id
       c. Add to region.cell_ids (and header_ids/value_ids by type)
       d. query_radius for neighbors
       e. For each neighbor: if unvisited AND in candidate_ids set → enqueue
    4. Push completed region

Note: "in candidate_ids set" — convert candidate_ids to AHashSet<u32>
for O(1) membership check inside the BFS loop.
```

---

## Module: detection.rs

```
SPEC PHASE:  Phase 7 (inline) + Phase 8 (formulas) + Phase 9 (comments)
PROMPT:      07
DEPENDS ON:  store.rs, candidates.rs (regex patterns), types.rs
DEPENDED BY: pipeline.rs
```

### Functions

```rust
pub fn detect_inline_credentials(
    store: &CellStore,
    candidate_ids: &[u32],
) -> Vec<Relationship>
    // Confidence: 250.0
    // Reason: "inline_same_cell"
    // Key: everything before captured group, stripped of :=-> etc.
    // Value: captured group, stripped of quotes, must be >= MIN_CANDIDATE_LENGTH

pub fn analyze_formulas(
    store: &CellStore,
    candidate_ids: &[u32],
) -> Vec<Relationship>
    // Strategy 1: concatenate quoted strings, try inline match
    //   Confidence: 240.0, Reason: "formula_concatenated"
    // Strategy 2: keyword string followed by value string(s)
    //   Confidence: 230.0, Reason: "formula_split_strings"

pub fn analyze_comments(
    store: &CellStore,
    candidate_ids: &[u32],
) -> Vec<Relationship>
    // Mode 1: cell is password header, comment is the credential
    //   Confidence: 230.0, Reason: "comment_on_header_cell"
    // Mode 2: comment text contains credential pattern
    //   Confidence: 220.0, Reason: "comment_contains_credential"
```

---

## Module: scoring.rs

```
SPEC PHASE:  Phase 11 — Scoring engine
PROMPT:      08
DEPENDS ON:  spatial.rs (DistanceTable, query_radius), types.rs
DEPENDED BY: inference.rs
```

### Functions

```rust
pub fn score_candidate(
    store: &CellStore,
    header_id: u32,
    candidate_id: u32,
    dist_table: &DistanceTable,
) -> (f32, String)
```

### Score components (from spec)

```
DISTANCE:       dist_table.score(dr, dc)                 0–100
LENGTH:         +20 if len(value) >= 8
CHAR CLASSES:   +10 each for upper, lower, digit, special  0–40
ENTROPY:        +25 if entropy > ENTROPY_THRESHOLD
CONTEXT:        +30 if same region (region_ids match, != -1)
USERNAME NEAR:  +30 if FLAG_IS_USERNAME_HEADER in RADIUS
URL NEAR:       +20 if FLAG_IS_URL_HEADER in RADIUS
PENALTY HEADER: -80 if candidate has FLAG_IS_PASSWORD_HEADER or FLAG_IS_USERNAME_HEADER
PENALTY SHORT:  -50 if len(value) < MIN_CANDIDATE_LENGTH

Maximum possible: 100 + 20 + 40 + 25 + 30 + 30 + 20 = 265
Threshold to accept: 120.0
```

---

## Module: inference.rs

```
SPEC PHASE:  Phase 10 — Relationship inference (spatial)
PROMPT:      09
DEPENDS ON:  scoring.rs, spatial.rs, store.rs, regions.rs
DEPENDED BY: pipeline.rs
```

### Functions

```rust
pub fn infer_relationships(
    store: &CellStore,
    dist_table: &DistanceTable,
) -> Vec<Relationship>
    // For each cell with FLAG_IS_PASSWORD_HEADER:
    //   1. Try detect_split_password — if found, emit with confidence 200.0
    //   2. Otherwise, query_radius for neighbors
    //   3. Filter to TYPE_VALUE, not already used
    //   4. Score each via score_candidate
    //   5. Pick best, accept if >= RELATIONSHIP_THRESHOLD
    //   6. Mark value cell as used (prevent double-assignment)

fn detect_split_password(
    store: &CellStore,
    header_id: u32,
) -> Option<(String, u32)>
    // Walk up to 5 cells directly below header (same column)
    // Each cell must be TYPE_VALUE, non-empty, not a password header
    // If >= 2 parts, each < 30 chars, combined >= 6 chars → return concatenated
    // Returns (combined_value, first_below_cell_id)
```

---

## Module: pipeline.rs

```
SPEC PHASE:  Phase 12 (dedup) + Phase 13 (parallel) + Phase 14 (entry)
PROMPT:      10
DEPENDS ON:  every other gridmap-core module
DEPENDED BY: gridmap-py FFI
```

### Functions

```rust
pub fn deduplicate(relationships: Vec<Relationship>) -> Vec<Relationship>
    // Group by value string. Keep highest confidence per value.

pub fn process_sheet(cells: Vec<RawCell>) -> Vec<Relationship>
    // Full single-sheet pipeline:
    //   CellStore::from_raw → precompute_features → reduce_candidate_space
    //   → classify_cells → detect_regions → detect_inline_credentials
    //   → analyze_formulas → analyze_comments → infer_relationships
    //   → deduplicate all results

pub fn process_workbook(sheets: Vec<Vec<RawCell>>) -> Vec<Relationship>
    // Rayon par_iter over sheets → process_sheet each → collect → deduplicate
```

```
GAP 1 FIX — No shared file writes:
  Each sheet's process_sheet runs independently.
  Results are Vec<Relationship> collected by Rayon.
  No mutex, no result queue, no sentinel.
  Final deduplication happens after all sheets complete.
```

---

## Module: gridmap-py/src/lib.rs (FFI boundary)

```
PROMPT:      11
DEPENDS ON:  gridmap-core::pipeline (process_sheet, process_workbook)
DEPENDED BY: Python gridmap package
```

### Type conversion rules

```
Python → Rust:
  list[tuple[int,int,str,str,str,str,bool]] → Vec<RawCell>
  list[list[tuple[...]]] → Vec<Vec<RawCell>>

Rust → Python:
  Vec<Relationship> → list[dict]
  Each dict: {"header_cell_id": int, "value_cell_id": int,
              "key": str, "value": str, "confidence": float, "reason": str}
```

### Panic safety

```
Every #[pyfunction] wraps its body in:
  std::panic::catch_unwind(|| { ... })
    .map_err(|_| PyRuntimeError::new_err("internal engine error"))

Never let a Rust panic propagate across the FFI boundary.
```

---

## Module: python/gridmap/extract.py

```
PROMPT:      12
DEPENDS ON:  openpyxl (external), no Rust dependency
DEPENDED BY: python/gridmap/api.py
```

### Functions

```python
def extract_workbook(filepath: str) -> list[list[tuple]]:
    """Open workbook ONCE with data_only=False. Extract all sheets."""
    # FIX 4: single open, never reopen
    # FIX 3: comment merge by (row,col) — seen dict tracks coordinates

def extract_single_sheet(worksheet, sheet_name: str, sheet_state: str) -> list[tuple]:
    """Extract one sheet into raw cell tuples."""
    # Tuple format: (row, col, value, formula, comment, sheet_name, is_merged_origin)

def collect_merge_origins(worksheet) -> set[tuple[int, int]]:
    """Set of (min_row, min_col) from merged cell ranges."""

def clean_comment(raw_text: str) -> str:
    """Strip and remove 'Comment:\\n' prefix."""
```

---

## Module: python/gridmap/api.py

```
PROMPT:      13
DEPENDS ON:  extract.py, gridmap._core (FFI)
DEPENDED BY: end users
```

### Public API

```python
def load(filepath: str | Path) -> GridDoc:
    """Single entry point. Opens xlsx, extracts, runs engine, returns results."""

@dataclass(frozen=True)
class Relationship:
    key: str
    value: str
    confidence: float
    reason: str
    header_cell_id: int
    value_cell_id: int

class GridDoc:
    filepath: Path
    sheet_count: int
    cell_count: int

    def relationships(self) -> list[Relationship]:
        """All inferred key-value relationships."""

    def credentials(self, min_confidence: float = 0.0) -> list[Relationship]:
        """Convenience filter for credential-type findings."""
```

---

## Data flow (end to end)

```
User calls: gridmap.load("file.xlsx")

  1. api.py opens file via extract.py
     → openpyxl reads workbook (data_only=False, single open)
     → extract_workbook returns list[list[tuple]]

  2. api.py calls gridmap._core.process_workbook(sheets)
     → PyO3 converts Python list[list[tuple]] to Vec<Vec<RawCell>>
     → Rust process_workbook called (one FFI crossing)

  3. Inside Rust (all parallel via Rayon):
     Per sheet:
       from_raw → precompute_features → reduce_candidates
       → classify → detect_regions → inline/formula/comment detection
       → infer_relationships → collect
     After all sheets:
       → deduplicate across all sheets

  4. Rust returns Vec<Relationship>
     → PyO3 converts to list[dict]
     → api.py wraps each dict in Relationship dataclass
     → api.py returns GridDoc

  5. User accesses: doc.relationships() or doc.credentials()
```

---

## Spec fix / gap cross-reference

| Fix/Gap | Where implemented | Prompt |
|---------|-------------------|--------|
| FIX 1 — VecDeque not list.pop(0) | regions.rs | 06 |
| FIX 2 — Spatial index over ALL cells | store.rs (coord_to_id built in from_raw) | 02 |
| FIX 3 — Comment merge by coordinate | store.rs (from_raw duplicate check) | 02 |
| FIX 4 — Single workbook open | extract.py (data_only=False once) | 12 |
| WIN 1 — deque.popleft() | regions.rs (VecDeque::pop_front) | 06 |
| WIN 2 — Single normalization | features.rs (normalize once in precompute) | 04 |
| WIN 3 — Vectorized regex | candidates.rs (lazy_static compiled regex) | 05 |
| GAP 1 — No shared file writes | pipeline.rs (Rayon collect, no mutex) | 10 |
| GAP 2 — Distance table offset | spatial.rs (dr + RADIUS indexing) | 03 |
| GAP 3 — Entropy pre-filter | features.rs (bitmask check before log2) | 04 |
| GAP 4 — Formula deduplication | store.rs + extract.py (single-pass) | 02, 12 |

---

## Rust dependency budget

```toml
# gridmap-core/Cargo.toml — these 4 only, nothing else
[dependencies]
ahash = "0.8"          # faster HashMap (FNV-family hash)
regex = "1"            # compiled patterns for credential matching
rayon = "1.10"         # parallel sheet processing
lazy_static = "1.5"    # one-time init for regex + header sets
```

Every additional dependency must be justified against this budget.
Do not add serde, log, or any other crate unless a prompt explicitly
requires it.
