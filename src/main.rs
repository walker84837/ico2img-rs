use anyhow::{anyhow, bail, Result};
use clap::{Parser, ValueEnum};
use ico::IconDir;
use image::{load_from_memory, ImageFormat};
use log::{error, info};
use std::{
    fs::File,
    io::{BufReader, BufWriter, Cursor, Read, Write},
    path::PathBuf,
};
use toml::Value;

#[derive(Parser)]
struct Args {
    #[arg(help = "The path to the ICO image.")]
    file: PathBuf,

    #[arg(short, help = "The output PNG image.")]
    output: PathBuf,

    #[arg(
        short,
        long = "index",
        default_value_t = 0,
        help = "Index of the image to convert."
    )]
    image_index: usize,

    #[arg(
        short,
        long,
        default_value = "png",
        help = "The format of the resulting converted image."
    )]
    format: SupportedImages,

    #[arg(short, long, help = "Enable verbose output.")]
    verbose: bool,

    #[arg(short, help = "The configuration path")]
    config: Option<PathBuf>,
}

/// Enumeration of supported image output formats
#[derive(ValueEnum, Clone, Debug)]
enum SupportedImages {
    Png,
    Jpeg,
    Bmp,
    Webp,
}

impl std::str::FromStr for SupportedImages {
    type Err = anyhow::Error;

    fn from_str(format: &str) -> Result<Self, Self::Err> {
        match format.to_lowercase().as_str() {
            "png" => Ok(Self::Png),
            "jpg" | "jpeg" => Ok(Self::Jpeg),
            "bmp" => Ok(Self::Bmp),
            "webp" => Ok(Self::Webp),
            _ => {
                error!(
                    "The specified format '{}' is not supported at the moment.",
                    format
                );
                bail!("The specified format is not supported at the moment.")
            }
        }
    }
}

fn main() -> Result<()> {
    let args = Args::parse();

    if args.verbose {
        simple_logger::init().unwrap();
    }

    let path = &args.file;
    info!("Opening ICO file: {:?}", path);
    let reader = BufReader::new(File::open(path).map_err(|e| {
        error!("Failed to open ICO file: {:?}", e);
        e
    })?);

    info!("Reading ICO directory.");
    let icon_dir = IconDir::read(reader).map_err(|e| {
        error!("Failed to read ICO directory: {:?}", e);
        e
    })?;

    let index = args.image_index;
    let mut format = args.format.clone();

    info!(
        "Number of entries in ICO file: {}",
        icon_dir.entries().len()
    );

    if icon_dir.entries().is_empty() {
        error!("No images found in the ICO file.");
        bail!("No images found in the ICO file.");
    }

    if index >= icon_dir.entries().len() {
        error!("Invalid image index: {}.", index);
        bail!("Invalid image index: {}.", index);
    }

    let entry = &icon_dir.entries()[index];
    info!(
        "Image details: {}x{} - {} bits per pixel",
        entry.width(),
        entry.height(),
        entry.bits_per_pixel()
    );

    info!("Creating output file: {:?}", &args.output);
    let mut writer = BufWriter::new(File::create(&args.output).map_err(|e| {
        error!("Failed to create output file: {:?}", e);
        e
    })?);

    if let Some(ref conf) = args.config {
        info!("Loading configuration from: {:?}", conf);
        let mut reader = BufReader::new(File::open(conf).map_err(|e| {
            error!("Failed to open configuration file: {:?}", e);
            e
        })?);
        let mut contents = String::new();
        reader.read_to_string(&mut contents).map_err(|e| {
            error!("Failed to read configuration file: {:?}", e);
            e
        })?;
        let config: Value = toml::from_str(&contents).map_err(|e| {
            error!("Failed to parse configuration file: {:?}", e);
            e
        })?;

        format = config["ico2img"]["format"]
            .as_str()
            .ok_or_else(|| {
                error!("Output format type isn't specified in configuration file.");
                anyhow!("Output format type isn't specified.")
            })?
            .parse()
            .map_err(|e| {
                error!("Failed to parse output format: {:?}", e);
                e
            })?;
    }

    info!("Handling ICO file.");
    let buffer = handle_ico(&icon_dir, index)?;

    info!("Writing image to output file.");
    write_image(&mut writer, &buffer, &format)?;

    info!("Image conversion completed successfully.");
    Ok(())
}

