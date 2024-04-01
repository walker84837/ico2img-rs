use anyhow::{anyhow, Result};
use clap::Parser;
use ico::IconDir;
use image::{load_from_memory, ImageFormat};
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
        help = "Index of the image to convert.",
        default_value = "0"
    )]
    image_index: usize,

    #[arg(
        short,
        long,
        help = "The format of the resulting converted image.",
        default_value = "png"
    )]
    format: String,


    #[arg(short, long, help = "Enable verbose output.")]
    verbose: bool,

    #[arg(short, help = "The configuration path")]
    config: Option<PathBuf>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let path = &args.file;
    let reader = BufReader::new(File::open(path)?);
    let icon_dir = IconDir::read(reader)?;

    let index = args.image_index;
    let verbose = args.verbose;
    let mut format = args.format.as_str();

    if verbose {
        println!(
            "Number of entries in ICO file: {}",
            icon_dir.entries().len()
        );

        let entry = &icon_dir.entries()[args.image_index];
        println!(
            "Image details: {}x{} - {} bits per pixel",
            entry.width(),
            entry.height(),
            entry.bits_per_pixel()
        );
    }

    let mut writer = BufWriter::new(File::create(&args.output)?);

    if let Some(ref conf) = args.config {
        let mut reader = BufReader::new(File::open(conf)?);
        let mut contents = String::new();
        reader.read_to_string(&mut contents)?;
        let config: Value = toml::from_str(contents.as_str())?;

        format = config["ico2img"]["format"]
            .as_str()
            .ok_or_else(|| anyhow!("Output format type isn't specified."))?;

        let buffer = handle_ico(&icon_dir, index)?;

        match format.to_lowercase().as_str() {
            "png" => {
                writer.write_all(&buffer)?;
            }
            "jpg" | "jpeg" => {
                let image = load_from_memory(buffer.as_slice())?.to_rgb8();
                let mut img_buffer = Vec::new();
                let mut cursor = Cursor::new(&mut img_buffer);
                image.write_to(&mut cursor, ImageFormat::Jpeg)?;
                writer.write_all(&img_buffer)?;
            }
            "bmp" => {
                let image = load_from_memory(buffer.as_slice())?;
                let mut img_buffer = Vec::new();
                let mut cursor = Cursor::new(&mut img_buffer);
                image.write_to(&mut cursor, ImageFormat::Bmp)?;
                writer.write_all(&img_buffer)?;
            }
            "webp" => {
                let image = load_from_memory(buffer.as_slice())?;
                let mut img_buffer = Vec::new();
                let mut cursor = Cursor::new(&mut img_buffer);
                image.write_to(&mut cursor, ImageFormat::WebP)?;
                writer.write_all(&img_buffer)?;
            }
            _ => {
                eprintln!("The specified format is not supported at the moment.");
                eprintln!("Feel free to contribute to add new formats");
                panic!();
            }
        }
    } else {
        let buffer = handle_ico(&icon_dir, index)?;
        writer.write_all(&buffer)?;
    }

    Ok(())
}

/// Arguments:
///   - icon_dir: The list of icons in the ICO file.
///   - index: The index of the icon to get.
/// Returns a `Vec<u8>`, containing the bytes to a PNG image, converted from the ICO image at the
/// specified index.
fn handle_ico(icon_dir: &IconDir, index: usize) -> Result<Vec<u8>> {
    if icon_dir.entries().is_empty() {
        return Err(anyhow!("No images found in the ICO file."));
    } else if index >= icon_dir.entries().len() {
        return Err(anyhow!("Invalid image index: {}.", index));
    }

    if index == 0 {
        let image = icon_dir.entries()[0].decode()?;
        let mut png_buffer = Vec::new();
        image.write_png(&mut png_buffer)?;
        Ok(png_buffer)
    } else {
        let entry = &icon_dir.entries()[index];
        let image = entry.decode()?;
        let mut buffer = Vec::new();
        image.write_png(&mut buffer)?;
        Ok(buffer)
    }
}
