# code-rain-win

Matrix-style code rain screensaver for Windows 10 / 11. Ports the visual
design of the macOS [code-rain-spike](https://github.com/MakiDevelop) build
(green katakana + digits, white head, 8-bit nearest-pixel feel) to a native
Windows `.scr`.

## Build

### Native (Windows 11, recommended)

1. Install Rust: <https://rustup.rs>
2. Open `cmd` or PowerShell in this folder
3. Run:

   ```cmd
   build.bat
   ```

Output: `dist\coderain.scr` (~300 KB, no dependencies)

### Cross-compile from macOS (optional)

```bash
brew install mingw-w64
rustup target add x86_64-pc-windows-gnu
./build.sh
```

## Install

1. Copy `coderain.scr` to `C:\Windows\System32\`
2. **Settings → Personalization → Lock screen → Screen saver**
3. Pick `coderain` from the dropdown, set wait time, OK

To preview without installing system-wide: right-click the `.scr` → **Test**.

## Exit

Mouse move (>5 px), any key, or any click.

## Tech

- Rust + `windows-sys` (raw Win32 / GDI)
- `AlphaBlend` for the trail fade
- `TextOutW` with `NONANTIALIASED_QUALITY` for crisp pixel chars
- Spans the full virtual desktop (works across multiple monitors)
- `/c` opens a no-op settings dialog; `/p` (preview pane) is a no-op
