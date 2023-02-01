import argparse
import datetime
import logging
import sys

from check_license.check_license import LicenseChecker


def main():
    # TODO Foreign files
    parser = argparse.ArgumentParser()
    parser.add_argument("-b", "--branch")
    parser.add_argument("-w", "--write", action="store_true")
    args = parser.parse_args(sys.argv[1:])
    current_year = datetime.date.today().year
    license_checker = LicenseChecker()
    if args.write:
        failed = license_checker.update_branch(current_year, args.branch)
    else:
        failed = license_checker.check_branch(current_year, args.branch)
    if failed:
        print("check-license: failed!")
        sys.exit(1)
    print("check-license: success!")
    sys.exit(0)
