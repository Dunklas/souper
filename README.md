# souper

Scans a given directory for [software of unknown provinence (SOUP)](https://en.wikipedia.org/wiki/Software_of_unknown_pedigree) and writes them to a json-file.
The json-file contains name, version and a meta property for each SOUP.
The meta property is a json object which may be populated with arbitrary metadata.
If you run souper after the version of a SOUP has been updated, the json-file will be updated with the new version, while preserving content of the meta property.
If a SOUP has been added or removed, the json-file will be updated accordingly.

*Why*? 
In order to be compliant with standards such as [IEC 62304](https://en.wikipedia.org/wiki/IEC_62304), you might need to maintain documentation related to software of unknown provinence (SOUP).
With souper you can keep this documentation close to your source code and have it updated together with the relevant changes.

An example of how the output may look like can be found [here](./soups.json).

Souper will attempt to identify SOUPs from the following sources:
 - package.json (npm)
 - *.csproj (ASP.NET)
 - Cargo.toml (rust)
 - Dockerfile
    - base images
    - packages installed with apt(-get)


## Installation

1. Grab binary for your OS from the latest release
2. Extract the downloaded file: `tar xvf <-FILENAME->`
3. Make sure the extracted executable is present in your `PATH`

## Usage

Navigate to to the repository where you'd like to run souper.

`souper --output-file soups.json`

Alternatively, you can run souper from any directory:

`souper --directory /path/to/my/repo --output-file soups.json`

### Excluding directories

In case there's a directory that you'd like to skip, use the `--exclude-directory` argument.

`souper --output-file soups.json --exclude-directory ./test/`

### Default meta keys

If you know what properties that you'd like in the meta property, you can have them created automatically by using the `--meta-key` argument.

`souper --output-file soups.json --meta-key requirements --meta-key manufacturer`

## Create a release

1. On your feature branch, bump to a proper version number in [`Cargo.toml`](./Cargo.toml)
2. Create, review and complete a pull request
3. Tag latest commit on the main branch with the version set in [`Cargo.toml`](./Cargo.toml)
    - E.g. `git tag v6.6.6`
4. Push tags
    - `git push origin --tags`
