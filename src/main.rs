use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{
        Block, Borders, Cell, Gauge, List, ListItem, Paragraph, Row, Table, Tabs,
    },
    Frame, Terminal,
};
use sysinfo::{System, ProcessesToUpdate, Disks, Components, Networks};
use std::{
    error::Error,
    io,
    time::{Duration, Instant},
    fs,
    path::Path,
};
use chrono::Local;
use dirs;

struct App {
    system: System,
    disks: Disks,
    components: Components,
    networks: Networks,
    last_update: Instant,
    tab_index: usize,
}

impl App {
    fn new() -> App {
        App {
            system: System::new_all(),
            disks: Disks::new_with_refreshed_list(),
            components: Components::new_with_refreshed_list(),
            networks: Networks::new_with_refreshed_list(),
            last_update: Instant::now(),
            tab_index: 0,
        }
    }

    fn refresh(&mut self) {
        if self.last_update.elapsed() >= Duration::from_secs(1) {
            self.system.refresh_cpu_all();
            self.system.refresh_processes(ProcessesToUpdate::All, true);
            self.system.refresh_memory();
            self.disks.refresh(true);
            self.components.refresh(true);
            self.networks.refresh(true);
            self.last_update = Instant::now();
        }
    }

    fn next_tab(&mut self) {
        self.tab_index = (self.tab_index + 1) % 2;
    }

    fn previous_tab(&mut self) {
        if self.tab_index > 0 {
            self.tab_index -= 1;
        } else {
            self.tab_index = 1;
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run it
    let app = App::new();
    let res = run_app(&mut terminal, app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        app.refresh();
        terminal.draw(|f| ui(f, &app))?;

        if crossterm::event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Right | KeyCode::Tab => app.next_tab(),
                    KeyCode::Left => app.previous_tab(),
                    _ => {}
                }
            }
        }
    }
}

fn ui(f: &mut Frame, app: &App) {
    let size = f.area();
    
    // Create main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Title bar
            Constraint::Length(3),  // Tabs
            Constraint::Min(0),     // Content
            Constraint::Length(1),  // Status bar
        ])
        .split(size);

    // Title bar
    let title = Paragraph::new("🖥️  System Monitor TUI")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    // Tabs
    let tab_titles = vec!["📊 Overview", "💾 Processes"];
    let tabs = Tabs::new(tab_titles)
        .block(Block::default().borders(Borders::ALL))
        .select(app.tab_index)
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));
    f.render_widget(tabs, chunks[1]);

    // Content based on selected tab
    match app.tab_index {
        0 => draw_overview_tab(f, chunks[2], app),
        1 => draw_processes_tab(f, chunks[2], app),
        _ => {}
    }

    // Status bar
    let status = format!("Last updated: {} | Press 'q' to quit | ←/→ or Tab to switch tabs", 
                        Local::now().format("%H:%M:%S"));
    let status_bar = Paragraph::new(status)
        .style(Style::default().fg(Color::Gray));
    f.render_widget(status_bar, chunks[3]);
}

