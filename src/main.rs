#[cfg(feature = "genpdf_")]
mod genpdf;
#[cfg(feature = "printpdf_")]
mod printpdf;

fn main() -> anyhow::Result<()> {
	#[cfg(feature = "printpdf_")]
	crate::printpdf::run()?;
	#[cfg(feature = "genpdf_")]
	crate::genpdf::run()?;
	Ok(())
}
