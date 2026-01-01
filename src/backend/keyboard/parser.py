import sys
import threading
import queue
import termios
import tty


def write_input_to_file_real_time(filename: str) -> None:
    """Reads input from the user and writes it to a file in real-time."""
    print(f"Writing to {filename}. Press Ctrl+C or type 'exit' to stop.")
    q: queue.Queue[str] = queue.Queue()

    def reader() -> None:
        old_settings = termios.tcgetattr(sys.stdin)
        try:
            tty.setraw(sys.stdin.fileno())
            while True:
                char = sys.stdin.read(1)
                q.put(char)
        finally:
            termios.tcsetattr(sys.stdin, termios.TCSADRAIN, old_settings)

    t = threading.Thread(target=reader, daemon=True)
    t.start()  # Starting thread

    with open(filename, "a", encoding="utf-8") as f:
        current_line = ""
        try:
            while True:
                char = q.get()
                if char == "\x03":  # Ctrl+C
                    print("\nStopped by user.")
                    break
                elif char == "\x7f":  # Backspace
                    if current_line:
                        current_line = current_line[:-1]
                        print("\b \b", end="", flush=True)
                elif char in "\r\n":  # Enter
                    print()
                    f.write(current_line + "\n")
                    f.flush()
                    current_line = ""
                    if current_line.lower() == "exit":
                        break
                else:
                    current_line += char
                    print(char, end="", flush=True)
                    f.write(char)
                    f.flush()
        except KeyboardInterrupt:
            print("\nStopped")


if __name__ == "__main__":
    write_input_to_file_real_time("real_time_log.log")
