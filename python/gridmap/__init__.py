"""gridmap — spatial document graph engine for credential detection."""

from gridmap._core import version
from gridmap.api import GridDoc, Relationship, load

__all__ = ["load", "GridDoc", "Relationship", "version"]
