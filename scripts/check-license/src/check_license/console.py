from __future__ import annotations

import click


def echo(msg: str) -> None:
    click.echo(msg)


def echo_error(msg: str) -> None:
    click.echo(click.style("error: ", fg="red"), nl=False)
    click.echo(msg)
