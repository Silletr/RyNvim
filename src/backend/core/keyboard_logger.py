import sys
from prompt_toolkit import PromptSession


def log_input_to_file(filename: str) -> None:
    session = PromptSession("> ")
    with open(filename, "w", encoding="utf-8") as f:
        while True:
            try:
                line = session.prompt()
                f.write(line + "\n")
                f.flush()
                if line.lower().strip() == "exit":
                    break
            except KeyboardInterrupt:
                print("\nLogging stopped.", file=sys.stderr)
                break
            except EOFError:
                break


if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python3 keyboard_logger.py <filename>", file=sys.stderr)
        sys.exit(1)
    log_input_to_file(sys.argv[1])