fn draw_overview_tab(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // CPU, Memory, Swap gauges on same line
            Constraint::Length(8), // System info
            Constraint::Min(0),    // Network and storage info
        ])
        .split(area);

    // CPU, Memory, and Swap gauges - all on same line
    let gauge_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(33), Constraint::Percentage(33), Constraint::Percentage(34)])
        .split(chunks[0]);

    // CPU gauge
    let cpu_usage = app.system.global_cpu_usage();
    let cpu_gauge = Gauge::default()
        .block(Block::default().title("🖥️ CPU").borders(Borders::ALL))
        .gauge_style(Style::default().fg(Color::Green))
        .percent(cpu_usage as u16)
        .label(format!("{:.1}%", cpu_usage));
    f.render_widget(cpu_gauge, gauge_chunks[0]);

    // Memory gauge
    let total_memory = app.system.total_memory();
    let used_memory = app.system.used_memory();
    let memory_usage = (used_memory as f64 / total_memory as f64) * 100.0;
    let memory_gauge = Gauge::default()
        .block(Block::default().title(format!("💾 Memory {}/{}", format_bytes(used_memory), format_bytes(total_memory))).borders(Borders::ALL))
        .gauge_style(Style::default().fg(Color::Blue))
        .percent(memory_usage as u16)
        .label(format!("{:.1}%", memory_usage));
    f.render_widget(memory_gauge, gauge_chunks[1]);

    // Swap gauge
    let total_swap = app.system.total_swap();
    let used_swap = app.system.used_swap();
    if total_swap > 0 {
        let swap_usage = (used_swap as f64 / total_swap as f64) * 100.0;
        let swap_color = if swap_usage > 50.0 { Color::Red } else if swap_usage > 10.0 { Color::Yellow } else { Color::Green };
        let swap_gauge = Gauge::default()
            .block(Block::default().title(format!("🔄 Swap {}/{}", format_bytes(used_swap), format_bytes(total_swap))).borders(Borders::ALL))
            .gauge_style(Style::default().fg(swap_color))
            .percent(swap_usage as u16)
            .label(format!("{:.1}%", swap_usage));
        f.render_widget(swap_gauge, gauge_chunks[2]);
    } else {
        let swap_info = Paragraph::new("Not configured")
            .block(Block::default().title("🔄 Swap").borders(Borders::ALL))
            .style(Style::default().fg(Color::Gray));
        f.render_widget(swap_info, gauge_chunks[2]);
    }

    // System info panel - compact
    let mut system_info = Vec::new();
    
    // Load average
    let load_avg = System::load_average();
    system_info.push(ListItem::new(format!("📊 Load Average: {:.2} {:.2} {:.2} (1m 5m 15m)", 
                                          load_avg.one, load_avg.five, load_avg.fifteen)));
    
    // CPU temperature
    let temp_info = get_cpu_temperature(&app.components);
    system_info.push(ListItem::new(temp_info));
    
    // Uptime
    system_info.push(ListItem::new(format!("⏰ Uptime: {}", format_uptime(System::uptime()))));

    let system_list = List::new(system_info)
        .block(Block::default().title("📈 System Information").borders(Borders::ALL))
        .style(Style::default().fg(Color::White));
    f.render_widget(system_list, chunks[1]);

    // Bottom section - Network and Storage with Home directory
    let bottom_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(chunks[2]);

    // Network summary
    let mut network_info = Vec::new();
    let (total_rx, total_tx, interface_count) = get_network_summary(&app.networks);
    network_info.push(ListItem::new(format!("📡 Active Interfaces: {}", interface_count)));
    network_info.push(ListItem::new(format!("📥 Total Received: {}", format_bytes(total_rx))));
    network_info.push(ListItem::new(format!("📤 Total Transmitted: {}", format_bytes(total_tx))));
    
    // Add per-interface breakdown for active ones
    let mut interface_count = 0;
    for (interface_name, network) in &app.networks {
        let received = network.received();
        let transmitted = network.transmitted();
        
        if (received > 0 || transmitted > 0) && interface_count < 4 { // Show top 4 interfaces
            network_info.push(ListItem::new(format!("  {} | RX: {} TX: {}", 
                                                   truncate_name(interface_name, 10),
                                                   format_bytes(received), 
                                                   format_bytes(transmitted))));
            interface_count += 1;
        }
    }

    let network_list = List::new(network_info)
        .block(Block::default().title("🌐 Network I/O").borders(Borders::ALL))
        .style(Style::default().fg(Color::White));
    f.render_widget(network_list, bottom_chunks[0]);

    // Combined Storage and Home directory info
    let mut storage_info = Vec::new();
    
    // Storage summary
    let mut total_storage = 0u64;
    let mut total_used = 0u64;
    let mut disk_count = 0;
    
    for disk in &app.disks {
        let total_space = disk.total_space();
        let available_space = disk.available_space();
        let used_space = total_space - available_space;
        
        if total_space > 0 {
            total_storage += total_space;
            total_used += used_space;
            disk_count += 1;
            
            let usage_percent = (used_space as f64 / total_space as f64) * 100.0;
            if disk_count <= 4 { // Show details for first 4 disks
                storage_info.push(ListItem::new(format!("💽 {} | {:.1}% | {}/{}", 
                                                       truncate_name(&disk.name().to_string_lossy(), 20),
                                                       usage_percent,
                                                       format_bytes(used_space),
                                                       format_bytes(total_space))));
            }
        }
    }
    
    // Add storage summary at the top
    if total_storage > 0 {
        let total_usage = (total_used as f64 / total_storage as f64) * 100.0;
        storage_info.insert(0, ListItem::new(format!("📊 {} Disks Total | {:.1}% | {}/{}", 
                                                    disk_count, total_usage, 
                                                    format_bytes(total_used), 
                                                    format_bytes(total_storage))));
        storage_info.insert(1, ListItem::new("".to_string())); // Separator
    }
    
    // Add home directory information
    if let Some(home_dir) = dirs::home_dir() {
        storage_info.push(ListItem::new("🏠 Home Directory:".to_string()));
        storage_info.push(ListItem::new(format!("   📂 Path: {}", truncate_name(&home_dir.display().to_string(), 35))));
        
        match calculate_directory_size(&home_dir) {
            Ok(size) => {
                storage_info.push(ListItem::new(format!("   📊 Size: {}", format_bytes(size))));
                
                // Show largest subdirectories
                let common_dirs = ["Downloads", "Documents", "Pictures", "Videos", "Desktop", "Music"];
                let mut dir_sizes = Vec::new();
                
                for dir_name in &common_dirs {
                    let dir_path = home_dir.join(dir_name);
                    if dir_path.exists() && dir_path.is_dir() {
                        if let Ok(dir_size) = calculate_directory_size(&dir_path) {
                            if dir_size > 0 {
                                dir_sizes.push((dir_name, dir_size));
                            }
                        }
                    }
                }
                
                // Sort and show top 2 directories
                dir_sizes.sort_by(|a, b| b.1.cmp(&a.1));
                for (dir_name, size) in dir_sizes.iter().take(2) {
                    storage_info.push(ListItem::new(format!("   📁 {}: {}", dir_name, format_bytes(*size))));
                }
            }
            Err(_) => {
                storage_info.push(ListItem::new("   ❌ Could not calculate size".to_string()));
            }
        }
    }

    let storage_list = List::new(storage_info)
        .block(Block::default().title("💽 Storage & Home Directory").borders(Borders::ALL))
        .style(Style::default().fg(Color::White));
    f.render_widget(storage_list, bottom_chunks[1]);
}

