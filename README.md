# ico2img: ICO converter to images

Convert ICO images to other image formats:

  - PNG
  - JPG
  - BMP
  - WebP

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
$ git clone https://github.com/walker84837/ico2img-rs.git
$ cd ico2img
$ cargo build --release
```

## Usage

``` console
$ ico2img <ICO_FILE> -o <OUTPUT_DIRECTORY> -c <CONFIG_FILE> [-i <INDEX>] [--verbose]
```

#### Options

  - `<ICO_FILE>`: The path to the ICO image.
  - `-o`: The output PNG image or directory.
  - `-i`: Index of the image to convert (default is 0).
  - `-f, --format`: The format of the converted image.
  - `-c`: Configuration file path (optional)
  - `-v, --verbose`: Enable verbose output.

## Support

If you encounter any issues or have questions, feel free to [open an
issue](https://github.com/walker84837/ico2img-rs/issues).

## Contributing

Contributions to the ico2img-rs project are always welcome! If you want to
contribute:

  - Format your code with
    
    ``` console
    $ rustfmt --edition 2021 src/*
    ```

  - Follow the [code of conduct](CODE_OF_CONDUCT.md), of course.

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
[issue](https://github.com/walker84837/ico2img-rs/issues).

## License

This project is dual-licensed under the [MIT License](LICENSE_MIT.md) or the
[Apache License](LICENSE_APACHE.md), version 2.