/// Handles the ICO file and returns the image data in PNG format.
///
/// # Arguments
///
/// * `icon_dir` - The ICO directory containing image entries
/// * `index` - The index of the image to convert
///
fn handle_ico(icon_dir: &IconDir, index: usize) -> Result<Vec<u8>> {
    if icon_dir.entries().is_empty() {
        error!("No images found in the ICO file.");
        bail!("No images found in the ICO file.");
    } else if index >= icon_dir.entries().len() {
        error!("Invalid image index: {}.", index);
        bail!("Invalid image index: {}.", index);
    }

    let entry = &icon_dir.entries()[index];
    info!(
        "Decoding image at index {}: {}x{} - {} bits per pixel",
        index,
        entry.width(),
        entry.height(),
        entry.bits_per_pixel()
    );
    let image = entry.decode().map_err(|e| {
        error!("Failed to decode image: {:?}", e);
        e
    })?;
    let mut buffer = Vec::new();
    image.write_png(&mut buffer).map_err(|e| {
        error!("Failed to write image as PNG: {:?}", e);
        e
    })?;
    Ok(buffer)
}

/// Writes the image buffer to the specified writer in the given format.
///
/// # Arguments
///
/// * `writer` - The writer to output the image to
/// * `buffer` - The image data buffer
/// * `format` - The desired output image format
///
fn write_image<W: Write>(writer: &mut W, buffer: &[u8], format: &SupportedImages) -> Result<()> {
    let mut img_buffer = Vec::new();
    let mut cursor = Cursor::new(&mut img_buffer);

    match format {
        SupportedImages::Png => {
            info!("Writing image in PNG format.");
            writer.write_all(buffer).map_err(|e| {
                error!("Failed to write PNG image: {:?}", e);
                e
            })?;
        }
        SupportedImages::Jpeg => {
            info!("Writing image in JPEG format.");
            let image = load_from_memory(buffer)
                .map_err(|e| {
                    error!("Failed to load image from memory: {:?}", e);
                    e
                })?
                .to_rgb8();
            image
                .write_to(&mut cursor, ImageFormat::Jpeg)
                .map_err(|e| {
                    error!("Failed to write JPEG image: {:?}", e);
                    e
                })?;
            writer.write_all(&img_buffer).map_err(|e| {
                error!("Failed to write JPEG image to writer: {:?}", e);
                e
            })?;
        }
        SupportedImages::Bmp => {
            info!("Writing image in BMP format.");
            let image = load_from_memory(buffer).map_err(|e| {
                error!("Failed to load image from memory: {:?}", e);
                e
            })?;
            image.write_to(&mut cursor, ImageFormat::Bmp).map_err(|e| {
                error!("Failed to write BMP image: {:?}", e);
                e
            })?;
            writer.write_all(&img_buffer).map_err(|e| {
                error!("Failed to write BMP image to writer: {:?}", e);
                e
            })?;
        }
        SupportedImages::Webp => {
            info!("Writing image in WebP format.");
            let image = load_from_memory(buffer).map_err(|e| {
                error!("Failed to load image from memory: {:?}", e);
                e
            })?;
            image
                .write_to(&mut cursor, ImageFormat::WebP)
                .map_err(|e| {
                    error!("Failed to write WebP image: {:?}", e);
                    e
                })?;
            writer.write_all(&img_buffer).map_err(|e| {
                error!("Failed to write WebP image to writer: {:?}", e);
                e
            })?;
        }
    }
    Ok(())
}
