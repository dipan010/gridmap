use pyo3::prelude::*;

/// Returns the gridmap-core version string.
#[pyfunction]
fn version() -> &'static str {
    gridmap_core::version()
}

/// The native extension module.
#[pymodule]
fn _core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(version, m)?)?;
    Ok(())
}
