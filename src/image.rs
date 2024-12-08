use std::io::Cursor;
use std::sync::LazyLock;
use std::time::Duration;

use fumen::enums::BlockType;
use fumen::fumen::Fumen;
use image::{Delay, DynamicImage, GenericImage, GenericImageView, ImageBuffer, ImageFormat, ImageReader, Pixel, Rgb, RgbImage};
use image::codecs::gif::{GifEncoder, Repeat};
use image::imageops::resize;

static BOARD_IMG: LazyLock<DynamicImage> = LazyLock::new(|| {
	let board_img = ImageReader::open("res/board.png").unwrap().decode().unwrap();
	board_img
});

static MINOS_IMG: LazyLock<DynamicImage> = LazyLock::new(|| {
	let minos_img = ImageReader::open("res/blocks.png").unwrap().decode().unwrap();
	minos_img
});

const BOARD_OFFSET_X: usize = 124;
const BOARD_OFFSET_Y: usize = 75;
const MINO_SIZE: usize = 48;
const ACTUAL_MINO_SIZE: usize = 23;

static MINO_IMGS: LazyLock<Vec<ImageBuffer<Rgb<u8>, Vec<u8>>>> = LazyLock::new(|| {
	let mut mino_imgs = Vec::new();
	for mino_pos in 0..8 {
		let mut mino_img = RgbImage::new(MINO_SIZE as u32, MINO_SIZE as u32);
		for x in 0..MINO_SIZE {
			for y in 0..MINO_SIZE {
				let pixel = MINOS_IMG.get_pixel((mino_pos * MINO_SIZE + x) as u32, y as u32);
				mino_img.put_pixel(x as u32, y as u32, pixel.to_rgb());
			}
		}

		let resized_mino = resize(&mino_img, ACTUAL_MINO_SIZE as u32, ACTUAL_MINO_SIZE as u32, image::imageops::FilterType::Gaussian);
		mino_imgs.push(resized_mino);
	}

	mino_imgs
});


pub fn create_dynamic_image(fumen: &Fumen, fumen_index: usize) -> DynamicImage {
	let page = &fumen.pages[fumen_index];
	let mut result_img = BOARD_IMG.clone();

	for x in 0..10 {
		for y in 0..20 {
			let mino = match page.field.0[x + (y + 3) * 10] {
				BlockType::Empty => continue,
				BlockType::Z => &MINO_IMGS[0],
				BlockType::L => &MINO_IMGS[1],
				BlockType::O => &MINO_IMGS[2],
				BlockType::S => &MINO_IMGS[3],
				BlockType::I => &MINO_IMGS[4],
				BlockType::J => &MINO_IMGS[5],
				BlockType::T => &MINO_IMGS[6],
				BlockType::Gray => &MINO_IMGS[7],
			};

			for mino_x in 0..ACTUAL_MINO_SIZE {
				for mino_y in 0..ACTUAL_MINO_SIZE {
					let pixel = mino.get_pixel(mino_x as u32, mino_y as u32);
					result_img.put_pixel(BOARD_OFFSET_X as u32 + (x * ACTUAL_MINO_SIZE + mino_x) as u32, BOARD_OFFSET_Y as u32 + (y * ACTUAL_MINO_SIZE + mino_y) as u32, pixel.to_rgba());
				}
			}
		}
	}
	result_img
}

pub fn create_webp(fumen: &Fumen, fumen_index: usize) -> Vec<u8> {
	let image = create_dynamic_image(fumen, fumen_index);
	let mut bytes: Vec<u8> = Vec::new();
	image.write_to(&mut Cursor::new(&mut bytes), ImageFormat::WebP).unwrap();
	bytes
}


pub fn create_gif(fumen: &Fumen) -> Vec<u8> {
	let mut buffer = Cursor::new(Vec::new());
	//let mut ref_buf=buffer.get_ref();

	{
		let mut encoder = GifEncoder::new_with_speed(&mut buffer, 30);
		encoder.set_repeat(Repeat::Infinite).unwrap();

		let mut image_buffer = Vec::new();
		for page_index in 0..fumen.pages.len()
		{
			let  img = create_dynamic_image(fumen, page_index);
			let duration = Duration::from_millis(500);
			let delay = Delay::from_saturating_duration(duration);
			let frame = image::Frame::from_parts(img.to_rgba8(), 0, 0, delay);
			image_buffer.push(frame);
		}

		let result = encoder.encode_frames(image_buffer);
		match result {
			Ok(_) => {}
			Err(e) => {
				dbg!(e);
				panic!()
			}
		}
	}

	buffer.into_inner()
}