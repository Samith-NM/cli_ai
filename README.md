# CLI AI

CLI AI is a Rust command-line tool that runs a target program in a sandbox, captures its output and crash details, and then explains the likely cause using the Gemini API.

It is designed for debugging native crashes, hangs, and abnormal exits by combining:

- process execution and timeout handling
- crash signal and exit-code detection
- source context extraction around crash sites
- LLM-based explanation of the failure
- optional bug report generation

## Requirements

- Rust toolchain
- A C/C++/native executable to analyze
- A Gemini API key for live explanations

## Install

```powershell
cd "C:\Users\nmsam\OneDrive\Documents\GitHub\CLI ai\cli_ai"
cargo build
```

## Set your API key

This project reads the key from the environment variable `GEMINI_API_KEY`.

### Temporary shell session

```powershell
$env:GEMINI_API_KEY = "your_api_key_here"
```

### Persistent user environment (Windows)

```powershell
[Environment]::SetEnvironmentVariable("GEMINI_API_KEY", "your_api_key_here", "User")
```

Then restart your terminal.

### Verify it is set

```powershell
$env:GEMINI_API_KEY
```

If the variable is missing, the app falls back to a mock explanation instead of calling the Gemini API.

## Usage

### Show help

```powershell
cargo run -- --help
```

### Run a program

```powershell
g++ sample_crash.cpp -o sample_crash.exe
cargo run -- run "C:\Users\nmsam\OneDrive\Documents\GitHub\CLI ai\cli_ai\sample_crash.exe"
```

### Run with a custom timeout

```powershell
cargo run -- run "C:\path\to\program.exe" --timeout 30
```

### Disable reports

```powershell
cargo run -- run "C:\path\to\program.exe" --no_reports
```

### Use mock mode instead of Gemini

```powershell
cargo run -- run "C:\path\to\program.exe" --mock
```

## Command format

```text
cli_ai.exe run <PROGRAM> [ARGS...] [OPTIONS]
```

Available options:

- `--context <N>`: number of source lines to include around the crash site
- `--mock`: use mock explanation output instead of Gemini
- `--report-dir <DIR>`: directory for generated bug reports
- `--no_reports`: skip writing report files
- `--timeout <SECONDS>`: max runtime before the process is treated as hung

## Example workflow

1. Compile your target program
2. Set `GEMINI_API_KEY`
3. Run:

```powershell
cargo run -- run "C:\path\to\your\program.exe"
```

4. Review the crash explanation and optional report files

## Report output

By default, reports are written to a folder named `cli_ai_bugreports` inside the project directory.

You can override it with:

```powershell
cargo run -- run "C:\path\to\your\program.exe" --report-dir "C:\temp\bugreports"
```

## Notes

- The program under test should be a compiled executable, not a `.cpp` source file.
- For live analysis, the environment variable `GEMINI_API_KEY` must be set before running the tool.
- If no key is present, the app still works in mock mode and prints a local explanation instead of contacting the API.
