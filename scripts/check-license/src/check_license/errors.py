from __future__ import annotations


class LicenseCheckerError(Exception):
    pass


class MissingCopyrightError(LicenseCheckerError):
    def __init__(self, path: str, holder: str = "") -> None:
        if holder:
            msg = f"{path}: no copyright notice for {holder} found"
        else:
            msg = f"{path}: no copyright notice found"
        super().__init__(msg)


class IllegalCopyrightError(LicenseCheckerError):
    def __init__(self, path: str, number: int, line: str) -> None:
        msg = f"{path}:{number}: expected copyright notice, found '{line}'"
        super().__init__(msg)


class DuplicateCopyrightError(LicenseCheckerError):
    def __init__(self, path: str) -> None:
        msg = f"{path}: duplicate copyright notice"
        super().__init__(msg)


class OutdatedCopyrightError(LicenseCheckerError):
    def __init__(self, path: str, actual: Copyright, year: int) -> None:
        msg = f"{path}: year {year} missing from copyright notice '{actual}'"
        super().__init__(msg)
