"""User-facing API for gridmap: load(), GridDoc, and Relationship."""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path

from gridmap import _core
from gridmap.extract import extract_workbook


@dataclass(frozen=True)
class Relationship:
    """A detected key-value relationship between two cells."""

    key: str
    value: str
    confidence: float
    reason: str
    header_cell_id: int
    value_cell_id: int

    def __repr__(self) -> str:
        return f"Relationship({self.key}={self.value} ({self.confidence}))"


class GridDoc:
    """Result of processing an xlsx file through the gridmap engine."""

    def __init__(
        self,
        filepath: Path,
        sheet_count: int,
        cell_count: int,
        rels: list[Relationship],
    ) -> None:
        self.filepath = filepath
        self.sheet_count = sheet_count
        self.cell_count = cell_count
        self._relationships = rels

    def relationships(self) -> list[Relationship]:
        """All inferred key-value relationships."""
        return list(self._relationships)

    def credentials(self, min_confidence: float = 0.0) -> list[Relationship]:
        """Convenience filter returning relationships at or above min_confidence."""
        return [r for r in self._relationships if r.confidence >= min_confidence]


def load(filepath: str | Path) -> GridDoc:
    """Single entry point. Opens an xlsx file, runs the detection engine, and returns results.

    Args:
        filepath: Path to an xlsx file.

    Returns:
        A GridDoc containing all detected relationships.

    Raises:
        FileNotFoundError: If the file does not exist.
        ValueError: If the file does not have an .xlsx extension.
    """
    filepath = Path(filepath)

    if not filepath.exists():
        raise FileNotFoundError(f"File not found: {filepath}")

    if filepath.suffix.lower() != ".xlsx":
        raise ValueError(f"Expected .xlsx file, got: {filepath.suffix}")

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
        rels=rels,
    )
