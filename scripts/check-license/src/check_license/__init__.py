import argparse
import datetime
import logging
import sys

from check_license.check_license import check_files, update_files
from check_license.console import echo


def main():
    # TODO Add option to ignore files?
    parser = argparse.ArgumentParser()
    parser.add_argument("files", nargs="*")
    parser.add_argument("-w", "--write", action="store_true")
    args = parser.parse_args(sys.argv[1:])
    current_year = datetime.date.today().year
    if args.write:
        failed = update_files(current_year, args.files)
    else:
        failed = check_files(current_year, args.files)
    if failed:
        sys.exit(1)
    echo("All copyright notices are up to date! 🍉")
    sys.exit(0)
