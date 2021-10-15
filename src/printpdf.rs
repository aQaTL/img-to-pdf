use printpdf::{Image, Mm, PdfDocument};
use std::fs::File;
use std::io::{Seek, BufWriter};

const A4: (Mm, Mm) = (Mm(210.0), Mm(297.0));

pub fn run() -> anyhow::Result<()> {
	let (doc, page1, layer1) = PdfDocument::new("printpdf_test", A4.0, A4.1, "Layer 1");
	let current_layer = doc.get_page(page1).get_layer(layer1);

	let mut image_file = File::open("assets/benchmark.jpg")?;
	let image = Image::try_from(printpdf::image::codecs::jpeg::JpegDecoder::new(
		&mut image_file,
	)?)?;

	image.add_to_layer(current_layer.clone(), None, None, None, None, None, None);

	// let mut image_file = File::open("assets/benchmark.png")?;
	// let image = Image::try_from(printpdf::image::codecs::png::PngDecoder::new(
	// 	&mut image_file,
	// )?)?;
	image_file.rewind()?;
	let image = Image::try_from(printpdf::image::codecs::jpeg::JpegDecoder::new(
		&mut image_file,
	)?)?;

	let (page2, layer2) = doc.add_page(A4.0, A4.1, "Layer 2");
	let current_layer = doc.get_page(page2).get_layer(layer2);
	image.add_to_layer(current_layer.clone(), None, None, None, None, None, None);

	doc.save(&mut BufWriter::new(File::create("printpdf_test.pdf")?))?;

	Ok(())
}
