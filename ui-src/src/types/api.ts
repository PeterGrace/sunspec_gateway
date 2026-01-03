// Config Types
export interface PointConfig {
  point?: string;
  catalog_ref?: string;
  interval: number;
  device_class?: string;
  display_name?: string;
  state_class?: string;
  uom?: string;
  precision?: number;
  readwrite?: boolean;
  homeassistant?: boolean;
  scale_factor?: number;
  // inputs?: InputType; // Complex type, skipping for now or can be added later
  input_only?: boolean;
  value_min?: number;
  value_max?: number;
  check_deviations?: number;
  topic_name?: string;
}

export interface UnitConfig {
  addr: string;
  slaves: number[];
  tls?: any;
}

export interface GatewayConfig {
  hass_enabled?: boolean;
  units: UnitConfig[];
  models: Record<string, PointConfig[]>;
  mqtt_server_addr: string;
  mqtt_server_port?: number;
  mqtt_client_id?: string;
  mqtt_username?: string;
  mqtt_password?: string;
  // tracing?: TracingConfig;
}

export interface DeviceStatus {
  name: string;
  connected: boolean;
  last_seen: number;
  last_error?: string;
}

export interface SystemStatus {
  mqtt_connected: boolean;
  mqtt_enabled?: boolean;
  mqtt_last_error?: string;
  devices: DeviceStatus[];
}

export interface PointResponse {
  model: number;
  name: string;
  description: string;
}

export interface ModelResponse {
  model: number;
  name: string;
  description: string;
  points: PointResponse[];
}

export interface UnitResponse {
  unit: string;
  models: ModelResponse[];
}

export interface UnitList {
  units: UnitResponse[];
}

export interface AppAPIResponse {
  message: string;
  data?: any;
}

// Dashboard Types
export type DeviceType = 'inverter' | 'pvlink' | 'battery' | 'stringcombiner' | 'unknown';

export interface DeviceData {
  serial_number: string;
  model_name: string;
  model_id: number;
  device_type: DeviceType;
  operating_mode?: string;
  state?: string;
  power?: number;
  voltage?: number;
  temperature?: number;
  lifetime_energy?: number;
  energy_today?: number;
  soc?: number;
  soh?: number;
  dc_current?: number;
  frequency?: number;
  last_updated: string;
}

export interface DashboardDevices {
  inverters: DeviceData[];
  pv_links: DeviceData[];  // Combined PV Links and String Combiners
  batteries: DeviceData[];
}

export interface DashboardMetrics {
  yield_today: number;
  yield_yesterday: number;
  consumption_today: number;
  consumption_yesterday: number;
  grid_net_today: number;
  grid_export_today: number;
  grid_import_today: number;
  grid_net_month: number;
  battery_in_today: number;
  battery_out_today: number;
}

export interface PowerFlow {
  solar_power: number;
  grid_power: number;
  battery_power: number;
  consumption_power: number;
  solar_active: boolean;
  grid_connected: boolean;
  battery_connected: boolean;
}

export type AlertSeverity = 'info' | 'warning' | 'error' | 'critical';

export interface DeviceAlert {
  serial_number: string;
  alert_type: string;
  message: string;
  timestamp: string;
  severity: AlertSeverity;
}

export type ControlType = 'select' | 'switch' | 'number' | 'button';

export interface QuickControl {
  id: string;
  name: string;
  control_type: ControlType;
  current_value: string;
  options?: string[];
  target_serial: string;
  model_id: number;
  point_name: string;
}

export interface DashboardState {
  devices: DashboardDevices;
  metrics: DashboardMetrics;
  power_flow: PowerFlow;
  alerts: DeviceAlert[];
  controls: QuickControl[];
  timestamp: string;
}

// History types for performance chart
export interface HistoryDataPoint {
  timestamp: string;
  solar: number;
  battery: number;
  grid: number;
  consumption: number;
}

export interface HistoryResponse {
  data: HistoryDataPoint[];
  period: string;
}