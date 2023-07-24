import pytest

from check_license.copyright import Copyright, Years, CopyrightError


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
        "value",
        [
            "Copyright 2022-2022 same year range.",
            "Copyright missing years.",
            "Copyright 2022-2021 decreasing years.",
            "Copyright 2020-2022, 2022-2023 overlapping years.",
        ],
    )
    def test_from_string_fails(self, value):
        with pytest.raises(CopyrightError):
            Copyright.from_string(value)

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
