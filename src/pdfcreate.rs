use pdf_create::common::{PdfString, Point, Rectangle};
use pdf_create::high::{Handle, Image, Page, Resource, Resources, XObject};
use std::collections::BTreeMap;
use std::fs::File;

pub fn run() -> anyhow::Result<()> {
	println!("hi");
	let mut doc = Handle::new();

	doc.info.author = Some(PdfString::new("aqatl"));
	doc.info.title = Some(PdfString::new("image_test"));

	let mut images = BTreeMap::<String, Resource<XObject>>::new();
	images.insert(
		String::from("img"),
		Resource::Immediate(Box::new(XObject::Image(Image {}))),
	);

	let page = Page {
		media_box: Rectangle {
			ll: Point { x: 0, y: 0 },
			ur: Point { x: 592, y: 842 },
		},
		resources: Resources {
			x_objects: Resource::Immediate(Box::new(images)),
			..Default::default()
		},
		contents: vec![],
	};

	doc.pages.push(page);

	let mut output_file = File::create("pdfcrate_test.pdf")?;
	doc.write(&mut output_file)?;

	Ok(())
}
