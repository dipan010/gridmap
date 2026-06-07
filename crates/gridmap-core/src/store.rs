use ahash::AHashMap;

use crate::types::CellType;

// ---------- Raw input from Python ----------

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

// ---------- SoA cell store ----------

#[derive(Debug, Clone)]
pub struct CellStore {
    pub rows: Vec<u32>,
    pub cols: Vec<u32>,
    pub values: Vec<String>,
    pub formulas: Vec<String>,
    pub comments: Vec<String>,
    pub sheet_names: Vec<String>,
    pub is_merged_origin: Vec<bool>,
    pub cell_types: Vec<CellType>,
    pub normalized_values: Vec<String>,
    pub feature_flags: Vec<u16>,
    pub entropy: Vec<f32>,
    pub region_ids: Vec<i32>,
    pub coord_to_id: AHashMap<(u32, u32), u32>,
}

impl CellStore {
    pub fn from_raw(cells: Vec<RawCell>) -> Self {
        let mut store = CellStore {
            rows: Vec::with_capacity(cells.len()),
            cols: Vec::with_capacity(cells.len()),
            values: Vec::with_capacity(cells.len()),
            formulas: Vec::with_capacity(cells.len()),
            comments: Vec::with_capacity(cells.len()),
            sheet_names: Vec::with_capacity(cells.len()),
            is_merged_origin: Vec::with_capacity(cells.len()),
            cell_types: Vec::with_capacity(cells.len()),
            normalized_values: Vec::with_capacity(cells.len()),
            feature_flags: Vec::with_capacity(cells.len()),
            entropy: Vec::with_capacity(cells.len()),
            region_ids: Vec::with_capacity(cells.len()),
            coord_to_id: AHashMap::with_capacity(cells.len()),
        };

        for cell in cells {
            let key = (cell.row, cell.col);
            if let Some(&existing_id) = store.coord_to_id.get(&key) {
                // FIX 3: merge comment into existing slot
                let id = existing_id as usize;
                if !cell.comment.is_empty() {
                    let existing = &mut store.comments[id];
                    if existing.is_empty() {
                        *existing = cell.comment;
                    } else {
                        existing.push('\n');
                        existing.push_str(&cell.comment);
                    }
                }
                continue;
            }

            let id = store.rows.len() as u32;
            store.coord_to_id.insert(key, id);

            store.rows.push(cell.row);
            store.cols.push(cell.col);
            store.values.push(cell.value);
            store.formulas.push(cell.formula);
            store.comments.push(cell.comment);
            store.sheet_names.push(cell.sheet_name);
            store.is_merged_origin.push(cell.is_merged_origin);
            store.cell_types.push(CellType::Value);
            store.normalized_values.push(String::new());
            store.feature_flags.push(0);
            store.entropy.push(0.0);
            store.region_ids.push(-1);
        }

        store
    }

    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_raw(row: u32, col: u32, value: &str, comment: &str) -> RawCell {
        RawCell {
            row,
            col,
            value: value.into(),
            formula: String::new(),
            comment: comment.into(),
            sheet_name: "Sheet1".into(),
            is_merged_origin: false,
        }
    }

    #[test]
    fn from_raw_basic() {
        let cells: Vec<RawCell> = (0..5)
            .map(|i| make_raw(i, 0, &format!("v{i}"), ""))
            .collect();
        let store = CellStore::from_raw(cells);
        assert_eq!(store.len(), 5);
        assert!(!store.is_empty());
    }

    #[test]
    fn from_raw_dedup_coords() {
        let cells = vec![
            make_raw(0, 0, "first", ""),
            make_raw(0, 0, "duplicate", ""),
            make_raw(1, 0, "other", ""),
        ];
        let store = CellStore::from_raw(cells);
        assert_eq!(store.len(), 2);
        // First occurrence wins for the value
        assert_eq!(store.values[0], "first");
    }

    #[test]
    fn duplicate_coord_merges_comment() {
        let cells = vec![
            make_raw(0, 0, "val", "note1"),
            make_raw(0, 0, "val", "note2"),
        ];
        let store = CellStore::from_raw(cells);
        assert_eq!(store.len(), 1);
        assert_eq!(store.comments[0], "note1\nnote2");
    }

    #[test]
    fn duplicate_coord_empty_comment_no_merge() {
        let cells = vec![
            make_raw(0, 0, "val", "original"),
            make_raw(0, 0, "val", ""),
        ];
        let store = CellStore::from_raw(cells);
        assert_eq!(store.comments[0], "original");
    }

    #[test]
    fn coord_to_id_lookup() {
        let cells = vec![
            make_raw(3, 7, "target", ""),
            make_raw(0, 0, "origin", ""),
        ];
        let store = CellStore::from_raw(cells);
        let id = store.coord_to_id[&(3, 7)];
        assert_eq!(id, 0);
        assert_eq!(store.values[id as usize], "target");
    }

    #[test]
    fn from_raw_empty_input() {
        let store = CellStore::from_raw(vec![]);
        assert_eq!(store.len(), 0);
        assert!(store.is_empty());
    }
}
