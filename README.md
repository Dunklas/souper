# souper

Souper scans a given directory for software of unknown provinence (SOUP) and writes them to a json-file.
The json-file contains a meta property for each SOUP which you can populate with arbitrary metadata.
If you run souper after a SOUP has been added, removed or updated the json-file will be updated accordingly, without overwriting your arbitrary metadata.

*Why*? 
In order to be compliant with standards such as [IEC 62304](https://en.wikipedia.org/wiki/IEC_62304), you might need to maintain documentation related to software of unknown provinence (SOUP).

Below is an example of how the output looks like, with some arbitrary metadata.


```json
{
    "src/package.json": [
        {
            "name": "react",
            "version": "18.2.0",
            "meta": {
                "purpose": "Enable us to efficiently build single page applications"
            }
        }
    ]
}
```

At the time of writing, souper will attempt to identify SOUPs from the following sources:
 - package.json (npm)
 - *.csproj (ASP.NET)

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
