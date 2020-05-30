# Rim
Simple application for viewing one or multiple images.
It automatically reloads files from disk when they change.

Supports png, jpg, more in progress.

# Usage
`rim <file or directory> [-f] [-s <width> <height>]`

- `-f`: Open as floating window
- `-s`: Set size of window

# Controls
- `Ctrl+P`: Quit
- `Ctrl+O`: Open file
- `Ctrl+W`: Close image
- `Ctrl+M`: Toggle fullscreen
- `Ctrl+N`: Switch to nearest filtering
- `Ctrl+L`: Switch to linear filtering
- `Ctrl+R`: Reload selected image from disk
- `Ctrl+A`: Auto layout (default)
- `Ctrl+H`: Horizontal layout
- `Ctrl+V`: Vertical layout
- `Tab`: Select next image
- `Shift+Tab`: Select previous image
- `I/J/K/L`: Select image above/left/below/right
- `W/A/S/D`: Pan up/left/down/right
- `.`: Zoom in
- `,`: Zoom out
- `Space`: Reset zoom

## When open file dialog is open

- `I/J/K/L`: Navigate folder structure
- `Enter`: Open selected file and close dialog
- `Space`: Open selected file
- `Escape`: Close dialog