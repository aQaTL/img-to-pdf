use anyhow::bail;
use printpdf::image::{self, DynamicImage, GenericImage, ImageBuffer, Rgb, RgbImage};
use printpdf::{Image, Mm, PdfDocument};
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Seek, Write};
use std::path::Path;

pub fn write_pdf<W: Write, P: AsRef<Path>>(filenames: &[P], output: W) -> anyhow::Result<()> {
	const DPI: f64 = 300.0;

	let doc = PdfDocument::empty("img-to-pdf");

	for filename in filenames {
		let image = load_image(filename)?;

		let width_mm = px_to_mm(image.image.width.0 as f64, DPI);
		let height_mm = px_to_mm(image.image.height.0 as f64, DPI);

		let (page_idx, layer_idx) = doc.add_page(Mm(width_mm), Mm(height_mm), "Layer 1");
		let layer = doc.get_page(page_idx).get_layer(layer_idx);
		image.add_to_layer(layer.clone(), None, None, None, None, None, Some(DPI));
	}

	doc.save(&mut BufWriter::new(output))?;

	Ok(())
}

fn load_image<P: AsRef<Path>>(filename: P) -> anyhow::Result<Image> {
	let mut file = File::open(filename.as_ref())?;
	let mut file_signature = [0u8; 8];
	file.read_exact(&mut file_signature)?;
	match file_signature {
		[b'B', b'M', _, _, _, _, _, _] => {
			file.rewind()?;
			Ok(Image::try_from(image::codecs::bmp::BmpDecoder::new(
				&mut file,
			)?)?)
		}
		[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A] => {
			file.rewind()?;
			let mut image =
				image::io::Reader::with_format(BufReader::new(&mut file), image::ImageFormat::Png)
					.decode()?;

			// Workaround around some bug that causes png's with alpha channel to not render properly
            // https://github.com/fschutt/printpdf/issues/84
			if image.color().has_alpha() {
				let rgba_image = image.into_rgba8();
				let (width, height) = rgba_image.dimensions();
				let mut rgb_image: RgbImage = ImageBuffer::new(width, height);

				let (bg_red, bg_green, bg_blue) = (255.0, 255.0, 255.0);

				for (x, y, pixel) in rgba_image.enumerate_pixels() {
					let alpha_float = pixel.0[3] as f64 / 255.0;
					let (red_float, green_float, blue_float) =
						(pixel.0[0] as f64, pixel.0[1] as f64, pixel.0[2] as f64);
					let red = (1.0 - alpha_float) * bg_red + alpha_float * red_float;
					let blue = (1.0 - alpha_float) * bg_green + alpha_float * green_float;
					let green = (1.0 - alpha_float) * bg_blue + alpha_float * blue_float;

					let pixel = Rgb::from([red as u8, blue as u8, green as u8]);
					unsafe {
						rgb_image.unsafe_put_pixel(x, y, pixel);
					}
				}

				image = DynamicImage::ImageRgb8(rgb_image);
			}

			Ok(Image::from_dynamic_image(&image))
		}
		[0xFF, 0xD8, 0xFF, 0xDB, _, _, _, _]
		| [0xFF, 0xD8, 0xFF, 0xEE, _, _, _, _]
		| [0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46] => {
			file.rewind()?;
			Ok(Image::try_from(image::codecs::jpeg::JpegDecoder::new(
				&mut file,
			)?)?)
		}
		_ => bail!(
			"{} has an unsupported file format",
			filename.as_ref().display()
		),
	}
}

fn px_to_mm(px: f64, dpi: f64) -> f64 {
	px / dots_per_inch_to_dots_per_mm(dpi)
}

fn dots_per_inch_to_dots_per_mm(dpi: f64) -> f64 {
	dpi / 2.54 / 10.0
}
