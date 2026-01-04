use nix::sys::termios::{
    ControlFlags, InputFlags, LocalFlags, OutputFlags, SetArg, SpecialCharacterIndices, Termios,
    tcgetattr, tcsetattr,
};
use nix::unistd::isatty;
use pyo3::prelude::*;
use std::fs::OpenOptions;
use std::io::{self, Read, Write};
use std::os::fd::BorrowedFd;
use std::os::unix::io::AsRawFd;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[pyfunction]
pub fn write_input_to_file_real_time(filename: &str) -> PyResult<()> {
    println!(
        "Writing to {}. Press Ctrl+C or type 'exit' to stop.",
        filename
    );

    let (tx, rx) = mpsc::channel();
    let running = Arc::new(Mutex::new(true));
    let running_clone = Arc::clone(&running);

    let reader_handle = thread::spawn(move || {
        capture_raw_stdin(tx, running_clone);
    });

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .write(true)
        .open(filename)
        .map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyOSError, _>(format!("Failed to open file: {}", e))
        })?;

    let stdout = io::stdout();
    let mut stdout_handle = stdout.lock();

    process_input_stream(rx, &mut file, &mut stdout_handle, &running)?;

    {
        let mut running_lock = running.lock().unwrap();
        *running_lock = false;
    }
    reader_handle.join().unwrap();

    println!("\nLogging stopped.");
    Ok(())
}

fn capture_raw_stdin(tx: mpsc::Sender<char>, running: Arc<Mutex<bool>>) {
    let stdin = io::stdin();
    let stdin_fd = stdin.as_raw_fd();
    let borrowed_fd = unsafe { BorrowedFd::borrow_raw(stdin_fd) };

    if !isatty(borrowed_fd).unwrap_or(false) {
        return;
    }

    let original_termios = match tcgetattr(borrowed_fd) {
        Ok(t) => t,
        Err(_) => return,
    };

    let mut raw_termios = original_termios.clone();
    configure_raw_mode(&mut raw_termios);

    let _ = tcsetattr(borrowed_fd, SetArg::TCSADRAIN, &raw_termios);

    let mut stdin_handle = stdin.lock();

    let mut big_buf = [0u8; 32];

    loop {
        match stdin_handle.read(&mut big_buf) {
            Ok(0) => break,
            Ok(n) => {
                let slice = &big_buf[..n];
                if let Ok(s) = std::str::from_utf8(slice) {
                    for ch in s.chars() {
                        if tx.send(ch).is_err() {
                            return; // канал закрыт
                        }
                    }
                } else {
                    for &byte in slice {
                        if tx.send(byte as char).is_err() {
                            return;
                        }
                    }
                }
            }
            Err(_) => break,
        }

        if !*running.lock().unwrap() {
            break;
        }
    }
    // Restore termios
    let _ = tcsetattr(borrowed_fd, SetArg::TCSADRAIN, &original_termios);
}

fn configure_raw_mode(termios: &mut Termios) {
    // Disable default handling of in-, output
    termios.input_flags &= !(InputFlags::ICRNL
        | InputFlags::IXON
        | InputFlags::IXOFF
        | InputFlags::IXANY
        | InputFlags::ISTRIP
        | InputFlags::INLCR
        | InputFlags::IGNCR
        | InputFlags::INPCK);
    termios.output_flags &= !(OutputFlags::OPOST | OutputFlags::ONLCR);
    termios.local_flags &=
        !(LocalFlags::ECHO | LocalFlags::ICANON | LocalFlags::ISIG | LocalFlags::IEXTEN);
    termios.control_flags &= !ControlFlags::CS8;

    // VMIN = 1, VTIME = 0 — read 1 byte at a time without timeout
    termios.control_chars[SpecialCharacterIndices::VMIN as usize] = 1;
    termios.control_chars[SpecialCharacterIndices::VTIME as usize] = 0;
}

fn process_input_stream(
    rx: mpsc::Receiver<char>,
    file: &mut std::fs::File,
    stdout: &mut impl Write,
    running: &Arc<Mutex<bool>>,
) -> PyResult<()> {
    let mut current_line = String::new();

    loop {
        match rx.recv_timeout(Duration::from_millis(50)) {
            Ok(ch) => {
                match ch {
                    '\x03' => {
                        // Ctrl+C
                        writeln!(stdout)?;
                        stdout.flush()?;
                        break;
                    }
                    '\x7f' | '\x08' => {
                        // Backspace
                        if !current_line.is_empty() {
                            current_line.pop();
                            write!(stdout, "\x08 \x08")?;
                            stdout.flush()?;
                        }
                    }
                    '\r' | '\n' => {
                        // Enter
                        writeln!(stdout)?;
                        stdout.flush()?;

                        if current_line.to_lowercase() == "exit" {
                            writeln!(file, "{}", current_line)?;
                            break;
                        }

                        writeln!(file, "{}", current_line)?;
                        file.flush()?;
                        current_line.clear();
                    }
                    ch if ch.is_ascii_graphic() || ch == '\t' || ch.is_ascii_control() => {
                        current_line.push(ch);
                        write!(stdout, "{}", ch)?;
                        write!(file, "{}", ch)?;
                        file.flush()?;
                        stdout.flush()?;
                    }
                    _ => {}
                }
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
            Err(mpsc::RecvTimeoutError::Timeout) => {
                if !*running.lock().unwrap() {
                    break;
                }
            }
        }
    }
    Ok(())
}
