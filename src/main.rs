#[cfg(feature = "genpdf_")]
mod genpdf;
#[cfg(feature = "pdfcreate_")]
mod pdfcreate;
#[cfg(feature = "printpdf_")]
mod printpdf;

fn main() -> anyhow::Result<()> {
	#[cfg(feature = "pdfcreate_")]
	crate::pdfcreate::run()?;
	#[cfg(feature = "printpdf_")]
	crate::printpdf::run()?;
	#[cfg(feature = "genpdf_")]
	crate::genpdf::run()?;
	Ok(())
}
