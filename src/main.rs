#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod printpdf;
mod ui;

fn main() -> anyhow::Result<()> {
	flexi_logger::Logger::try_with_env_or_str("img_to_pdf=info")?.start()?;

	crate::ui::run()?;

	Ok(())
}
