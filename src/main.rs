use anyhow::{anyhow, Result};
use clap::Parser;
use ico::IconDir;
use std::{
    fs::File,
    io::{BufReader, BufWriter, Write},
    path::PathBuf,
};

#[derive(Parser)]
struct Args {
    #[arg(help = "The path to the ICO image.")]
    file: PathBuf,

    #[arg(short, help = "The output PNG image.")]
    output: PathBuf,

    #[arg(short, default_value = "0", help = "Index of the image to convert.")]
    image_index: usize,

    #[arg(short, long, help = "Enable verbose output.")]
    verbose: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let path = args.file;
    let reader = BufReader::new(File::open(path)?);
    let icon_dir = IconDir::read(reader)?;

    if args.verbose {
        println!("Number of images in ICO file: {}", icon_dir.entries().len());

        let entry = &icon_dir.entries()[args.image_index];
        println!(
            "Image details: {}x{} - {} bits per pixel",
            entry.width(),
            entry.height(),
            entry.bits_per_pixel()
        );
    }

    if icon_dir.entries().is_empty() {
        return Err(anyhow!("No images found in the ICO file."));
    }

    if args.image_index >= icon_dir.entries().len() {
        return Err(anyhow!("Invalid image index."));
    }

    if args.image_index == 0 {
        let f = File::create(args.output).expect("Failed to open output file");
        let mut writer = BufWriter::new(f);

        let image = icon_dir.entries()[0].decode()?;
        image.write_png(&mut writer)?;
        writer.flush()?;
    } else {
        for (index, entry) in icon_dir.entries().iter().enumerate() {
            let image = entry.decode()?;
            let output_path = args.output.join(format!("image_{}.png", index));
            let writer = BufWriter::new(File::create(output_path)?);
            image.write_png(writer)?;
        }
    }

    Ok(())
}
