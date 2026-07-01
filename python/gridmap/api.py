"""User-facing API for gridmap: load(), GridDoc, and Relationship."""

from __future__ import annotations

from dataclasses import dataclass, field
from pathlib import Path

from gridmap import _core
from gridmap.extract import extract_workbook


@dataclass(frozen=True)
class Relationship:
    """A detected key-value relationship between two cells.

    Attributes:
        key: The header or label text (e.g., "Password").
        value: The detected credential value.
        confidence: Heuristic confidence score. Values above 120 typically
            indicate real credentials; above 200 indicates high-confidence
            inline or formula-embedded detections.
        reason: Semicolon-separated breakdown of scoring factors
            (e.g., "distance=100;upper;lower;digit;special;length>=8").
        header_cell_id: Internal cell index of the header/label cell.
        value_cell_id: Internal cell index of the value cell.
    """

    key: str
    value: str
    confidence: float
    reason: str
    header_cell_id: int
    value_cell_id: int

    def __repr__(self) -> str:
        return f"Relationship({self.key}={self.value} ({self.confidence}))"


@dataclass(frozen=True)
class GridDoc:
    """Result of processing an xlsx file through the gridmap engine.

    Attributes:
        filepath: Path to the source xlsx file.
        sheet_count: Number of sheets in the workbook.
        cell_count: Total number of non-empty cells across all sheets.
    """

    filepath: Path
    sheet_count: int
    cell_count: int
    _relationships: list[Relationship] = field(repr=False)

    def relationships(self) -> list[Relationship]:
        """Return all inferred key-value relationships.

        Returns:
            A list of all Relationship objects detected in the workbook,
            regardless of confidence score.
        """
        return list(self._relationships)

    def credentials(self, min_confidence: float = 0.0) -> list[Relationship]:
        """Return relationships at or above a minimum confidence threshold.

        Args:
            min_confidence: Minimum confidence score to include.
                Defaults to 0.0 (all relationships). Use 120.0 for
                typical credential filtering.

        Returns:
            A filtered list of Relationship objects with confidence
            >= min_confidence.
        """
        return [r for r in self._relationships if r.confidence >= min_confidence]


def load(filepath: str | Path) -> GridDoc:
    """Load an xlsx file and run the credential detection engine.

    Opens the workbook, extracts all cells (including formulas, comments,
    merged cells, and hidden sheets), sends them through the Rust detection
    pipeline, and returns the results as a GridDoc.

    Args:
        filepath: Path to an xlsx file. Accepts both str and pathlib.Path.

    Returns:
        A GridDoc containing all detected relationships and workbook metadata.

    Raises:
        FileNotFoundError: If the file does not exist.
        ValueError: If the file does not have an .xlsx extension.

    Example::

        import gridmap

        doc = gridmap.load("workbook.xlsx")
        for cred in doc.credentials(min_confidence=120):
            print(f"{cred.key} = {cred.value}")
    """
    filepath = Path(filepath)

    if not filepath.exists():
        raise FileNotFoundError(f"File not found: {filepath}")

    if filepath.suffix.lower() != ".xlsx":
        raise ValueError(f"Expected .xlsx file, got: {filepath.suffix}")

    with open(filepath, "rb") as f:
        magic = f.read(4)
    if magic != b"PK\x03\x04":
        raise ValueError(
            f"File does not appear to be a valid .xlsx file "
            f"(expected ZIP/OOXML signature, got {magic!r})"
        )

    sheets = extract_workbook(filepath)

    sheet_count = len(sheets)
    cell_count = sum(len(s) for s in sheets)

    raw_results = _core.process_workbook(sheets)

    rels = [
        Relationship(
            key=r["key"],
            value=r["value"],
            confidence=r["confidence"],
            reason=r["reason"],
            header_cell_id=r["header_cell_id"],
            value_cell_id=r["value_cell_id"],
        )
        for r in raw_results
    ]

    return GridDoc(
        filepath=filepath,
        sheet_count=sheet_count,
        cell_count=cell_count,
        _relationships=rels,
    )
