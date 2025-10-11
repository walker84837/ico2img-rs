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
    Unsupported,
}

impl From<String> for SupportedImages {
    fn from(value: String) -> Self {
        match value.to_lowercase().as_str() {
            "png" => Self::Png,
            "jpg" | "jpeg" => Self::Jpeg,
            "bmp" => Self::Bmp,
            "webp" => Self::Webp,
            _ => Self::Unsupported,
        }
    }
}

fn main() -> Result<()> {
    let args = Args::parse();

    if args.verbose {
        simple_logger::init().unwrap();
    }

    let path = &args.file;
    println!("Opening ICO file: {:?}", path);
    let reader = BufReader::new(File::open(path)?);

    println!("Reading ICO directory.");
    let icon_dir = IconDir::read(reader)?;
    let index = args.image_index;
    let mut format = args.format;

    println!(
        "Number of entries in ICO file: {}",
        icon_dir.entries().len()
    );

    // TODO: should i use assert!()?
    if icon_dir.entries().is_empty() {
        bail!("No images found in the ICO file.");
    }

    if index >= icon_dir.entries().len() {
        bail!("Invalid image index: {}.", index);
    }

    let entry = &icon_dir.entries()[index];
    println!(
        "Image details: {}x{} - {} bits per pixel",
        entry.width(),
        entry.height(),
        entry.bits_per_pixel()
    );

    println!("Creating output file: {:?}", &args.output);
    let mut writer = BufWriter::new(File::create(&args.output)?);

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
            .to_string()
            .into();
    }

    if let SupportedImages::Unsupported = format {
        bail!("Unsupported image format");
    }

    println!("Handling ICO file.");
    let buffer = handle_ico(&icon_dir, index)?;

    println!("Writing image to output file.");
    write_image(&mut writer, &buffer, &format)?;

    println!("Image conversion completed successfully.");
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
    // same here, should i use asserts?
    if icon_dir.entries().is_empty() {
        bail!("No images found in the ICO file.");
    } else if index >= icon_dir.entries().len() {
        bail!("Invalid image index: {}.", index);
    }

    let entry = &icon_dir.entries()[index];
    println!(
        "Decoding image at index {}: {}x{} - {} bits per pixel",
        index,
        entry.width(),
        entry.height(),
        entry.bits_per_pixel()
    );
    let image = entry.decode()?;
    let mut buffer = Vec::new();
    image.write_png(&mut buffer)?;
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
            writer.write_all(buffer)?;
        }
        SupportedImages::Jpeg => {
            info!("Writing image in JPEG format.");
            let image = load_from_memory(buffer)?.to_rgb8();
            image.write_to(&mut cursor, ImageFormat::Jpeg)?;
            writer.write_all(&img_buffer)?;
        }
        SupportedImages::Bmp => {
            info!("Writing image in BMP format.");
            let image = load_from_memory(buffer)?;
            image.write_to(&mut cursor, ImageFormat::Bmp)?;
            writer.write_all(&img_buffer)?;
        }
        SupportedImages::Webp => {
            info!("Writing image in WebP format.");
            let image = load_from_memory(buffer)?;
            image.write_to(&mut cursor, ImageFormat::WebP)?;
            writer.write_all(&img_buffer)?;
        }
        SupportedImages::Unsupported => unreachable!(),
    }
    Ok(())
}
