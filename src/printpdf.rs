use anyhow::bail;
use printpdf::image::{self, DynamicImage};
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
			if image.color().has_alpha() {
				image = DynamicImage::ImageRgb8(image.into_rgb8());
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
