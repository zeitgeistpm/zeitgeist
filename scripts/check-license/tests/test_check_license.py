import builtins
import textwrap

import pytest

from check_license.check_license import Copyright, Years, File
from check_license.errors import *


class TestCopyright:
    @pytest.mark.parametrize(
        "value, holder, years",
        [
            (
                "Copyright 2020 Copyright Holder.",
                "Copyright Holder",
                [Years(2020)],
            ),
            (
                "Copyright 2020-2021 Copyright Holder.",
                "Copyright Holder",
                [Years(2020, 2021)],
            ),
            (
                "Copyright 2020-2021, 2023 Copyright Holder.",
                "Copyright Holder",
                [Years(2020, 2021), Years(2023)],
            ),
        ],
    )
    def test_from_string(self, value, holder, years):
        actual = Copyright.from_string(value)
        assert actual == Copyright(holder, years)

    @pytest.mark.parametrize(
        "copyright, expected",
        [
            (Copyright("Holder", [Years(2020)]), "Copyright 2020 Holder."),
            (Copyright("Holder", [Years(2020, 2021)]), "Copyright 2020-2021 Holder."),
            (
                Copyright("Holder", [Years(2020, 2021), Years(2023)]),
                "Copyright 2020-2021, 2023 Holder.",
            ),
        ],
    )
    def test_str(self, copyright, expected):
        assert str(copyright) == expected


class TestFile:
    def test_read(self):
        path_to_file = "resources/test_read"
        file = File(path_to_file)
        file.read()
        assert file.path == path_to_file
        assert file._copyright_notices == [
            Copyright("Holder", [Years(2020, 2021), Years(2023)]),
            Copyright("This other guy", [Years(1999)]),
        ]
        assert (
            file._blob
            == "//\n// This is the license.\n\nThis is the rest of the file!\n"
        )

    def test_read_fails_on_broken_copyright_notice(self):
        path_to_file = "resources/test_read_fails_on_broken_copyright_notice"
        file = File(path_to_file)
        with pytest.raises(CopyrightError):
            file.read()

    def test_check_success_or_outdated(self):
        file = File(
            "path/to/file",
            [
                Copyright("Forecasting Technologies LTD", [Years(2023)]),
                Copyright("Zeitgeist PM LLC", [Years(2021, 2022)]),
            ],
            "blob",
        )
        file.check(2023)
        with pytest.raises(OutdatedCopyrightError):
            file.check(2024)

    @pytest.mark.parametrize(
        "copyright_notices, year, error",
        [
            (
                [
                    Copyright.from_string(
                        "Copyright 2023 Forecasting Technologies LTD."
                    ),
                    Copyright.from_string("Copyright 2021-2022 Zeitgeist PM LLC."),
                    Copyright.from_string(
                        "Copyright 2022 Forecasting Technologies LTD."
                    ),
                ],
                2023,
                DuplicateCopyrightError,
            ),
            (
                [Copyright.from_string("Copyright 2023 some dude.")],
                2023,
                MissingCopyrightError,
            ),
            (
                [Copyright.from_string("Copyright 2022 Forecasting Technologies LTD.")],
                2023,
                OutdatedCopyrightError,
            ),
        ],
    )
    def test_check_fails(self, copyright_notices, year, error):
        file = File(
            "path/to/file",
            copyright_notices,
            "blob",
        )
        with pytest.raises(error):
            file.check(year)

    @pytest.mark.parametrize(
        "before, after, year, expected",
        [
            (
                [
                    Copyright.from_string(
                        "Copyright 2023 Forecasting Technologies LTD."
                    ),
                    Copyright.from_string("Copyright 2019-2021, 2023 Someone."),
                ],
                [
                    Copyright.from_string(
                        "Copyright 2023 Forecasting Technologies LTD."
                    ),
                    Copyright.from_string("Copyright 2019-2021, 2023 Someone."),
                ],
                2023,
                False,
            ),
            (
                [
                    Copyright.from_string("Copyright 2019-2021, 2023 Someone."),
                ],
                [
                    Copyright.from_string(
                        "Copyright 2023 Forecasting Technologies LTD."
                    ),
                    Copyright.from_string("Copyright 2019-2021, 2023 Someone."),
                ],
                2023,
                True,
            ),
            (
                [
                    Copyright.from_string("Copyright 2019-2021, 2023 Someone."),
                    Copyright.from_string(
                        "Copyright 2022 Forecasting Technologies LTD."
                    ),
                ],
                [
                    Copyright.from_string("Copyright 2019-2021, 2023 Someone."),
                    Copyright.from_string(
                        "Copyright 2022-2023 Forecasting Technologies LTD."
                    ),
                ],
                2023,
                True,
            ),
        ],
    )
    def test_update_license(self, before, after, year, expected):
        blob = "blob"
        file = File("path/to/file", before, blob)
        result = file.update_license(year)
        assert file.path == "path/to/file"
        assert file._copyright_notices == after
        assert file._blob == blob
        assert result == expected

    def test_write(self, mocker, monkeypatch):
        mock_manager = mocker.Mock(__enter__=mocker.Mock(), __exit__=mocker.Mock())
        mock_open = mocker.Mock(return_value=mock_manager)
        monkeypatch.setattr(builtins, "open", mock_open)

        path_to_file = "path/to/file"
        file = File(
            path_to_file,
            [
                Copyright.from_string("Copyright 2022-2023 New Company."),
                Copyright.from_string("Copyright 2021-2022 Old Company."),
            ],
            textwrap.dedent(
                """\
                //
                // The license

                The rest of the file.
                """,
            ),
        )
        file.write()

        expected = textwrap.dedent(
            """\
            // Copyright 2022-2023 New Company.
            // Copyright 2021-2022 Old Company.
            //
            // The license

            The rest of the file.
            """,
        )
        mock_open.assert_called_once_with(path_to_file, "w")
        mock_manager.__enter__().write.assert_called_once_with(expected)