fn draw_processes_tab(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Top CPU processes
    let mut cpu_processes: Vec<_> = app.system.processes()
        .iter()
        .map(|(pid, process)| (*pid, process))
        .collect();
    cpu_processes.sort_by(|a, b| {
        b.1.cpu_usage().partial_cmp(&a.1.cpu_usage()).unwrap_or(std::cmp::Ordering::Equal)
    });

    let cpu_header = Row::new(vec!["PID", "Name", "CPU %"])
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));
    let cpu_rows: Vec<Row> = cpu_processes
        .iter()
        .take(15)
        .map(|(pid, process)| {
            Row::new(vec![
                Cell::from(format!("{}", pid)),
                Cell::from(truncate_name(process.name().to_string_lossy().as_ref(), 25)),
                Cell::from(format!("{:.1}%", process.cpu_usage())),
            ])
        })
        .collect();

    let cpu_table = Table::new(
        cpu_rows,
        &[Constraint::Length(8), Constraint::Min(20), Constraint::Length(8)]
    )
        .header(cpu_header)
        .block(Block::default().title("⚡ Top CPU Processes").borders(Borders::ALL))
        .column_spacing(1);
    f.render_widget(cpu_table, chunks[0]);

    // Top Memory processes
    let mut mem_processes: Vec<_> = app.system.processes()
        .iter()
        .map(|(pid, process)| (*pid, process))
        .collect();
    mem_processes.sort_by(|a, b| b.1.memory().cmp(&a.1.memory()));

    let mem_header = Row::new(vec!["PID", "Name", "Memory"])
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));
    let mem_rows: Vec<Row> = mem_processes
        .iter()
        .take(15)
        .map(|(pid, process)| {
            Row::new(vec![
                Cell::from(format!("{}", pid)),
                Cell::from(truncate_name(process.name().to_string_lossy().as_ref(), 25)),
                Cell::from(format_bytes(process.memory())),
            ])
        })
        .collect();

    let mem_table = Table::new(
        mem_rows,
        &[Constraint::Length(8), Constraint::Min(20), Constraint::Length(12)]
    )
        .header(mem_header)
        .block(Block::default().title("💾 Top Memory Processes").borders(Borders::ALL))
        .column_spacing(1);
    f.render_widget(mem_table, chunks[1]);
}

