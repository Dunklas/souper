# souper

Scans a given folder for software of unknown provinence (SOUP) and dumps them in a json-file.

## Installation

1. Grab binary for your OS from the latest release
2. Extract the downloaded file: `tar xvf <-FILENAME->`
3. Make sure the extracted executable (`souper`) is present in your `PATH`

## Usage

Navigate to to the repository where you'd like to run souper.

`souper --output-file soups.json`

Alternatively, you can run souper from any directory:

`souper --directory /path/to/my/repo --output-file soups.json`

## Create a release

1. Bump version number in [`Cargo.toml`](./Cargo.toml)
2. Create and complete a pull request
3. Tag latest commit: `git tag v6.6.6`
4. Push tags: `git push origin --tags`
