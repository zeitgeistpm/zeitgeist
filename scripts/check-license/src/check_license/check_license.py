from __future__ import annotations

import dataclasses
import datetime
import logging
import re
import os

import git

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
        match = re.match(COPYRIGHT_REGEX, s)
        if match:
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
        if True:  # TODO already checked in and modified
            return datetime.datetime.utcfromtimestamp(os.path.getmtime(self._path))
        else:
            return 0

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


class LicenseChecker:
    def __init__(self, git_object: Optional[GitObject] = None) -> None:
        self._git = git_object or GitObject()

    def check_files(
        self,
        year: int,
        files: list[str],
    ) -> bool:
        files = [File(f) for f files]
        return self._check_files_common(files, year)

    def update_files(
        self,
        year: int,
        files: list[str],
    ) -> bool:
        files = [File(f) for f files]
        return self._update_files_common(files, year)

    def check_branch(
        self,
        year: int,
        branch: Optional[str] = None,
        file_filter: Optional[Callable] = None,
    ) -> bool:
        """Check if the copyright notices of files changed in a branch are up to date.

        Args:
            year: The current year.
            branch: The name of the branch (the active branch if ``None``).
            file_filter: A predicate which filters files which should not be tracked.
        """
        files = self._get_files_from_branch(branch, file_filter)
        return self._check_files_common(files, year)

    def update_branch(
        self,
        year: int,
        branch: Optional[str] = None,
        file_filter: Optional[Callable] = None,
    ) -> bool:
        files = self._get_files_from_branch(branch, file_filter)
        return self._update_files_common(files, year)

    def _get_files_from_branch(
        self, branch: Optional[str] = None, file_filter: Optional[Callable] = None
    ) -> list[File]:
        if branch is None:
            branch = self._git.get_active_branch()
        if file_filter is None:
            file_filter = lambda f: f.endswith(".rs")
        files = self._git.get_files_changed_in_branch(branch)
        files = filter(file_filter, files)
        return [File(f) for f in files]

    def _check_files_common(self, files: list[str], year: int) -> bool:
        result = False
        for f in files:
            try:
                f.read()
                f.check(year)
            except LicenseCheckerError as e:
                logging.error(str(e))
                result = True
        return result

    def _update_files_common(self, files: list[str], year: int) -> None:
        result = False
        for f in files:
            try:
                f.read()
                f.update_license(year)
                f.write()
            except LicenseCheckerError as e:
                logging.error(str(e))
                result = True
        return result


class GitObject:
    def __init__(self) -> None:
        g = git.Git()
        path = g.rev_parse("--show-toplevel")
        self._repo = git.Repo(path)

    def get_active_branch(self) -> str:
        return self._repo.active_branch.name

    def get_files_changed_in_branch(self, branch: str) -> list[str]:
        """Return the list of files changed in ``branch``."""
        branch = self._get_branch(branch)
        diff = branch.commit.diff(self._repo.commit("main"))
        # We exclude any deleted files (including renames).
        return [
            d.a_path
            for d in diff
            if not (d.deleted_file or d.raw_rename_from or d.rename_from)
        ]

    def _get_branch(self, branch_name: str):
        return next(b for b in self._repo.branches if b.name == branch_name)
