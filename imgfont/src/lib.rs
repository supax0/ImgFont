use std::fs;
use std::fs::File;
use std::io::{self, BufReader, Read}; // Correctly importing std::io and required structs
use std::path::Path;
use image::{RgbImage, Rgb};
use imageproc::drawing::draw_text_mut;
use rusttype::{Font, Scale};
use rayon::prelude::*;
use rayon::ThreadPoolBuilder;
use num_cpus;

/// Supported font file extensions
const SUPPORTED_FONTS: [&str; 4] = ["ttf", "otf", "woff", "woff2"];

/// Load a font from a file
pub fn load_font(font_path: &str) -> io::Result<Font<'static>> {
    let mut font_data = Vec::new();
    let file = File::open(font_path)?;
    BufReader::new(file).read_to_end(&mut font_data)?;
    Font::try_from_vec(font_data).ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Failed to load font"))
}

/// Generate an image for a word using a specific font and font size
pub fn generate_image_for_word(font: &Font, word: &str, font_size: u32) -> Option<RgbImage> {
    let scale = Scale::uniform(font_size as f32);
    let white = Rgb([255, 255, 255]);
    let black = Rgb([0, 0, 0]);

    let max_width = 2000;
    let width = (font_size * word.len() as u32 * 2).min(max_width);
    let height = font_size * 2;

    if width == 0 || height == 0 {
        eprintln!("Error: Image dimensions are zero for word '{}'", word);
        return None;
    }

    let mut image = RgbImage::new(width, height);
    for pixel in image.pixels_mut() {
        *pixel = white;
    }

    draw_text_mut(&mut image, black, font_size / 2, font_size / 2, scale, font, word);

    if image.pixels().all(|&p| p == Rgb([255, 255, 255])) {
        eprintln!("Skipping image generation for word '{}'. Image is empty.", word);
        return None;
    }

    Some(image)
}

/// Check if an image has content other than just white pixels
pub fn is_not_empty(image: &RgbImage) -> bool {
    image.pixels().any(|&p| p != Rgb([255, 255, 255]))
}

/// Process a single font and generate images for all words
pub fn process_font(
    font_name: &str,
    font_path: &Path,
    words: &[String],
    font_size: u32,
    border_size: u32,
    images_dir: &Path,
) {
    let font_output_dir = images_dir.join(font_name);
    if !font_output_dir.exists() {
        fs::create_dir(&font_output_dir).unwrap();
    }

    match load_font(font_path.to_str().unwrap()) {
        Ok(font) => {
            for word in words {
                let output_path = font_output_dir.join(format!("{}.png", word));
                let temp_image_path = font_output_dir.join(format!("{}_temp.png", word));

                if let Some(image) = generate_image_for_word(&font, word, font_size) {
                    image.save(&temp_image_path).unwrap();

                    if is_not_empty(&image) {
                        std::process::Command::new("magick")
                            .arg(&temp_image_path)
                            .arg("-trim")
                            .arg("+repage")
                            .arg("-border")
                            .arg(border_size.to_string())
                            .arg("-bordercolor")
                            .arg("white")
                            .arg(&output_path)
                            .status()
                            .unwrap();
                        fs::remove_file(&temp_image_path).unwrap();
                    } else {
                        println!("Skipping image generation for word '{}'. Image is empty.", word);
                        fs::remove_file(&temp_image_path).unwrap();
                    }
                } else {
                    eprintln!("Error generating image for word '{}'.", word);
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to load font '{}': {}", font_path.to_str().unwrap(), e);
        }
    }
}

#[allow(non_snake_case)]
pub fn ImgFont(
    fonts_dir: &Path,
    words_file: &Path,
    font_size: u32,
    border_size: u32,
    images_dir: &Path,
) {
    let num_threads = num_cpus::get();
    ThreadPoolBuilder::new().num_threads(num_threads).build_global().unwrap();

    if !images_dir.exists() {
        fs::create_dir(images_dir).unwrap();
    }

    if !fonts_dir.exists() {
        panic!("Font directory does not exist");
    }

    let words: Vec<String> = fs::read_to_string(words_file)
        .expect("Failed to read words file")
        .lines()
        .map(String::from)
        .collect();

    let font_entries: Vec<_> = fs::read_dir(fonts_dir)
        .unwrap()
        .filter_map(|entry| entry.ok())
        .collect();

    font_entries.par_iter().for_each(|entry| {
        let entry_path = entry.path();

        // Process individual font files
        if entry_path.is_file() && SUPPORTED_FONTS.iter().any(|ext| entry_path.extension().unwrap_or_default() == *ext) {
            let font_name = entry_path.file_stem().unwrap().to_str().unwrap().to_string();
            process_font(&font_name, &entry_path, &words, font_size, border_size, images_dir);
        }
        // Process subdirectories containing font files
        else if entry_path.is_dir() {
            let font_name = entry_path.file_name().unwrap().to_str().unwrap().to_string();

            for font_file in fs::read_dir(&entry_path).unwrap() {
                let font_file = font_file.unwrap();
                let font_path = font_file.path();

                if font_path.is_file() && SUPPORTED_FONTS.iter().any(|ext| font_path.extension().unwrap_or_default() == *ext) {
                    process_font(&font_name, &font_path, &words, font_size, border_size, images_dir);
                }
            }
        }
    });

    println!("Finished generating images.");
}

#[cfg(test)]
mod tests {
    use super::*;  // This brings the functions from your library into the test module
    use std::path::PathBuf;

    #[test]
    fn test_imgfont_function() {
        let fonts_dir = PathBuf::from("./fonts");
        let words_file = PathBuf::from("./words.txt");
        let images_dir = PathBuf::from("./output_images");

        let font_size = 200;
        let border_size = 10;

        // Call the ImgFont function from the library
        ImgFont(&fonts_dir, &words_file, font_size, border_size, &images_dir);

        // Simple assertion to check if the output directory was created
        assert!(images_dir.exists());
    }
}
