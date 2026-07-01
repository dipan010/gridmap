"""Extraction subpackage — format-specific extractors for gridmap.

Each extractor reads a spreadsheet file and returns a normalized
list[list[tuple]] where each tuple is:
    (row, col, value, formula, comment, sheet_name, is_merged_origin)
"""

from __future__ import annotations

from gridmap.extract.xlsx import (
    clean_comment,
    collect_merge_origins,
    extract_single_sheet,
    extract_xlsx,
)

# Backward-compatible alias used by api.py
extract_workbook = extract_xlsx

__all__ = [
    "clean_comment",
    "collect_merge_origins",
    "extract_single_sheet",
    "extract_xlsx",
    "extract_workbook",
]
