# 📊 TUI Process Monitor
A terminal-based process viewer written in Rust using the tui, crossterm, and sysinfo crates.

This tool provides a real-time, interactive view of system processes and network usage with powerful search and sorting features—all from your terminal.


# 🚀 Features
Live system process monitoring

Search by process name or PID

Sort by CPU, memory, or name

Toggle ascending/descending sort order

View basic network usage stats

Smooth keyboard navigation

Configurable refresh rate and default sort field via config.toml


# 📦 Installation
Prerequisites
Rust (install via rustup)

# Build and Run
git clone https://github.com/yourname/tui-process-monitor

cd tui-process-monitor

cargo build --release

cargo run --release

# ⚙️ Configuration
Create a config.toml in the same directory (optional):

# toml
refresh_rate = 1000      # Refresh interval in milliseconds

default_sort = "cpu"     # Options: "cpu", "mem", "name"

If no config.toml is found, defaults will be used.


# 🕹️ Key Bindings

q	  Quit the application

/	  Start typing a search query

Esc	  Cancel search

c	  Sort by CPU usage

m	  Sort by memory usage

n	  Sort by process name

r  	Toggle sort order (asc/desc)

↑ / ↓	  Navigate the process list

Backspace	  Delete character in search


# 🛠️ Dependencies
crossterm

tui

sysinfo

serde

toml

Add these in Cargo.toml if not already:

# toml
 
[dependencies]

crossterm = "0.27"

tui = "0.19"

sysinfo = "0.30"

serde = { version = "1.0", features = ["derive"] }

toml = "0.8"
