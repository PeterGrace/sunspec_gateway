# SunSpec Gateway

A high-performance, real-time monitoring gateway for SunSpec-compliant solar energy devices. This application polls SunSpec Modbus devices (inverters, PV links, batteries) and publishes metrics to MQTT for integration with Home Assistant and other smart home platforms. It features a beautiful, modern web dashboard for real-time visualization and device control.

---

## Table of Contents

1. [Features](#features)
2. [Screenshots](#screenshots)
3. [Architecture](#architecture)
4. [Quick Start](#quick-start)
5. [Installation](#installation)
   - [Docker (Recommended)](#docker-recommended)
   - [Docker Compose](#docker-compose)
   - [Building from Source](#building-from-source)
6. [Configuration](#configuration)
   - [MQTT Settings](#mqtt-settings)
   - [Units (Devices)](#units-devices)
   - [Models and Points](#models-and-points)
   - [Point Configuration Options](#point-configuration-options)
7. [Web Dashboard](#web-dashboard)
   - [Dashboard Page](#dashboard-page)
   - [Devices Page](#devices-page)
   - [Controls Page](#controls-page)
   - [Points Browser](#points-browser)
   - [Settings Page](#settings-page)
8. [Home Assistant Integration](#home-assistant-integration)
9. [API Reference](#api-reference)
10. [Support Devices](#supported-devices)
11. [Troubleshooting](#troubleshooting)
12. [Development](#development)

---

## Features

### Core Features
- **SunSpec Modbus Polling**: Connects to SunSpec-compliant devices over TCP/IP Modbus
- **MQTT Publishing**: Publishes device metrics to MQTT broker with Home Assistant auto-discovery
- **Standalone Mode**: Run without MQTT - use just the web dashboard for monitoring
- **Multi-Device Support**: Monitor multiple inverters, PV links, and batteries simultaneously
- **Auto-Discovery**: Automatically discovers battery modules (NMod) for Model 804 devices

### Web Dashboard
- **Real-Time Monitoring**: Live power flow visualization with dynamic updates
- **Energy Metrics**: Daily, monthly, and historical energy production/consumption tracking
- **Performance Charts**: Interactive historical charts with multiple time periods
- **Device Control**: Write commands to devices (system mode, enable/disable, etc.)
- **Dark/Light Theme**: Modern UI with theme toggle

### Home Assistant Integration
- **MQTT Auto-Discovery**: Devices automatically appear in Home Assistant
- **Sensors & Controls**: Creates sensors, selects, switches, and buttons
- **Device Classes**: Proper classification (power, energy, voltage, current, temperature, battery)
- **State Classes**: Supports measurement and total_increasing for energy counters

### Advanced Features
- **Points Browser**: Explore all available SunSpec points across connected devices
- **YAML Export**: Generate configuration snippets for points you want to monitor
- **Value Filtering**: Min/max validation and deviation checking
- **Precision Control**: Configurable decimal precision per point
- **Custom Display Names**: Human-readable names for MQTT and dashboard display

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        SunSpec Gateway                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐      │
│  │   Inverter   │    │   PV Link    │    │   Battery    │      │
│  │  (Model 102) │    │ (Model 64251)│    │  (Model 804) │      │
│  └──────┬───────┘    └──────┬───────┘    └──────┬───────┘      │
│         │                   │                   │               │
│         └─────────────┬─────┴───────────────────┘               │
│                       ▼                                         │
│              ┌────────────────┐                                 │
│              │  Modbus TCP    │                                 │
│              │  SunSpec Poll  │                                 │
│              └───────┬────────┘                                 │
│                      │                                          │
│         ┌────────────┼────────────┐                             │
│         ▼            ▼            ▼                             │
│  ┌────────────┐ ┌─────────┐ ┌──────────┐                       │
│  │   SQLite   │ │  MQTT   │ │   REST   │                       │
│  │  History   │ │ Publish │ │   API    │                       │
│  └────────────┘ └────┬────┘ └────┬─────┘                       │
│                      │           │                              │
└──────────────────────┼───────────┼──────────────────────────────┘
                       ▼           ▼
              ┌─────────────┐  ┌─────────────┐
              │    Home     │  │   React     │
              │  Assistant  │  │  Dashboard  │
              └─────────────┘  └─────────────┘
```

---

## Quick Start

### With MQTT/Home Assistant

```bash
# 1. Create a config file
cat > config.yaml << 'EOF'
mqtt_server_addr: "192.168.1.100"
mqtt_server_port: 1883
mqtt_username: "your_user"
mqtt_password: "your_password"
units:
  - addr: "192.168.1.50:502"
    slaves: [1]
models:
  "102":
    - point: "W"
      interval: 15
      device_class: "power"
      state_class: "measurement"
  "804":
    - point: "SoC"
      interval: 60
      device_class: "battery"
      state_class: "measurement"
EOF

# 2. Run with Docker
docker run -d \
  --name sunspec_gateway \
  -p 8300:8300 \
  -v $(pwd)/config.yaml:/opt/sunspec_gateway/config.yaml \
  docker.io/petergrace/sunspec-gateway:latest

# 3. Open the dashboard
open http://localhost:8300
```

### Standalone Mode (No MQTT)

You can run SunSpec Gateway without MQTT, using just the web dashboard for monitoring:

```bash
# 1. Create a minimal config (no mqtt_server_addr)
cat > config.yaml << 'EOF'
units:
  - addr: "192.168.1.50:502"
    slaves: [1]
models:
  "102":
    - point: "W"
      interval: 15
  "804":
    - point: "SoC"
      interval: 60
EOF

# 2. Run with Docker
docker run -d \
  --name sunspec_gateway \
  -p 8300:8300 \
  -v $(pwd)/config.yaml:/opt/sunspec_gateway/config.yaml \
  docker.io/petergrace/sunspec-gateway:latest

# 3. Open the dashboard - all monitoring works without MQTT!
open http://localhost:8300
```

In standalone mode:
- ✅ Web dashboard works fully (real-time monitoring, charts, device details)
- ✅ Historical data is stored in SQLite
- ✅ Device controls work via the web UI
- ❌ No Home Assistant integration
- ❌ No MQTT publishing

---

## Installation

### Docker (Recommended)

The easiest way to run SunSpec Gateway is using the pre-built Docker image, available for both AMD64 (x86_64) and ARM64 (Raspberry Pi 3+) architectures.

```bash
docker run -d \
  --name sunspec_gateway \
  -p 8300:8300 \
  -v /path/to/config.yaml:/opt/sunspec_gateway/config.yaml \
  -v /path/to/sunspec_gateway.db:/opt/sunspec_gateway/sunspec_gateway.db \
  docker.io/petergrace/sunspec-gateway:latest
```

### Docker Compose

For easier management, use Docker Compose:

```yaml
# docker-compose.yaml
services:
  sunspec_gateway:
    image: docker.io/petergrace/sunspec-gateway:latest
    container_name: sunspec_gateway
    restart: unless-stopped
    ports:
      - "8300:8300"
    volumes:
      - "./config.yaml:/opt/sunspec_gateway/config.yaml"
      - "./sunspec_gateway.db:/opt/sunspec_gateway/sunspec_gateway.db"
```

Then run:

```bash
docker compose up -d
docker compose logs -f  # Watch logs
```

### Building from Source

#### Prerequisites
- Docker (recommended for building)
- Rust 1.83+ (if building natively)
- Node.js 20+ (for UI development)

#### Build with Docker

```bash
git clone https://github.com/petergrace/sunspec-gateway.git
cd sunspec-gateway
docker build -t sunspec_gateway .
```

This multi-stage build:
1. Compiles the React UI
2. Compiles the Rust backend with SQLx migrations
3. Creates a lightweight Ubuntu runtime image

#### Run Your Local Build

```yaml
# docker-compose.yaml
services:
  sunspec_gateway:
    image: sunspec_gateway:latest
    pull_policy: never  # Use local image
    ports:
      - "8300:8300"
    volumes:
      - "./config.yaml:/opt/sunspec_gateway/config.yaml"
      - "./sunspec_gateway.db:/opt/sunspec_gateway/sunspec_gateway.db"
```

```bash
docker compose up -d
```

---

## Configuration

Configuration is stored in `config.yaml` and can also be managed through the web UI Settings page. On first run, the config file is automatically migrated to the SQLite database for persistence.

### Minimal Configuration

```yaml
mqtt_server_addr: "192.168.1.100"
units:
  - addr: "192.168.1.50:502"
    slaves: [1]
models: {}
```

### Full Configuration Example

```yaml
# MQTT Broker Settings
mqtt_server_addr: "192.168.1.100"
mqtt_server_port: 1883
mqtt_username: "homeassistant"
mqtt_password: "your_secure_password"

# Enable Home Assistant MQTT Discovery
hass_enabled: true

# OpenTelemetry Tracing (optional)
tracing:
  url: http://otel-collector:4318/v1/traces
  sample_rate: 0.2

# SunSpec Devices
units:
  # Multiple devices with multiple slaves
  - addr: "192.168.1.50:502"
    slaves: [1, 3, 4, 5, 6]
  - addr: "192.168.1.51:502"
    slaves: [1]

# Model Point Configurations
models:
  # Inverter (Split-Phase)
  "102":
    - point: "W"
      interval: 15
      device_class: "power"
      state_class: "measurement"
      precision: 1
      value_min: -10000.0
      value_max: 10000.0
    - point: "WH"
      interval: 60
      device_class: "energy"
      state_class: "total_increasing"
    - point: "PhVphA"
      interval: 15
      device_class: "voltage"
      state_class: "measurement"
      precision: 1

  # Battery Base (Model 802)
  "802":
    - point: "SoC"
      interval: 60
      device_class: "battery"
      state_class: "measurement"
      uom: "%"
      value_min: 0
      value_max: 100

  # Battery Modules (Model 804) - Auto-Discovery
  "804":
    - point: "ModSoC"  # Auto-expanded to mod1_ModSoC, mod2_ModSoC, etc.
      interval: 60
      device_class: "battery"
      state_class: "measurement"
      uom: "%"
    - point: "ModSoH"
      interval: 60
      device_class: "battery"
      state_class: "measurement"
      uom: "%"

  # PV Link / String Combiner
  "64251":
    - point: "Ena"
      interval: 30
      display_name: "Unit Enable"
      readwrite: true
      inputs:
        switch:
          on: ENABLE
          off: DISABLE

  # System Mode Control
  "64200":
    - point: "SysMd"
      interval: 60
      readwrite: true
      inputs:
        select:
          - "GRID_CONNECT"
          - "SELF_SUPPLY"
          - "CLEAN_BACKUP"
          - "PRIORITY_BACKUP"
          - "SELL"
```

### MQTT Settings

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `mqtt_server_addr` | string | required | MQTT broker IP or hostname |
| `mqtt_server_port` | integer | 1883 | MQTT broker port |
| `mqtt_client_id` | string | "sunspec_gateway" | MQTT client identifier |
| `mqtt_username` | string | optional | MQTT authentication username |
| `mqtt_password` | string | optional | MQTT authentication password |
| `hass_enabled` | boolean | true | Enable Home Assistant auto-discovery |

### Units (Devices)

Each unit represents a SunSpec Modbus device:

| Field | Type | Description |
|-------|------|-------------|
| `addr` | string | IP:Port of the Modbus device (e.g., "192.168.1.50:502") |
| `slaves` | array | List of Modbus slave IDs to poll on this device |

### Models and Points

Models are organized by their SunSpec Model ID. Common models include:

| Model ID | Description |
|----------|-------------|
| 1 | Common Model (device info) |
| 101-103 | Single/Split/3-Phase Inverter |
| 401-404 | String Combiner |
| 802 | Battery Base Model |
| 804 | Lithium-Ion Battery Strings |
| 64200 | Generac PWRcell System Mode |
| 64204 | REbus Grid Meter |
| 64207 | REbus Unit Status |
| 64208 | Generac Inverter Status |
| 64251 | PV Link |

### Point Configuration Options

| Field | Type | Description |
|-------|------|-------------|
| `point` | string | SunSpec point name (e.g., "W", "SoC") |
| `catalog_ref` | string | Full catalog path for nested points |
| `interval` | integer | Polling interval in seconds |
| `device_class` | string | Home Assistant device class |
| `state_class` | string | "measurement" or "total_increasing" |
| `display_name` | string | Human-readable name for UI/MQTT |
| `topic_name` | string | Custom MQTT topic suffix |
| `uom` | string | Unit of measurement (e.g., "V", "A", "°C") |
| `precision` | integer | Decimal places (0-8) |
| `value_min` | float | Minimum valid value |
| `value_max` | float | Maximum valid value |
| `check_deviations` | integer | Max allowed deviation between readings |
| `readwrite` | boolean | Enable write capability |
| `inputs` | object | Input control type (select, switch, button, number) |
| `input_only` | boolean | Control-only point (no polling) |
| `scale_factor` | integer | Scale factor exponent |

---

## Web Dashboard

Access the web dashboard at `http://your-host:8300/`

### Dashboard Page

The main dashboard provides an at-a-glance view of your solar energy system:

- **Power Flow Diagram**: Visual representation of solar → battery → grid → home power flow
- **Energy Metrics**: Today's yield, consumption, grid import/export, battery activity
- **Performance Chart**: Historical data with selectable time periods (Today, Yesterday, 7 Days, 30 Days)
- **Device Tables**: Status of inverters, PV links, and batteries with key metrics

### Devices Page

Detailed view of all monitored devices and their points:

- **Grouped by Device**: Each serial number shown with its models
- **Expandable Models**: Click to see all monitored points
- **Live Values**: Real-time updates every 5 seconds
- **Point Categories**: Color-coded icons (power, voltage, temperature, battery, status)

### Controls Page

Interactive controls for writable points:

- **System Mode**: Change between Grid Connect, Self Supply, Clean Backup, etc.
- **Battery Operations**: Enable/disable charge modes, reset alarms
- **PV Link Enable**: Turn solar inputs on/off
- **Confirmation Modal**: All writes require confirmation to prevent accidents

### Points Browser

Explore all available SunSpec points on connected devices:

- **Browse All Points**: See every point available on each model
- **Filter & Search**: Find specific points by name or model
- **YAML Export**: Generate config snippets to add to your configuration
- **Point Details**: View data types, descriptions, and current values

### Settings Page

Configure the gateway through the web UI:

- **MQTT Configuration**: Broker address, port, credentials
- **Device Management**: Add/remove units and slaves
- **Model Configuration**: Edit monitored points per model
- **System Status**: View connection status of devices and MQTT
- **Restart**: Apply configuration changes

---

## Home Assistant Integration

SunSpec Gateway integrates seamlessly with Home Assistant via MQTT Discovery.

### Automatic Entity Creation

When enabled, the gateway automatically creates:

- **Sensors**: For all monitored read-only points
- **Selects**: For points with enumerated options (e.g., System Mode)
- **Switches**: For on/off controls (e.g., PV Link Enable)
- **Buttons**: For action triggers (e.g., Alarm Reset)
- **Numbers**: For numeric inputs with min/max ranges

### Entity Naming

Entities follow the naming pattern:
```
sensor.<serial_number>_<model_name>_<point_name>
```

Example:
```
sensor.0001002364be_inverter_102_w
sensor.0001002364be_battery_804_soc
select.0001002364be_system_64200_sysmd
```

### Example Lovelace Card

```yaml
type: entities
title: Solar System
entities:
  - entity: sensor.0001002364be_inverter_102_w
    name: Inverter Power
  - entity: sensor.0001002364be_battery_804_soc
    name: Battery State of Charge
  - entity: select.0001002364be_system_64200_sysmd
    name: System Mode
```

---

## API Reference

The gateway exposes a REST API at `/api/v1/`. Full OpenAPI documentation is available at `/scalar`.

### Key Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/v1/dashboard/devices` | GET | Get devices grouped by type |
| `/api/v1/dashboard/metrics` | GET | Get aggregated energy metrics |
| `/api/v1/dashboard/history` | GET | Get historical data for charts |
| `/api/v1/dashboard/all` | GET | Get all device data with points |
| `/api/v1/dashboard/stream` | GET | SSE stream for real-time updates |
| `/api/v1/points/units` | GET | List all units and their models |
| `/api/v1/settings/config` | GET/PUT | Get or update configuration |
| `/api/v1/settings/status` | GET | Get system connection status |
| `/api/v1/settings/restart` | POST | Restart the gateway service |
| `/api/v1/controls/points` | GET | Get writable control points |
| `/api/v1/controls/write` | POST | Write a value to a control point |

---

## Supported Devices

### Tested Devices

- **Generac PWRcell**: Inverter, Battery, PV Links, REbus Beacon
- **SolarEdge** (with SunSpec Modbus enabled)
- **Fronius** (with SunSpec Modbus enabled)

### Compatible Devices

Any device implementing the SunSpec Information Model should work. Common compatibility includes:

- Grid-tied inverters (Models 101-103, 111-113)
- Battery systems (Models 802-804)
- String combiners (Models 401-404)
- Meters (Models 201-204)

---

## Troubleshooting

### Common Issues

**Cannot connect to device**
```
ERROR Unable to create connection to SunSpec Unit
```
- Verify the IP address and port are correct
- Check that Modbus TCP is enabled on the device
- Ensure no firewall is blocking port 502
- Verify the slave ID is correct

**MQTT connection failed**
```
ERROR Couldn't create mqtt connection object
```
- Verify MQTT broker is running
- Check credentials are correct
- Ensure the broker allows the connection

**No data appearing**
- Check the models configuration matches your device
- Verify the polling intervals are reasonable
- Look for point errors in the logs

### Debug Logging

Set the `RUST_LOG` environment variable for detailed logging:

```bash
docker run -e RUST_LOG=debug ...
```

Log levels: `error`, `warn`, `info`, `debug`, `trace`

### Check Device Status

Visit `/ui/settings` to see the connection status of all devices and MQTT.

---

## Development

### Project Structure

```
sunspec_gateway/
├── src/                    # Rust backend
│   ├── main.rs            # Application entry point
│   ├── sunspec_poll.rs    # SunSpec device polling
│   ├── mqtt_poll.rs       # MQTT publishing
│   ├── modules/           # API modules
│   │   ├── dashboard/     # Dashboard API
│   │   ├── controls/      # Device control API
│   │   ├── points/        # Points browser API
│   │   └── settings/      # Settings API
│   └── payload.rs         # MQTT payload generation
├── ui-src/                # React frontend
│   ├── src/
│   │   ├── pages/         # Main page components
│   │   ├── components/    # Reusable UI components
│   │   └── services/      # API client
│   └── package.json
├── migrations/            # SQLite migrations
├── models/                # SunSpec model definitions
├── Dockerfile             # Multi-stage build
└── docker-compose.yaml
```

### Running for Development

```bash
# Backend (Rust)
cargo run

# Frontend (React)
cd ui-src
npm install
npm run dev
```

### Building

```bash
# Full Docker build
docker build -t sunspec_gateway .

# Rust only (native)
cargo build --release

# UI only
cd ui-src && npm run build
```

---

## License

MIT License - See [LICENSE](LICENSE) for details.

## Contributing

Contributions are welcome! Please submit issues and pull requests on GitHub.

## Acknowledgments

- [SunSpec Alliance](https://sunspec.org/) for the Information Model specification
- [Home Assistant](https://www.home-assistant.io/) for MQTT discovery
- Built with Rust, Axum, React, and Recharts
