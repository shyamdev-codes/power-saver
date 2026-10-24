# ⚡ Adaptive Power Governor (Power Saver)

[![Rust](https://img.shields.io/badge/rust-2024_edition-orange.svg?logo=rust)](https://www.rust-lang.org/)
[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20Linux-lightgrey.svg)](#cross-platform-support)
[![Binary Size](https://img.shields.io/badge/binary%20size-~1.5MB-brightgreen.svg)]()

An ultra-lightweight, zero-bloat background governor written in pure **Rust** that dynamically throttles CPU power states during idle and non-intensive tasks. It cuts idle power draw, lowers thermals, quiets fan noise, and extends laptop battery life—without sacrificing responsiveness when you need full performance.

---

## 💡 Why Adaptive Power Governor?

Modern operating system schedulers often ramp CPUs to peak boost clocks for minor background tasks (file indexing, browser background tabs, telemetry). This causes:
- **Unnecessary heat & fan spikes** even when the machine is idle.
- **Higher battery drain** on laptops.
- **Wasted electricity** on always-on desktops and home servers.

**Adaptive Power Governor** continuously monitors real CPU load and smoothly scales maximum processor states between energy-conserving baselines and full 100% performance:

```
    [ CPU Load < 20% (Idle / Office Work) ]  ───► Throttle to 50-70% (Cool & Silent)
    [ CPU Load > 60% (Gaming / Compiling) ] ───► Instant 100% Boost (Zero Lag)
```

Unlike third-party tuning utilities that bundle Chromium or heavy frameworks, this tool compiles to a **single ~1.5 MB native binary** with **near-zero CPU footprint (< 0.1%)**.

---

## ✨ Features

- **🚀 Ultra-Lightweight & Native**: Written in 100% safe, fast Rust. Zero Electron, zero webview, zero bloat.
- **🔋 Dual AC / Battery Management**: Automatically adjusts power limits for both plugged-in (AC) and on-battery (DC) operation.
- **🛡️ Crash-Proof Recovery Journal**: Ensures CPU limits are *always* restored to 100% if the system restarts or the app terminates unexpectedly.
- **📊 Energy & Cost Telemetry**: Estimates real-time wattage saved, logs cumulative kWh to CSV, and calculates financial savings using your local power tariff.
- **🖥️ Native Win32 UI (Windows)**: Built using pure Win32 GDI rendering with a modern dark theme and live telemetry graph.
- **🐧 Headless Daemon (Linux)**: Direct `/sys/devices/system/cpu/cpufreq` scaling and native `systemd` user service autostart.

---

## ⚖️ Comparison

| Feature | Windows Default | Generic Tuning Tools | Adaptive Power Governor |
| :--- | :---: | :---: | :---: |
| **RAM Usage** | Built-in | 100 MB – 300 MB (Electron) | **< 8 MB** |
| **Executable Size** | N/A | 50 MB – 150 MB | **~1.5 MB** |
| **Failsafe CPU Restore** | N/A | ❌ Manual | **✅ Hardware Recovery Journal** |
| **Telemetry & Log CSV** | ❌ No | Paid / Complex | **✅ Built-in Daily Tracking** |
| **Open Source** | ❌ Proprietary | ❌ Proprietary | **✅ Free & Open Source (GPL-3.0)** |

---

## 🛠️ Cross-Platform Architecture

The governor uses compile-time target switching (`#[cfg(target_os)]`) to interface with kernel power management APIs natively without any wrapper overhead:

### Windows (`target_os = "windows"`)
- **API**: Windows Power Management (`powrprof.dll`, `GUID_PROCTHROTTLEMAX`).
- **Telemetry**: `GetSystemTimes` & Win32 hardware registry queries.
- **Interface**: Native Win32 GDI window + system tray integration.
- **Autostart**: Windows Run Registry (`HKCU\Software\Microsoft\Windows\CurrentVersion\Run`).

### Linux (`target_os = "linux"`)
- **API**: Linux Kernel CPUFreq sysfs subsystem (`/sys/devices/system/cpu/cpu*/cpufreq/scaling_max_freq`).
- **Telemetry**: `/proc/stat` & `/proc/cpuinfo`.
- **Interface**: Silent headless daemon with structured stdout logging.
- **Autostart**: `systemd` user service (`~/.config/systemd/user/adaptive-power-governor.service`).

---

## 📦 Building from Source

### Prerequisites
- [Rust toolchain](https://www.rust-lang.org/tools/install) (Rust 1.85+ recommended, 2024 edition).

### 1. Clone the repository
```bash
git clone https://github.com/shyamdev-codes/power-saver.git
cd power-saver
```

### 2. Build Release Binary

**On Windows:**
```powershell
cargo build --release
# Output executable: target\release\adaptive_power_governor.exe
```

**On Linux:**
```bash
cargo build --release
# Output binary: target/release/adaptive_power_governor
```

---

## ⚙️ Configuration

The app reads an optional `app_settings.json` file in the same directory to customize currency and energy tariff rates:

```json
{
  "currency_symbol": "₹",
  "tariff_per_kwh": 3.5
}
```

- `currency_symbol`: Your local currency character (`$`, `€`, `£`, `₹`, etc.).
- `tariff_per_kwh`: Cost per kilowatt-hour from your electricity provider.

---

## 🔒 Safety & Recovery Guarantee

Altering CPU clock bounds can be concerning if software crashes while throttled. **Adaptive Power Governor** is designed with defense-in-depth safety:

1. **Recovery Journal**: Whenever a non-100% power limit is applied, a recovery state is written to `recovery_journal.json`.
2. **On Startup**: The app inspects the journal. If an abnormal shutdown occurred previously, it instantly resets the CPU power scheme to 100% before starting.
3. **Graceful Exit**: Normal exit hooks automatically reset limits back to full stock defaults (100%).

---

## 🤝 Contributing

Contributions, bug reports, and suggestions are welcome!
1. Fork the Project.
2. Create your Feature Branch (`git checkout -b feature/AmazingFeature`).
3. Commit your Changes (`git commit -m "Add some AmazingFeature"`).
4. Push to the Branch (`git push origin feature/AmazingFeature`).
5. Open a Pull Request.

---

## 📜 License

Distributed under the **GNU General Public License v3.0** (`GPL-3.0`). See [LICENSE](LICENSE) for details.
