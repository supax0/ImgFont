use std::fs;
use std::path::Path;
use std::process::Command;
use image::{RgbImage, Rgb};
use imageproc::drawing::draw_text_mut;
use rusttype::{Font, Scale};
use std::fs::File;
use std::io::{self, BufReader, Read};
use rayon::prelude::*;
use rayon::ThreadPoolBuilder;
use num_cpus;

const FONT_DIR: &str = "./fonts";
const IMAGES_DIR: &str = "./database/metadata";
const FONT_SIZE: u32 = 200;
const BORDER_SIZE: u32 = 10;
const WORDS_FILE: &str = "./words.txt";

fn load_font(font_path: &str) -> io::Result<Font<'static>> {
    let mut font_data = Vec::new();
    let file = File::open(font_path)?;
    BufReader::new(file).read_to_end(&mut font_data)?;
    Font::try_from_vec(font_data).ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Failed to load font"))
}

fn generate_image_for_word(font: &Font, word: &str, font_size: u32) -> Option<RgbImage> {
    let scale = Scale::uniform(font_size as f32);
    let white = Rgb([255, 255, 255]);
    let black = Rgb([0, 0, 0]);
    
    // Adjust width based on character count, limiting max size to avoid overflow
    let max_width = 2000;
    let width = (font_size * word.len() as u32 * 2).min(max_width); // Estimate width based on character count
    let height = font_size * 2;

    // Ensure the size is reasonable
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
        // If the image is completely white, consider it empty
        eprintln!("Skipping image generation for word '{}'. Image is empty.", word);
        return None;
    }

    Some(image)
}

fn is_not_empty(image: &RgbImage) -> bool {
    image.pixels().any(|&p| p != Rgb([255, 255, 255]))
}

fn process_font(font_name: String, font_path: &Path, words: &[String]) {
    let font_output_dir = format!("{}/{}", IMAGES_DIR, font_name);
    if !Path::new(&font_output_dir).exists() {
        fs::create_dir(&font_output_dir).unwrap();
    }

    match load_font(font_path.to_str().unwrap()) {
        Ok(font) => {
            for word in words {
                let output_path = format!("{}/{}.png", font_output_dir, word);
                let temp_image_path = format!("{}/{}_temp.png", font_output_dir, word);

                if let Some(image) = generate_image_for_word(&font, word, FONT_SIZE) {
                    image.save(&temp_image_path).unwrap();

                    if is_not_empty(&image) {
                        Command::new("magick")
                            .arg(&temp_image_path)
                            .arg("-trim")
                            .arg("+repage")
                            .arg("-border")
                            .arg(BORDER_SIZE.to_string())
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
        },
        Err(e) => {
            eprintln!("Failed to load font '{}': {}", font_path.to_str().unwrap(), e);
        }
    }
}

fn main() {
    // Configure Rayon to use all available logical CPU cores
    let num_threads = num_cpus::get(); // Detect number of logical CPU cores
    ThreadPoolBuilder::new().num_threads(num_threads).build_global().unwrap();

    if !Path::new(IMAGES_DIR).exists() {
        fs::create_dir(IMAGES_DIR).unwrap();
    }

    if !Path::new(FONT_DIR).exists() {
        panic!("Font directory does not exist");
    }

    let words: Vec<String> = fs::read_to_string(WORDS_FILE)
        .expect("Failed to read words file")
        .lines()
        .map(String::from)
        .collect();

    let font_dirs: Vec<_> = fs::read_dir(FONT_DIR)
        .unwrap()
        .filter_map(|entry| entry.ok())
        .collect();

    // Parallel processing of fonts using rayon
    font_dirs.par_iter().for_each(|font_dir_entry| {
        let subdir_path = font_dir_entry.path();

        if subdir_path.is_dir() {
            let font_name = subdir_path.file_name().unwrap().to_str().unwrap().to_string();

            for font_file in fs::read_dir(&subdir_path).unwrap() {
                let font_file = font_file.unwrap();
                let font_path = font_file.path();

                if font_path.is_file() && (font_path.extension().unwrap() == "ttf" || font_path.extension().unwrap() == "otf") {
                    process_font(font_name.clone(), &font_path, &words);
                }
            }
        }
    });

    println!("Finished generating images.");
}
