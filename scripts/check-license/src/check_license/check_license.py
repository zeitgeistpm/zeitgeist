from __future__ import annotations

import dataclasses
import datetime
import re
import os

from check_license.console import echo
from check_license.errors import (
    LicenseCheckerError,
    MissingCopyrightError,
    CopyrightError,
    DuplicateCopyrightError,
    OutdatedCopyrightError,
)

DATE_REGEX = r"(\d{4})-(\d{4})"
COPYRIGHT_REGEX = r"Copyright ([0-9,\- ]*) (.*)\."
FORECASTING_TECH = "Forecasting Technologies LTD"
# TODO Get owner according to exact date
OWNER = FORECASTING_TECH


@dataclasses.dataclass
class Years:
    """A class for inclusive ranges of years."""

    start: int
    end: int = None

    def __post_init__(self) -> None:
        if self.end is None:
            self.end = self.start

    @classmethod
    def from_string(cls, s: str) -> Years:
        try:
            year = int(s)
            return Years(year, year)
        except ValueError:
            pass
        match = re.match(DATE_REGEX, s)
        start, end = match.group(1, 2)
        return Years(int(start), int(end))

    def __str__(self) -> str:
        if self.start == self.end:
            return str(self.start)
        else:
            return f"{self.start}-{self.end}"


@dataclasses.dataclass
class Copyright:
    owner: str
    years: list[Years]

    @classmethod
    def from_string(cls, s) -> Copyright:
        # TODO This could use better error handling!
        match = re.match(COPYRIGHT_REGEX, s)
        assert match
        years, holder = match.group(1, 2)
        years = years.split(", ")
        return Copyright(holder, [Years.from_string(y) for y in years])

    def __str__(self) -> str:
        dates = ", ".join(str(y) for y in self.years)
        return f"Copyright {dates} {self.owner}."

    @property
    def end(self) -> int:
        return self.years[-1].end

    def push_year(self, year: int) -> None:
        """Safely add ``year`` to this copyright."""
        if year == self.years[-1].end + 1:
            self.years[-1].end = year
        else:
            self.years.push(Years(year, year))


class File:
    def __init__(
        self, path: str, copyright_notices: Optional[list] = None, blob: str = ""
    ) -> None:
        self._path = path
        self._copyright_notices = copyright_notices or []
        self._blob = blob

    @property
    def path(self) -> str:
        return self._path

    def last_changed(self) -> datetime.datetime:
        """Return the UTC date at which the file was last changed."""
        # FIXME This doesn't take git into account.
        return datetime.datetime.utcfromtimestamp(os.path.getmtime(self._path))

    def read(self) -> None:
        """Read contents of file to buffer.

        May fail due to broken copyright notices. Should be run before calling any other function.
        """
        raw_copyright = []
        blob = ""
        with open(self._path, "r") as f:
            # We're assuming that all copyright notices come in one bunch, so once
            # we meet a line of whitespace, we give up.
            while (line := f.readline()) and line.startswith("//"):
                if re.match(r"// *$", line):
                    blob += line
                    break
                raw_copyright.append(line[3:])  # Strip "// ".
            blob += f.read()
        for i, s in enumerate(raw_copyright):
            try:
                copyright = Copyright.from_string(s)
            except:
                raise CopyrightError(self._path, i, s)
            self._copyright_notices.append(copyright)
        self._blob = blob

    def check(self, year) -> None:
        """Check that this file's copyright notice reflects changed made in the current
        ``year``."""
        if not self._copyright_notices:
            raise MissingCopyrightError(self._path)
        owner_count = len({c.owner for c in self._copyright_notices})
        if owner_count != len(self._copyright_notices):
            raise DuplicateCopyrightError(self._path)
        # TODO Check that the license blob is as expected

        copyright = self._get_owner_copyright()
        if copyright is None:
            raise MissingCopyrightError(self._path, OWNER)
        if copyright.end < year:
            raise OutdatedCopyrightError(self._path, copyright, year)

    def update_license(self, year) -> bool:
        """Update the copyright notice and return `True` if anything changed."""
        owner_copyright = self._get_owner_copyright()
        if owner_copyright is None:
            self._copyright_notices.insert(0, Copyright(OWNER, [Years(year)]))
            return True
        if owner_copyright.end != year:
            owner_copyright.push_year(year)
            return True
        return False

    def write(self) -> None:
        content = (
            "\n".join(["// " + str(c) for c in self._copyright_notices])
            + "\n"
            + self._blob
        )
        with open(self._path, "w") as f:
            f.write(content)

    def _get_owner_copyright(self) -> Optional[Copyright]:
        matches = (c for c in self._copyright_notices if c.owner == OWNER)
        # `len(matches) < 2` at this point.
        return next(matches, None)


def check_files(year: int, files: list[str]) -> bool:
    files = [File(f) for f in files]
    result = False
    for f in files:
        try:
            f.read()
            f.check(year)
        except LicenseCheckerError as e:
            echo(str(e))
            result = True
    return result


def update_files(year: int, files: list[str]) -> bool:
    files = [File(f) for f in files]
    result = False
    for f in files:
        try:
            f.read()
            f.update_license(year)
            f.write()
        except LicenseCheckerError as e:
            echo(str(e))
            result = True
    return result
