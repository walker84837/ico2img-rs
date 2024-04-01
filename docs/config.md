# Configuration

This document outlines the configuration options available for the ICO to Image
converter tool.

## Command-line arguments

The converter tool accepts the following command-line arguments related to
configuration:

  - **config**: (Optional) The path to a TOML configuration file for additional
    customization.

## Configuration file

If provided, a TOML configuration file overrides command-line arguments for
customized conversion settings. The configuration file should have the following
structure:

``` toml
[ico2img]
format = "ext"
```

Where `format` specifies the output image format. Supported formats include:

  - `png`
  - `jpg` (or `jpeg`)
  - `bmp`
  - `webp`

If a configuration file isn't provided, and no formats are provided, the program
defaults to PNG.

## Examples

``` toml
[ico2img]
format = "webp"
```

This configuration file specifies the output format as WebP.

-----

The documentation is licensed under the [GNU Free Documentation License 1.3](LICENSE.md).
