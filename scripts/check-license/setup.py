from setuptools import setup

setup(
    name="check-license",
    packages=["check_license"],
    package_dir={"": "src"},
    entry_points={"console_scripts": ["check-license = check_license:main"]},
)
