#  ImgFont


It is a model that generate images from font, it can be used to generate any text into images.




use std::path::PathBuf;

use imgfont::ImgFont;

fn test_imgfont() {

    // Define the directories and file paths
    let fonts_dir = PathBuf::from("./tests/fonts");      // You can create a sample fonts folder within the tests directory
    let words_file = PathBuf::from("./tests/words.txt"); // A sample words file
    let images_dir = PathBuf::from("./tests/output_images");
    let font_size = 200;
    let border_size = 10;
    // Call the ImgFont function from the library
    ImgFont(&fonts_dir, &words_file, font_size, border_size, &images_dir);
    // Add assertions here as needed.
    // For now, we're not returning anything, but you can check file creation if necessary.
    assert!(images_dir.exists()); // Ensure the output directory is created
}
