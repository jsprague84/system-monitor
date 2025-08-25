# 🖥️ System Monitor TUI

A beautiful, real-time system monitoring tool built in Rust with a Terminal User Interface (TUI).

## ✨ Features

- **📊 Real-time Overview Dashboard**
  - CPU usage with visual gauge
  - Memory usage with actual values (used/total)
  - Swap usage with color-coded status
  - System load averages (1m, 5m, 15m)
  - CPU temperature monitoring
  - System uptime display

- **💾 Process Monitoring** 
  - Top CPU-consuming processes
  - Top memory-consuming processes
  - Live process tables with PID, name, and usage

- **🌐 Network I/O Monitoring**
  - Per-interface network statistics
  - Total RX/TX across all interfaces
  - Active interface detection

- **💽 Storage & Home Directory Analysis**
  - Disk usage for all mounted drives
  - Home directory size calculation
  - Largest subdirectory breakdown

- **🎨 Professional TUI Interface**
  - Color-coded status indicators
  - Smooth real-time updates
  - Tab-based navigation
  - Responsive layout

## 📷 Screenshots

```
🖥️ CPU        💾 Memory 11.2GB/15.6GB    🔄 Swap 425MB/5.0GB
[████████░░]   [██████████████░░░░░]      [██░░░░░░░░]
   15.2%              72.1%                    8.5%
```

## 🚀 Installation

### Prerequisites
- Rust 1.70+ (`rustc --version`)

### Install from source:
```bash
git clone https://github.com/YOUR_USERNAME/system-monitor.git
cd system-monitor
cargo build --release
cargo install --path .
```

### Run:
```bash
# If installed globally:
system-monitor

# Or run directly:
cargo run
```

## ⌨️ Controls

- **Tab** or **→** - Switch to next tab
- **←** - Switch to previous tab  
- **q** - Quit application

## 🏗️ Built With

- [**Rust**](https://rustlang.org/) - Systems programming language
- [**ratatui**](https://github.com/ratatui/ratatui) - Terminal UI framework
- [**sysinfo**](https://github.com/GuillaumeGomez/sysinfo) - System information library
- [**crossterm**](https://github.com/crossterm-rs/crossterm) - Cross-platform terminal manipulation

## 🎯 Compatibility

- **Linux** ✅
- **macOS** ✅  
- **Windows** ✅

## 🤝 Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Built with inspiration from `htop`, `btop`, and `glances`
- Thanks to the Rust community for amazing crates and tools
