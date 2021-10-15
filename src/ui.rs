use iced::{
	button, executor, image, scrollable, window, Application, Button, Clipboard, Column, Command,
	Container, Element, Length, Row, Rule, Scrollable, Settings, Subscription, Text,
};
use iced_native::Event;
use log::info;
use std::fs::File;
use std::path::PathBuf;
use std::sync::Arc;

const APP_NAME: &str = "img-to-pdf";

pub fn run() -> anyhow::Result<()> {
	Ok(Ui::run(Settings {
		window: window::Settings {
			size: (1000, 800),
			min_size: None,
			max_size: None,
			resizable: true,
			decorations: true,
			transparent: false,
			always_on_top: false,
			icon: None,
		},
		flags: (),
		default_font: None,
		default_text_size: 20,
		antialiasing: false,
		exit_on_close_request: true,
	})?)
}

struct Ui {
	dropped_files: Vec<PathBuf>,
	output_path: PathBuf,
	pdf_generation_state: PdfGenerationState,

	ui_states: UiStates,
}

impl Ui {
	fn new() -> Self {
		Ui {
			dropped_files: std::env::args().skip(1).map(PathBuf::from).collect(),
			output_path: PathBuf::from("img-to-pdf_output.pdf"),
			pdf_generation_state: PdfGenerationState::Nothing,
			ui_states: Default::default(),
		}
	}
}

enum PdfGenerationState {
	Nothing,
	Working,
	Done,
	Failed(String),
}

#[derive(Default)]
struct UiStates {
	generate_pdf_button: button::State,
	choose_output_path_button: button::State,
	images_preview_scrollable: scrollable::State,
}

#[derive(Debug, Clone)]
enum Message {
	FileDropped(PathBuf),
	GeneratePdf,
	GeneratePdfResult(Arc<anyhow::Result<()>>),
	ChooseOutputPath,
	ChooseOutputPathResult(Option<PathBuf>),
}

impl Application for Ui {
	type Executor = executor::Default;
	type Message = Message;
	type Flags = ();

	fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
		(Ui::new(), Command::none())
	}

	fn title(&self) -> String {
		String::from(APP_NAME)
	}

	fn update(
		&mut self,
		message: Self::Message,
		_clipboard: &mut Clipboard,
	) -> Command<Self::Message> {
		match message {
			Message::FileDropped(file_path) => {
				info!("File dropped: {}", file_path.display());
				self.dropped_files.push(file_path);
			}
			Message::GeneratePdf => {
				self.pdf_generation_state = PdfGenerationState::Working;
				return Command::perform(
					generate_pdf(self.dropped_files.clone(), self.output_path.clone()),
					|res| Message::GeneratePdfResult(Arc::new(res)),
				);
			}
			Message::GeneratePdfResult(result) => {
				info!("Generate pdf result: {:?}", result);
				self.pdf_generation_state = match &*result {
					Ok(_) => PdfGenerationState::Done,
					Err(e) => PdfGenerationState::Failed(e.to_string()),
				};
			}
			Message::ChooseOutputPath => {
				return Command::perform(choose_output_path(), |output_path| {
					Message::ChooseOutputPathResult(output_path)
				});
			}
			Message::ChooseOutputPathResult(Some(new_output_path)) => {
				self.output_path = new_output_path;
			}
			Message::ChooseOutputPathResult(None) => {
				info!("File dialog closed")
			}
		}

		Command::none()
	}

	fn subscription(&self) -> Subscription<Self::Message> {
		iced_native::subscription::events_with(|ev, status| match (ev, status) {
			(Event::Window(iced_native::window::Event::FileDropped(file_path)), _) => {
				Some(Message::FileDropped(file_path))
			}
			(Event::Window(iced_native::window::Event::FileHovered(file_path)), _) => {
				info!("File hovered: {}", file_path.display());
				None
			}
			// (Event::Window(iced_native::window::Event::FileHovered(file_path)), _) => Some(Message::FileDropped(file_path)),
			_ => None,
		})
	}

	fn view(&mut self) -> Element<Self::Message> {
		let generate_pdf_button = Button::new(
			&mut self.ui_states.generate_pdf_button,
			Text::new("Generate pdf"),
		)
		.on_press(Message::GeneratePdf);

		let output_path = Text::new(format!("Output path: {}", self.output_path.display()));
		let choose_output_path_button =
			Button::new(&mut self.ui_states.choose_output_path_button, output_path)
				.on_press(Message::ChooseOutputPath);

		let buttons_row = Container::new(
			Row::new()
				.height(Length::Shrink)
				.width(Length::Shrink)
				.padding(5)
				.spacing(5)
				.push(choose_output_path_button)
				.push(generate_pdf_button),
		)
		.width(Length::Fill)
		.center_x();

		let mut images_preview = Scrollable::new(&mut self.ui_states.images_preview_scrollable);

		for row_idx in (0..self.dropped_files.len()).step_by(4) {
			let mut row = Row::new();

			for column_idx in row_idx..(row_idx + 4).min(self.dropped_files.len()) {
				row = row.push(
					Container::new(image::Image::new(&self.dropped_files[column_idx]))
						.padding(5)
						.width(Length::FillPortion(4)),
				);
			}
			images_preview = images_preview.push(row);
		}

		let mut column = Column::new()
			.width(Length::Fill)
			.height(Length::Fill)
			.push(buttons_row);

		match &self.pdf_generation_state {
			PdfGenerationState::Working => {
				column = column.push(
					Container::new(Text::new("Generating pdf..."))
						.width(Length::Fill)
						.center_x(),
				)
			}
			PdfGenerationState::Done => {
				column = column.push(
					Container::new(Text::new("Done"))
						.width(Length::Fill)
						.center_x(),
				)
			}
			PdfGenerationState::Failed(err) => {
				column = column.push(
					Container::new(Text::new(format!("Failed: {}", err)))
						.width(Length::Fill)
						.center_x(),
				)
			}
			_ => (),
		}

		column = column.push(Rule::horizontal(10)).push(images_preview);

		Container::new(column)
			.width(Length::Fill)
			.height(Length::Fill)
			.center_x()
			.center_y()
			.into()
	}
}

async fn generate_pdf(file_paths: Vec<PathBuf>, output_path: PathBuf) -> anyhow::Result<()> {
	tokio::task::spawn_blocking(move || {
		info!("Generating pdf from {:#?}", file_paths);
		info!("Output path: {}", output_path.display());
		let mut output_file = File::create(output_path)?;
		crate::printpdf::write_pdf(&file_paths, &mut output_file)?;
		Ok(())
	})
	.await?
}

async fn choose_output_path() -> Option<PathBuf> {
	rfd::AsyncFileDialog::new()
		.add_filter("pdf", &["pdf"])
		.save_file()
		.await
		.map(|path| {
			let path = path.path();
			match path.extension() {
				Some(_) => path.to_path_buf(),
				None => path.with_extension("pdf"),
			}
		})
}
