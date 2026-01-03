# SunSpec Gateway Configuration Reference

This document provides a comprehensive reference for all configuration options available in SunSpec Gateway.

---

## Table of Contents

1. [Configuration File Location](#configuration-file-location)
2. [Top-Level Configuration](#top-level-configuration)
3. [MQTT Configuration](#mqtt-configuration)
4. [Units Configuration](#units-configuration)
5. [Models Configuration](#models-configuration)
6. [Point Configuration](#point-configuration)
7. [Input Types](#input-types)
8. [Device Classes](#device-classes)
9. [State Classes](#state-classes)
10. [Common SunSpec Models](#common-sunspec-models)
11. [Configuration Examples](#configuration-examples)

---

## Configuration File Location

The configuration file is typically located at:
- Docker: `/opt/sunspec_gateway/config.yaml`
- Environment variable: `CONFIG_FILE_PATH`

On first run, the YAML configuration is automatically migrated to the SQLite database. After that, configuration changes can be made through the web UI Settings page.

---

## Top-Level Configuration

```yaml
# MQTT Broker Settings (optional - leave empty for standalone mode)
mqtt_server_addr: "192.168.1.100"    # Optional - MQTT broker address
mqtt_server_port: 1883               # Optional - Default: 1883
mqtt_client_id: "sunspec_gateway"    # Optional - MQTT client ID
mqtt_username: "user"                # Optional - MQTT username
mqtt_password: "password"            # Optional - MQTT password

# Home Assistant Integration
hass_enabled: true                   # Optional - Enable HA discovery (default: true)

# OpenTelemetry Tracing
tracing:                             # Optional - Distributed tracing
  url: "http://otel:4318/v1/traces"
  sample_rate: 0.2                   # 0.0 to 1.0

# Device Connections
units: []                            # Required - List of SunSpec units

# Point Monitoring
models: {}                           # Required - Model-to-points mapping
```

### Standalone Mode (No MQTT)

SunSpec Gateway can run in standalone mode without MQTT. Simply omit or leave `mqtt_server_addr` empty:

```yaml
# No MQTT configuration - standalone web dashboard only
units:
  - addr: "192.168.1.50:502"
    slaves: [1]
models:
  "102":
    - point: "W"
      interval: 15
```

In standalone mode, the web dashboard provides full monitoring capabilities:
- Real-time device data and power flow visualization
- Historical charts and energy metrics
- Device control via the web UI
- Data persistence in SQLite

---

## MQTT Configuration

### Basic MQTT (No Authentication)

```yaml
mqtt_server_addr: "192.168.1.100"
mqtt_server_port: 1883
```

### MQTT with Authentication

```yaml
mqtt_server_addr: "192.168.1.100"
mqtt_server_port: 1883
mqtt_username: "homeassistant"
mqtt_password: "your_secure_password"
```

### MQTT with Custom Client ID

```yaml
mqtt_server_addr: "192.168.1.100"
mqtt_client_id: "solar_gateway_01"
```

### Disable Home Assistant Discovery

```yaml
hass_enabled: false
```

---

## Units Configuration

Units define the SunSpec Modbus devices to connect to.

### Single Device, Single Slave

```yaml
units:
  - addr: "192.168.1.50:502"
    slaves: [1]
```

### Single Device, Multiple Slaves

```yaml
units:
  - addr: "192.168.1.50:502"
    slaves: [1, 3, 4, 5, 6]
```

### Multiple Devices

```yaml
units:
  - addr: "192.168.1.50:502"
    slaves: [1, 3, 4, 5, 6]
  - addr: "192.168.1.51:502"
    slaves: [1, 3]
  - addr: "192.168.1.52:502"
    slaves: [1]
```

### Common Slave ID Patterns

| Slave ID | Typical Device (Generac PWRcell) |
|----------|----------------------------------|
| 1 | Inverter |
| 3 | PV Link 1 |
| 4 | PV Link 2 |
| 5 | PV Link 3 |
| 6 | Battery |

---

## Models Configuration

Models group points by their SunSpec Model ID.

```yaml
models:
  "102":           # Model ID as string
    - point: "W"   # First point
      interval: 15
    - point: "WH"  # Second point
      interval: 60
  "804":
    - point: "SoC"
      interval: 60
```

---

## Point Configuration

Each point can have the following options:

### Required Fields

| Field | Type | Description |
|-------|------|-------------|
| `point` OR `catalog_ref` | string | Point identifier (one required) |
| `interval` | integer | Polling interval in seconds |

### Optional Fields

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `device_class` | string | null | Home Assistant device class |
| `state_class` | string | null | "measurement" or "total_increasing" |
| `display_name` | string | null | Human-readable name |
| `topic_name` | string | null | Custom MQTT topic suffix |
| `uom` | string | null | Unit of measurement |
| `precision` | integer | null | Decimal places (0-8) |
| `value_min` | float | null | Minimum valid value |
| `value_max` | float | null | Maximum valid value |
| `check_deviations` | integer | null | Max deviation between readings |
| `readwrite` | boolean | false | Enable write capability |
| `inputs` | object | null | Input control configuration |
| `input_only` | boolean | false | Control-only (no polling) |
| `scale_factor` | integer | null | Scale factor exponent |
| `homeassistant` | boolean | true | Publish to Home Assistant |

### Basic Point

```yaml
- point: "W"
  interval: 15
```

### Full Featured Point

```yaml
- point: "W"
  interval: 15
  device_class: "power"
  state_class: "measurement"
  display_name: "AC Power Output"
  precision: 1
  uom: "W"
  value_min: -10000.0
  value_max: 10000.0
  check_deviations: 100
```

### Using Catalog References

For nested or repeating group points, use `catalog_ref`:

```yaml
# Single module point
- catalog_ref: ".lithium_ion_string.lithium_ion_string_module[1].ModSoH"
  interval: 60

# Range expansion (modules 1-6)
- catalog_ref: ".lithium_ion_string.lithium_ion_string_module[1..6].ModSoH"
  interval: 60
  display_name: "Module {index} State of Health"
  topic_name: "mod{index}_ModSoH"
```

### Auto-Discovery for Battery Modules

For Model 804, use simple point names to auto-discover modules:

```yaml
"804":
  - point: "ModSoC"   # Auto-expanded based on NMod
    interval: 60
  - point: "ModSoH"
    interval: 60
```

The gateway reads `NMod` from the device and automatically creates:
- `mod1_ModSoC`, `mod2_ModSoC`, etc.
- Display names: "Module 1 State of Charge", etc.

---

## Input Types

For writable points, define the input type:

### Select (Dropdown)

```yaml
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

### Switch (Toggle)

```yaml
- point: "Ena"
  interval: 30
  readwrite: true
  inputs:
    switch:
      on: "ENABLE"
      off: "DISABLE"
```

### Button (Action)

```yaml
- point: "AlmRst"
  interval: 30
  readwrite: true
  input_only: true
  inputs:
    button: "1"
```

### Number (Numeric Input)

```yaml
- point: "SoCRsvMin"
  interval: 60
  readwrite: true
  inputs:
    number:
      min: 0
      max: 100
      step: 1        # Optional
      mode: "slider" # Optional: "slider" or "box"
```

---

## Device Classes

Home Assistant device classes for proper display and units:

| Device Class | Unit | Description |
|--------------|------|-------------|
| `power` | W | Power measurement |
| `energy` | Wh | Energy measurement |
| `voltage` | V | Voltage measurement |
| `current` | A | Current measurement |
| `frequency` | Hz | Frequency measurement |
| `temperature` | °C | Temperature measurement |
| `battery` | % | Battery level |
| `power_factor` | % | Power factor |
| `apparent_power` | VA | Apparent power |
| `reactive_power` | var | Reactive power |

---

## State Classes

State classes define how values accumulate:

| State Class | Usage |
|-------------|-------|
| `measurement` | Instantaneous values (power, voltage, temperature) |
| `total_increasing` | Cumulative counters (energy, Wh) |

---

## Common SunSpec Models

### Model 1 - Common

Basic device identification.

```yaml
"1":
  - point: "Mn"    # Manufacturer
    interval: 300
  - point: "Md"    # Model
    interval: 300
  - point: "SN"    # Serial Number
    interval: 300
```

### Model 102 - Split-Phase Inverter

```yaml
"102":
  - point: "W"
    interval: 15
    device_class: "power"
    state_class: "measurement"
  - point: "WH"
    interval: 60
    device_class: "energy"
    state_class: "total_increasing"
  - point: "PhVphA"
    interval: 15
    device_class: "voltage"
    state_class: "measurement"
  - point: "PhVphB"
    interval: 15
    device_class: "voltage"
    state_class: "measurement"
  - point: "Hz"
    interval: 15
    device_class: "frequency"
    state_class: "measurement"
  - point: "DCV"
    interval: 60
    device_class: "voltage"
    state_class: "measurement"
  - point: "St"
    interval: 60
    display_name: "Inverter Status"
```

### Model 802 - Battery Base

```yaml
"802":
  - point: "SoC"
    interval: 60
    device_class: "battery"
    state_class: "measurement"
    uom: "%"
  - point: "SoH"
    interval: 60
    device_class: "battery"
    state_class: "measurement"
    uom: "%"
  - point: "W"
    interval: 60
    device_class: "power"
    state_class: "measurement"
  - point: "CellVMin"
    interval: 60
    device_class: "voltage"
    state_class: "measurement"
  - point: "CellVMax"
    interval: 60
    device_class: "voltage"
    state_class: "measurement"
```

### Model 804 - Battery Modules (Auto-Discovery)

```yaml
"804":
  - point: "SoC"
    interval: 60
    device_class: "battery"
    state_class: "measurement"
    uom: "%"
  - point: "SoH"
    interval: 60
    device_class: "battery"
    state_class: "measurement"
    uom: "%"
  - point: "A"
    interval: 15
    device_class: "current"
    state_class: "measurement"
  - point: "V"
    interval: 15
    device_class: "voltage"
    state_class: "measurement"
  # Module-level points (auto-expanded)
  - point: "ModSoC"
    interval: 60
    device_class: "battery"
    state_class: "measurement"
    uom: "%"
  - point: "ModSoH"
    interval: 60
    device_class: "battery"
    state_class: "measurement"
    uom: "%"
```

### Model 404 - String Combiner

```yaml
"404":
  - point: "DCW"
    interval: 60
    device_class: "power"
    state_class: "measurement"
  - point: "DCWh"
    interval: 60
    device_class: "energy"
    state_class: "total_increasing"
  - point: "DCV"
    interval: 60
    device_class: "voltage"
    state_class: "measurement"
  - point: "DCA"
    interval: 60
    device_class: "current"
    state_class: "measurement"
  - point: "Tmp"
    interval: 30
    device_class: "temperature"
    state_class: "measurement"
    uom: "°C"
```

### Model 64200 - Generac System Mode

```yaml
"64200":
  - point: "SysMd"
    interval: 60
    display_name: "System Mode"
    readwrite: true
    inputs:
      select:
        - "GRID_CONNECT"
        - "SELF_SUPPLY"
        - "CLEAN_BACKUP"
        - "PRIORITY_BACKUP"
        - "SAFETY_SHUTDOWN"
        - "SELL"
```

### Model 64204 - REbus Grid Meter

```yaml
"64204":
  - point: "Px1"
    interval: 15
    device_class: "power"
    state_class: "measurement"
    display_name: "Power, Phase A"
  - point: "Px2"
    interval: 15
    device_class: "power"
    state_class: "measurement"
    display_name: "Power, Phase B"
  - point: "Whx"
    interval: 60
    device_class: "energy"
    state_class: "total_increasing"
    display_name: "Total Exported Energy"
  - point: "Whin"
    interval: 60
    device_class: "energy"
    state_class: "total_increasing"
    display_name: "Total Imported Energy"
```

### Model 64251 - PV Link

```yaml
"64251":
  - point: "Ena"
    interval: 30
    display_name: "Unit Enable"
    readwrite: true
    inputs:
      switch:
        on: "ENABLE"
        off: "DISABLE"
  - point: "Iin"
    interval: 15
    device_class: "current"
    state_class: "measurement"
  - point: "Vin"
    interval: 15
    device_class: "voltage"
    state_class: "measurement"
  - point: "DCW"
    interval: 15
    device_class: "power"
    state_class: "measurement"
```

---

## Configuration Examples

### Minimal Configuration

```yaml
mqtt_server_addr: "192.168.1.100"
units:
  - addr: "192.168.1.50:502"
    slaves: [1]
models:
  "102":
    - point: "W"
      interval: 15
```

### Full Generac PWRcell System

```yaml
mqtt_server_addr: "192.168.1.100"
mqtt_server_port: 1883
mqtt_username: "homeassistant"
mqtt_password: "secure_password"
hass_enabled: true

units:
  - addr: "192.168.1.50:502"
    slaves: [1, 3, 4, 5, 6]

models:
  # Inverter
  "102":
    - point: "W"
      interval: 15
      device_class: "power"
      state_class: "measurement"
    - point: "WH"
      interval: 60
      device_class: "energy"
      state_class: "total_increasing"
    - point: "PhVphA"
      interval: 15
      device_class: "voltage"
      state_class: "measurement"
    - point: "PhVphB"
      interval: 15
      device_class: "voltage"
      state_class: "measurement"
    - point: "Hz"
      interval: 15
      device_class: "frequency"
      state_class: "measurement"
    - point: "TmpCab"
      interval: 30
      device_class: "temperature"
      state_class: "measurement"
      uom: "°C"
    - point: "St"
      interval: 60
      display_name: "Inverter Status"

  # PV Links
  "64251":
    - point: "DCW"
      interval: 15
      device_class: "power"
      state_class: "measurement"
    - point: "Iin"
      interval: 15
      device_class: "current"
      state_class: "measurement"
    - point: "Vin"
      interval: 15
      device_class: "voltage"
      state_class: "measurement"
    - point: "Ena"
      interval: 30
      display_name: "Unit Enable"
      readwrite: true
      inputs:
        switch:
          on: "ENABLE"
          off: "DISABLE"

  # Battery
  "802":
    - point: "SoC"
      interval: 60
      device_class: "battery"
      state_class: "measurement"
      uom: "%"
    - point: "SoH"
      interval: 60
      device_class: "battery"
      state_class: "measurement"
      uom: "%"
    - point: "W"
      interval: 60
      device_class: "power"
      state_class: "measurement"
    - point: "AlmRst"
      interval: 30
      display_name: "Battery Alarm Reset"
      readwrite: true
      input_only: true
      inputs:
        button: "1"

  # Battery Modules
  "804":
    - point: "SoC"
      interval: 60
      device_class: "battery"
      state_class: "measurement"
      uom: "%"
    - point: "SoH"
      interval: 60
      device_class: "battery"
      state_class: "measurement"
      uom: "%"
    - point: "A"
      interval: 15
      device_class: "current"
      state_class: "measurement"
    - point: "V"
      interval: 15
      device_class: "voltage"
      state_class: "measurement"
    - point: "ModSoC"
      interval: 60
      device_class: "battery"
      state_class: "measurement"
      uom: "%"
    - point: "ModSoH"
      interval: 60
      device_class: "battery"
      state_class: "measurement"
      uom: "%"

  # System Mode Control
  "64200":
    - point: "SysMd"
      interval: 60
      display_name: "System Mode"
      readwrite: true
      inputs:
        select:
          - "GRID_CONNECT"
          - "SELF_SUPPLY"
          - "CLEAN_BACKUP"
          - "PRIORITY_BACKUP"
          - "SELL"

  # Grid Meter
  "64204":
    - point: "Px1"
      interval: 15
      device_class: "power"
      state_class: "measurement"
      display_name: "Grid Power Phase A"
    - point: "Px2"
      interval: 15
      device_class: "power"
      state_class: "measurement"
      display_name: "Grid Power Phase B"
    - point: "Whx"
      interval: 60
      device_class: "energy"
      state_class: "total_increasing"
      display_name: "Grid Export Energy"
    - point: "Whin"
      interval: 60
      device_class: "energy"
      state_class: "total_increasing"
      display_name: "Grid Import Energy"
```

---

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `CONFIG_FILE_PATH` | `./config.yaml` | Path to configuration file |
| `RUST_LOG` | `INFO` | Log level (error, warn, info, debug, trace) |

---

## Notes

- Configuration is stored in SQLite after first load
- Changes via the Settings UI are persisted immediately
- Restart is required for unit/MQTT connection changes
- Point configuration changes take effect on next poll cycle