// Helper functions
fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;
    
    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }
    
    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

fn format_uptime(uptime_seconds: u64) -> String {
    let days = uptime_seconds / 86400;
    let hours = (uptime_seconds % 86400) / 3600;
    let minutes = (uptime_seconds % 3600) / 60;
    
    if days > 0 {
        format!("{}d {}h {}m", days, hours, minutes)
    } else if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else {
        format!("{}m", minutes)
    }
}

fn truncate_name(name: &str, max_len: usize) -> String {
    if name.len() > max_len {
        format!("{}...", &name[..max_len-3])
    } else {
        name.to_string()
    }
}

fn get_cpu_temperature(components: &Components) -> String {
    let mut cpu_temps = Vec::new();
    
    for component in components {
        let name = component.label().to_lowercase();
        if name.contains("cpu") || name.contains("core") || name.contains("processor") {
            if let Some(temp) = component.temperature() {
                if temp > 0.0 {
                    cpu_temps.push(temp);
                }
            }
        }
    }
    
    if !cpu_temps.is_empty() {
        let avg_temp = cpu_temps.iter().sum::<f32>() / cpu_temps.len() as f32;
        let max_temp = cpu_temps.iter().fold(0.0f32, |a, &b| a.max(b));
        
        let temp_status = if avg_temp > 80.0 {
            "🔴"
        } else if avg_temp > 70.0 {
            "🟡"
        } else {
            "🟢"
        };
        
        format!("🌡️ CPU Temp: {:.1}°C (max: {:.1}°C) {}", avg_temp, max_temp, temp_status)
    } else {
        "🌡️ CPU Temperature: Not available".to_string()
    }
}

fn get_network_summary(networks: &Networks) -> (u64, u64, usize) {
    let mut total_received = 0;
    let mut total_transmitted = 0;
    let mut active_interfaces = 0;
    
    for (_interface_name, network) in networks {
        let received = network.received();
        let transmitted = network.transmitted();
        
        if received > 0 || transmitted > 0 {
            active_interfaces += 1;
            total_received += received;
            total_transmitted += transmitted;
        }
    }
    
    (total_received, total_transmitted, active_interfaces)
}

fn calculate_directory_size(path: &Path) -> Result<u64, std::io::Error> {
    let mut total_size = 0u64;
    
    if !path.is_dir() {
        return Ok(0);
    }
    
    match fs::read_dir(path) {
        Ok(entries) => {
            for entry in entries {
                if let Ok(entry) = entry {
                    if let Ok(metadata) = entry.metadata() {
                        if metadata.is_file() {
                            total_size += metadata.len();
                        }
                    }
                }
            }
        }
        Err(e) => return Err(e),
    }
    
    Ok(total_size)
}
