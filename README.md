# apyr

`apyr` is a console-based Text User Interface (TUI) program for efficient browsing and filtering of log files.

## Features

- **Console-Based TUI**: Lightweight interface for seamless experience.
- **Regex Filtering**: Use regular expressions to filter log entries.

## User Guide

Navigation is loosely based on Vi keybinds.

### General Navigation

- **Tab**: Toggle between the Search and Matches panels.

### In the Search Panel

- **Character Keys (a-z, 0-9, etc.)**: Type characters to form a search query.
- **Backspace**: Remove the last character from the search query.
- **Escape (Esc)**: Switch focus to the Matches panel.

### In the Matches Panel

- **j**: Scroll down the matches by one line.
- **k**: Scroll up the matches by one line.
- **d**: Scroll down the log by five lines.
- **u**: Scroll up the log by five lines.
- **l**: Scroll horizontally to the right by three columns.
- **h**: Scroll horizontally to the left by three columns.
- **q**: Quit the application.
- **c**: Clear the search query and switch to the Search panel.
- **i**: Switch to the Search panel.
- **f**: Toggle the following mode.
