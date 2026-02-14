use pyo3::prelude::*;
use std::path::PathBuf;
use std::process::{Command, Stdio};

#[pyfunction]
pub fn write_input_to_file_real_time(filename: &str) -> PyResult<()> {
    println!(
        "Writing to {}. Type 'exit' and press Enter or Ctrl+C to stop.",
        filename
    );

    let mut script_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    script_path.push("keyboard_logger.py");

    // Fix: Bind Command first, THEN chain methods
    let mut cmd = Command::new("python3");
    cmd.arg(&script_path);
    cmd.arg(filename);
    cmd.stdin(Stdio::inherit());   // Allow keys
    cmd.stdout(Stdio::piped());    // Capture output
    cmd.stderr(Stdio::piped());    // See errors

    let child = cmd.spawn()
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyOSError, _>(format!("Spawn failed: {}", e)))?;

    let outputs = child.wait_with_output()
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyOSError, _>(format!("Wait failed: {}", e)))?;

    // Print Python's error for debugging
    if !outputs.stderr.is_empty() {
        eprintln!("Python error: {}", String::from_utf8_lossy(&outputs.stderr));
    }

    println!("\nLogging stopped.");

    let status = outputs.status;
    if status.success() {
        Ok(())
    } else {
        Err(PyErr::new::<pyo3::exceptions::PyIOError, _>(
            format!("Python script exited with error: {:?}", status.code())
        ))
    }
}
