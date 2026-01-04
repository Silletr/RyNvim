use pyo3::prelude::*;
pub mod keyboard_scrap;
// mod highlight;
// mod lsp;

#[pyfunction]
fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
    Ok((a + b).to_string())
}

#[pymodule]
fn core_ryvim(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(sum_as_string, m)?)?;
    m.add_function(wrap_pyfunction!(
        keyboard_scrap::parser::write_input_to_file_real_time,
        m
    )?)?;

    Ok(())
}
