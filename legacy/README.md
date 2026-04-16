# Legacy Python implementation

This directory contains the original Python 3.12 + portable-Python
implementation of SilentWitness. It is preserved for reference and for users
who have not yet migrated to the Rust build.

New work should go into `../rust/`. Bug fixes against the Python version are
not expected.

See the top-level [README.md](../README.md) and [rust/README.md](../rust/README.md)
for the modernised replacement.

## Files

- `Scripts/` — Python recorder, keylogger, status GUI, INI editor.
- `Startup/` — Start Menu shortcut stubs.
- `setup_silentwitness.ps1` / `.bat` — Downloads portable Python + FFmpeg
  and wires up `C:\Tools\SilentWitness`.
- `install_silentwitness.ps1` — One-liner `irm | iex` bootstrap.
- `requirements.txt` — pip dependencies.
