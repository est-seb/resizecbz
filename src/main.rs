use std::fs::{self, File};
use std::path::{Path, PathBuf};
use zip::ZipArchive;
use image::{DynamicImage, GenericImageView};
use image::imageops::resize;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get the .cbz file path from command-line arguments
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <path_to_file.cbz>", args[0]);
        std::process::exit(1);
    }
    let cbz_path = &args[1];

    // Extract the .cbz file
    let file = File::open(cbz_path)?;
    let mut archive = ZipArchive::new(file)?;

    // Create a directory to extract files (same as the .cbz file)
    let extract_dir = Path::new(cbz_path).parent().unwrap();
    fs::create_dir_all(extract_dir)?;

    // Extract all files from the archive
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = match file.enclosed_name() {
            Some(path) if !path.is_absolute() => extract_dir.join(path),
            _ => continue,
        };

        if file.name().ends_with('/') {
            fs::create_dir_all(&outpath)?;
        } else {
            if let Some(parent) = outpath.parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent)?;
                }
            }
            let mut outfile = File::create(&outpath)?;
            std::io::copy(&mut file, &mut outfile)?;
        }
    }

    // Create a new directory for resized images
    let cbz_stem = Path::new(cbz_path).file_stem().unwrap();
    let resized_dir = extract_dir.join(format!("{}_resized", cbz_stem.to_string_lossy()));
    fs::create_dir_all(&resized_dir)?;

    // Iterate through extracted files and resize images
    for entry in fs::read_dir(extract_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            // Check if the file is an image
            if let Ok(img) = image::open(&path) {
                // Calculate new dimensions to maintain aspect ratio
                let (width, height) = img.dimensions();
                let target_width = 1024;
                let target_height = 768;

                let width_ratio = target_width as f32 / width as f32;
                let height_ratio = target_height as f32 / height as f32;
                let scale = width_ratio.min(height_ratio);

                let new_width = (width as f32 * scale) as u32;
                let new_height = (height as f32 * scale) as u32;

                // Resize the image
                let resized_img = resize(&img, new_width, new_height, image::imageops::FilterType::Lanczos3);

                // Save the resized image to the new directory
                let filename = path.file_name().unwrap();
                let output_path = resized_dir.join(filename);
                resized_img.save(&output_path)?;
            }
        }
    }

    Ok(())
}