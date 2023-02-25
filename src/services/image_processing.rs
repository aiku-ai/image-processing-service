use image::imageops::overlay;
use image::io::Reader as ImageReader;
use image::{DynamicImage, ImageBuffer};
use reqwest::get;
use std::fmt::Error;
use std::io::Cursor;
use usvg::Tree;
use usvg_text_layout::{
    fontdb::{self, Database},
    TreeTextToPath,
};

use crate::models::image_overlay_req::{AikuText, ImageOverlayReq};

pub async fn process_image_overlay(overlay_req: ImageOverlayReq) -> Result<Vec<u8>, Error> {
    // base image
    let aiku_img = download_image(overlay_req.image_url).await;
    // overlay image
    let text_img = create_overlay_png(&overlay_req.aiku_text).await;

    // convert each image to a dynamic image
    let mut dyn_final_img = ImageReader::new(Cursor::new(aiku_img))
        .with_guessed_format()
        .unwrap()
        .decode()
        .unwrap();
    let dyn_text_img = ImageReader::new(Cursor::new(text_img))
        .with_guessed_format()
        .unwrap()
        .decode()
        .unwrap();

    // crop to size
    dyn_final_img = dyn_final_img.crop_imm(164, 0, 440, 768);

    // overlay text over aiku
    overlay(&mut dyn_final_img, &dyn_text_img, 10, 608);

    let mut ret_image_bytes: Vec<u8> = Vec::new();
    dyn_final_img
        .write_to(
            &mut Cursor::new(&mut ret_image_bytes),
            image::ImageOutputFormat::Png,
        )
        .unwrap();

    return Ok(ret_image_bytes);
}

async fn download_image(image_url: String) -> Vec<u8> {
    let body = get(image_url).await.unwrap().bytes().await.unwrap();

    return body.to_vec();
}

async fn create_overlay_png(aiku_text: &AikuText) -> Vec<u8> {
    let fontdb = create_font_db();
    let svg = parse_liquid_template(include_str!("../assets/overlay.svg"), &aiku_text);
    let tree = create_svg_tree(svg, fontdb);
    return render_tree_to_png(tree);
}

fn create_font_db() -> Database {
    let mut fontdb = fontdb::Database::new();

    let space_mono_bold = include_bytes!("../assets/fonts/SpaceMono-Bold.ttf").to_vec();
    let space_mono_reg = include_bytes!("../assets/fonts/SpaceMono-Regular.ttf").to_vec();

    fontdb.load_font_data(space_mono_bold);
    fontdb.load_font_data(space_mono_reg);

    return fontdb;
}

fn parse_liquid_template(svg: &str, aiku_text: &AikuText) -> String {
    let template = liquid::ParserBuilder::with_stdlib()
        .build()
        .unwrap()
        .parse(svg)
        .unwrap();

    let globals = liquid::object!({
        "line_one": aiku_text.line_one,
        "line_two": aiku_text.line_two,
        "line_three": aiku_text.line_three,
    });

    return template.render(&globals).unwrap();
}

fn create_svg_tree(svg: String, fontdb: Database) -> Tree {
    let opt = usvg::Options::default();
    let mut tree = usvg::Tree::from_str(&svg, &opt).unwrap();
    tree.convert_text(&fontdb);

    return tree;
}

fn render_tree_to_png(tree: Tree) -> Vec<u8> {
    let pixmap_size = tree.size.to_screen_size();
    let mut pixmap = tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height()).unwrap();
    resvg::render(
        &tree,
        usvg::FitTo::Original,
        tiny_skia::Transform::default(),
        pixmap.as_mut(),
    )
    .unwrap();

    return pixmap.encode_png().unwrap();
}

#[allow(dead_code)]
fn round_corners(image: DynamicImage) {
    // convert image to rbga
    let rgba_image = image.clone().into_rgba8();

    // extract bitmap from this
    //
    // The bitmap represents a pixel with 4 bytes, like this
    // [255 250 240 10]
    //
    // The entire bitmap in this example would be equal to:
    //
    // width * height * 4 (num bytes) OR using the actual values
    // 440 * 770 * 4 = 1355200
    //
    // if we logged bitmap.len() we would see 1355200 outputted
    let mut bitmap = rgba_image.clone().into_raw();

    // find threshold 90% of total size
    let crop_px_threshold = (bitmap.len() as f64) * 0.9;

    // loop through the bitmap and enumerate it
    // we enumerate it so that we know once we've crossed a
    // certain number of pixels and can set the rest to transparent
    for (i, byte) in bitmap.iter_mut().enumerate() {
        if i >= crop_px_threshold as usize {
            // setting the byte to 0 means "transparent" or nothing
            *byte = 0;
        }
    }

    let width = rgba_image.width();
    let height = rgba_image.height();
    let buffer = ImageBuffer::from_raw(width, height, bitmap).unwrap();

    // Create a new DynamicImage from the ImageBuffer
    let dynamic_image = DynamicImage::ImageRgba8(buffer);
}
