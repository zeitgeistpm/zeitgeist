from __future__ import annotations

import dataclasses
import re

@dataclasses.dataclass
class Copyright:
    owner: str
    years: list[Years]

    @classmethod
    def from_string(cls, s) -> Copyright:
        """Create ``Copyright`` object from the string ``s``."""
        match = re.match(r"^Copyright ([0-9,\- ]*) (.*)\.$", s)
        if not match:
            raise ParseError()
        years, holder = match.group(1, 2)
        years = years.split(", ")
        years = [Years.from_string(y) for y in years]
        # Check that year ranges don't overlap and are ordered correctly.
        for prev, curr in zip(years, years[1:]):
            if curr.start <= prev.end:
                raise IllegalYearRange()
        return Copyright(holder, years)

    @classmethod
    def from_year(cls, owner: str, year: int) -> Copyright:
        return Copyright(owner, [Years(year)])

    def __str__(self) -> str:
        dates = ", ".join(str(y) for y in self.years)
        return f"Copyright {dates} {self.owner}."

    @property
    def end(self) -> int:
        return self.years[-1].end

    def push_year(self, year: int) -> None:
        """Safely add ``year`` to this copyright."""
        # `year` must not be contained in the copyright yet.
        if year <= self.years[-1].end:
            raise IllegalYearRange()
        if year == self.years[-1].end + 1:
            self.years[-1].end = year
        else:
            self.years.push(Years(year, year))


@dataclasses.dataclass
class Years:
    """A class for inclusive ranges of years."""

    start: int
    end: int = None

    def __post_init__(self) -> None:
        if self.end is None:
            self.end = self.start
        if self.start > self.end:
            raise IllegalYearRange()

    @classmethod
    def from_string(cls, s: str) -> Years:
        # `s` is only a year, e.g. `"2023"`.
        match = re.match(r"^\d{4}$", s)
        if match:
            year = int(s)
            return Years(year, year)
        # `s` is a year range, e.g. `"2022-2023"` 
        match = re.match(r"^(\d{4})-(\d{4})$", s)
        if match:
            start, end = [int(n) for n in match.group(1, 2)]
            if start >= end:
                raise IllegalYearRange()
            return Years(start, end)
        raise ParseError()

    def __str__(self) -> str:
        if self.start == self.end:
            return str(self.start)
        else:
            return f"{self.start}-{self.end}"


class CopyrightError(Exception):
    pass


class ParseError(CopyrightError):
    pass


class IllegalYearRange(CopyrightError):
    pass
