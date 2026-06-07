// ---------- Cell classification ----------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CellType {
    Empty = 0,
    Header = 1,
    Value = 2,
    Section = 3,
}

// ---------- Relationship ----------

#[derive(Debug, Clone, PartialEq)]
pub struct Relationship {
    pub header_cell_id: usize,
    pub value_cell_id: usize,
    pub key: String,
    pub value: String,
    pub confidence: f32,
    pub reason: String,
}

// ---------- Region ----------

#[derive(Debug, Clone)]
pub struct Region {
    pub region_id: usize,
    pub cell_ids: Vec<usize>,
    pub header_ids: Vec<usize>,
    pub value_ids: Vec<usize>,
}

// ---------- Spatial / scoring constants ----------

pub const NEIGHBOR_RADIUS: i32 = 3;
pub const RELATIONSHIP_THRESHOLD: f32 = 120.0;
pub const MIN_CANDIDATE_LENGTH: usize = 4;

// ---------- Entropy constants ----------

pub const ENTROPY_THRESHOLD: f32 = 3.5;
pub const ENTROPY_MIN_LENGTH: usize = 8;

// ---------- Feature-flag bitmasks (u16, 9 bits used) ----------

pub const FLAG_HAS_UPPER: u16 = 0b0000_0001;
pub const FLAG_HAS_LOWER: u16 = 0b0000_0010;
pub const FLAG_HAS_DIGIT: u16 = 0b0000_0100;
pub const FLAG_HAS_SPECIAL: u16 = 0b0000_1000;
pub const FLAG_IS_SHORT: u16 = 0b0001_0000;
pub const FLAG_IS_NUMERIC: u16 = 0b0010_0000;
pub const FLAG_IS_ALPHA: u16 = 0b0100_0000;
pub const FLAG_HAS_COLON: u16 = 0b1000_0000;
pub const FLAG_IS_EMPTY: u16 = 0b0_0001_0000_0000;
pub const FLAG_HAS_FORMULA: u16 = 0b0_0010_0000_0000;
pub const FLAG_IS_PASSWORD_HEADER: u16 = 0b0_0100_0000_0000;
pub const FLAG_IS_USERNAME_HEADER: u16 = 0b0_1000_0000_0000;
pub const FLAG_IS_URL_HEADER: u16 = 0b01_0000_0000_0000;
pub const FLAG_HAS_COMMENT: u16 = 0b10_0000_0000_0000;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relationship_field_access() {
        let rel = Relationship {
            header_cell_id: 0,
            value_cell_id: 1,
            key: "Name".into(),
            value: "Alice".into(),
            confidence: 0.95,
            reason: "adjacent".into(),
        };
        assert_eq!(rel.header_cell_id, 0);
        assert_eq!(rel.value_cell_id, 1);
        assert_eq!(rel.key, "Name");
        assert_eq!(rel.value, "Alice");
        assert!((rel.confidence - 0.95).abs() < f32::EPSILON);
        assert_eq!(rel.reason, "adjacent");
    }

    #[test]
    fn cell_type_repr() {
        assert_eq!(CellType::Empty as u8, 0);
        assert_eq!(CellType::Header as u8, 1);
        assert_eq!(CellType::Value as u8, 2);
        assert_eq!(CellType::Section as u8, 3);
    }

    #[test]
    fn flags_are_distinct_bits() {
        let all = FLAG_HAS_UPPER
            | FLAG_HAS_LOWER
            | FLAG_HAS_DIGIT
            | FLAG_HAS_SPECIAL
            | FLAG_IS_SHORT
            | FLAG_IS_NUMERIC
            | FLAG_IS_ALPHA
            | FLAG_HAS_COLON
            | FLAG_IS_EMPTY
            | FLAG_HAS_FORMULA
            | FLAG_IS_PASSWORD_HEADER
            | FLAG_IS_USERNAME_HEADER
            | FLAG_IS_URL_HEADER
            | FLAG_HAS_COMMENT;
        assert_eq!(all, 0b11_1111_1111_1111);
    }
}
