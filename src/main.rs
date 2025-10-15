use anyhow::{anyhow, bail, Result};
use clap::{Parser, ValueEnum};
use ico::IconDir;
use image::ImageFormat;
use log::info;
use std::{
    fs::{create_dir_all, File},
    io::{prelude::*, BufReader, BufWriter},
    path::{Path, PathBuf},
    str::FromStr,
};
use toml::Value;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(help = "The path to the ICO image.")]
    file: PathBuf,

    #[arg(short, help = "The output directory for the PNG image(s).")]
    output: Option<PathBuf>,

    #[arg(
        short,
        long = "index",
        help = "Index of the image to convert.",
        conflicts_with_all = &["extract_all", "extract_range", "indices"]
    )]
    image_index: Option<usize>,

    #[arg(
        long,
        help = "Extract all images from the ICO file.",
        conflicts_with_all = &["image_index", "extract_range", "indices"]
    )]
    extract_all: bool,

    #[arg(
        long,
        help = "Extract a range of images (e.g., 0-5).",
        conflicts_with_all = &["image_index", "extract_all", "indices"]
    )]
    extract_range: Option<String>,

    #[arg(
        long,
        help = "Extract specific images by indices (e.g., 0,2,4).",
        conflicts_with_all = &["image_index", "extract_all", "extract_range"],
        use_value_delimiter = true,
        value_delimiter = ','
    )]
    indices: Option<Vec<usize>>,

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
#[derive(ValueEnum, Copy, Clone, Debug)]
enum SupportedImages {
    Png,
    Jpeg,
    Bmp,
    Webp,
}

impl FromStr for SupportedImages {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "png" => Ok(Self::Png),
            "jpg" | "jpeg" => Ok(Self::Jpeg),
            "bmp" => Ok(Self::Bmp),
            "webp" => Ok(Self::Webp),
            _ => bail!("Unsupported image format: {}", s),
        }
    }
}

fn main() -> Result<()> {
    let args = Args::parse();

    if args.verbose {
        simple_logger::init()?;
    }

    let path = &args.file;
    info!("Opening ICO file: {:?}", path);
    let reader = BufReader::new(File::open(path)?);

    info!("Reading ICO directory.");
    let icon_dir = IconDir::read(reader)?;
    let mut format = args.format.clone();

    info!(
        "Number of entries in ICO file: {}",
        icon_dir.entries().len()
    );

    if icon_dir.entries().is_empty() {
        bail!("No images found in the ICO file.");
    }

    if let Some(ref conf) = args.config {
        info!("Loading configuration from: {:?}", conf);
        let mut reader = BufReader::new(
            File::open(conf).map_err(|e| anyhow!("Failed to open configuration file: {:?}", e))?,
        );
        let mut contents = String::new();
        reader
            .read_to_string(&mut contents)
            .map_err(|e| anyhow!("Failed to read configuration file: {:?}", e))?;
        let config: Value = toml::from_str(&contents)
            .map_err(|e| anyhow!("Failed to parse configuration file: {:?}", e))?;

        format = config["ico2img"]["format"]
            .as_str()
            .ok_or_else(|| anyhow!("Output format type isn't specified."))?
            .parse()?;
    }

    let output_dir = args.output.clone().unwrap_or_else(|| PathBuf::from("."));
    create_dir_all(&output_dir)?;

    let indices_to_extract = get_indices_to_extract(&args, icon_dir.entries().len())?;

    for index in indices_to_extract {
        let entry = &icon_dir.entries()[index];
        info!(
            "Image details: {}x{} - {} bits per pixel",
            entry.width(),
            entry.height(),
            entry.bits_per_pixel()
        );

        let output_path = get_output_path(&output_dir, &args.file, index, format);
        info!("Creating output file: {:?}", &output_path);
        let mut writer = BufWriter::new(File::create(&output_path)?);

        info!("Handling ICO file.");
        let buffer = handle_ico(entry)?;

        info!("Writing image to output file.");
        write_image(&mut writer, &buffer, format)?;
    }

    info!("Image conversion completed successfully.");
    Ok(())
}

fn get_indices_to_extract(args: &Args, num_entries: usize) -> Result<Vec<usize>> {
    if args.extract_all {
        return Ok((0..num_entries).collect());
    }

    if let Some(range_str) = &args.extract_range {
        let parts: Vec<&str> = range_str.split('-').collect();
        assert_ne!(parts.len(), 2, "Invalid range format. Use start-end.");
        let start = parts[0].parse::<usize>()?;
        let end = parts[1].parse::<usize>()?;

        assert!(
            start > end,
            "Invalid range: start cannot be greater than end."
        );
        assert!(
            end >= num_entries,
            "Invalid range: end index is out of bounds."
        );

        return Ok((start..=end).collect());
    }

    if let Some(indices) = &args.indices {
        for &index in indices {
            if index >= num_entries {
                bail!("Invalid index: {} is out of bounds.", index);
            }
        }
        return Ok(indices.clone());
    }

    if let Some(index) = args.image_index {
        if index >= num_entries {
            bail!("Invalid image index: {}.", index);
        }
        return Ok(vec![index]);
    }

    // Default to extracting the first image if no other option is provided.
    if num_entries > 0 {
        Ok(vec![0])
    } else {
        bail!("No images to extract.")
    }
}

fn get_output_path(
    output_dir: &Path,
    input_path: &Path,
    index: usize,
    format: SupportedImages,
) -> PathBuf {
    let file_stem = input_path.file_stem().unwrap_or_default().to_string_lossy();
    let extension = match format {
        SupportedImages::Png => "png",
        SupportedImages::Jpeg => "jpg",
        SupportedImages::Bmp => "bmp",
        SupportedImages::Webp => "webp",
    };
    output_dir.join(format!("{}_{}.{}", file_stem, index, extension))
}

fn handle_ico(entry: &ico::IconDirEntry) -> Result<Vec<u8>> {
    info!(
        "Decoding image: {}x{} - {} bits per pixel",
        entry.width(),
        entry.height(),
        entry.bits_per_pixel()
    );
    let image = entry.decode()?;
    let mut buffer = Vec::new();
    image.write_png(&mut buffer)?;
    Ok(buffer)
}

fn write_image<W: Write + Seek>(
    writer: &mut W,
    buffer: &[u8],
    format: SupportedImages,
) -> Result<()> {
    match format {
        SupportedImages::Png => {
            info!("Writing image in PNG format.");
            writer.write_all(buffer)?;
        }
        SupportedImages::Jpeg => {
            info!("Writing image in JPEG format.");
            let image = image::load_from_memory(buffer)?.to_rgb8();
            image.write_to(writer, ImageFormat::Jpeg)?;
        }
        SupportedImages::Bmp => {
            info!("Writing image in BMP format.");
            let image = image::load_from_memory(buffer)?;
            image.write_to(writer, ImageFormat::Bmp)?;
        }
        SupportedImages::Webp => {
            info!("Writing image in WebP format.");
            let image = image::load_from_memory(buffer)?;
            image.write_to(writer, ImageFormat::WebP)?;
        }
    }
    Ok(())
}
