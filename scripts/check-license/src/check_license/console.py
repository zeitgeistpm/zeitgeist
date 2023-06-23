from __future__ import annotations

import click


def echo(msg: str) -> None:
    click.echo(msg)
