#![cfg_attr(all(windows, debug_assertions), windows_subsystem = "windows")]

mod printpdf;
mod ui;

#[cfg(feature = "genpdf_")]
mod genpdf;

fn main() -> anyhow::Result<()> {
	flexi_logger::Logger::try_with_env_or_str("img_to_pdf=info")?.start()?;

	#[cfg(feature = "genpdf_")]
	crate::genpdf::run()?;

	crate::ui::run()?;

	Ok(())
}
