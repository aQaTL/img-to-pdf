use anyhow::bail;
use genpdf::{elements, fonts};
use std::env;

pub fn run() -> anyhow::Result<()> {
	let args: Vec<String> = env::args().collect();
	if args.len() <= 1 {
		bail!("No input files specified.\nUsage:\n\t{} [FILES]", args[0]);
	}
	let images = &args[1..];

	let font = fonts::from_files(
		"/usr/share/fonts/truetype/liberation",
		"LiberationSans",
		Some(fonts::Builtin::Helvetica),
	)?;
	let mut doc = genpdf::Document::new(font);

	doc.set_title("img-to-pdf genpdf test");
	doc.set_minimal_conformance();
	doc.set_line_spacing(1.25);

	doc.set_page_decorator({
		let mut decorator = genpdf::SimplePageDecorator::new();
		decorator.set_margins(0);
		decorator
	});

	for (idx, image_path) in images.iter().enumerate() {
		doc.push(
			elements::Image::from_path(image_path)?.with_position(genpdf::Position::new(0, 0)),
		);
		if idx != images.len() - 1 {
			doc.push(elements::PageBreak::default());
		}
	}

	doc.render_to_file("genpdf_test.pdf")?;

	Ok(())
}
