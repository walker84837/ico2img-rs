# Rust ICO to PNG Converter

Convert ICO images to PNG format using this Rust command-line utility.

## Table of Contents

  - [Installation](#installation)
  - [Usage](#usage)
  - [Support](#support)
  - [Contributing](#contributing)
  - [License](#license)

## Installation

Make sure you have Rust and Cargo installed. Then, you can build and install the
converter using the following steps:

``` console
$ git clone https://github.com/walker84837/ico-to-png-rs.git
$ cd ico-to-png
$ cargo build --release
$ cargo install
```

## Usage

``` console
$ ico-to-png <ICO_FILE> -o <OUTPUT_DIRECTORY> [-i <INDEX>] [--verbose]
```

#### Options

  - `<ICO_FILE>`: The path to the ICO image.
  - `-o`: The output PNG image or directory.
  - `-i`: Index of the image to convert (default is 0).
  - `-v, --verbose`: Enable verbose output.

## Support

If you encounter any issues or have questions, feel free to [open an
issue](https://github.com/walker84837/ico-to-png-rs/issues).

## Contributing

Contributions to the colour-blender-rs project are always welcome\! If you want
to contribute:

  - Format your code with

    ``` console
    $ rustfmt --edition 2021 src/*
    ```

  - Follow the code of conduct, of course.

  - Use Rust stable rather than Rust nightly for compatibility reasons.

  - If you must use an external library, please use lightweight ones (e.g.
    `ureq` over `reqwest`, `async-std` over `tokio`).

  - Use the standard library rather than reinventing the wheel.

  - For major changes (e.g. a new feature), open an issue and describe the
    following points
    
      - Why should it be added? What does it add, and why should it even be
        considered?
      - What's the difference between using it and not using it?

If you need help or guidance with this project, open a new
[issue](https://github.com/walker84837/ico-to-png-rs/issues).

## License

This project is dual-licensed under the [MIT License](LICENSE_MIT.md) or the
[Apache License](LICENSE_APACHE.md), version 2.
