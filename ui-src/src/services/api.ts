import {
  UnitList,
  DashboardDevices,
  DashboardMetrics,
  PowerFlow,
  DashboardState,
  HistoryResponse
} from '../types/api';

const API_BASE_URL = '';

export class ApiService {
  // Points API (existing)
  static async getAllPoints(): Promise<UnitList> {
    const response = await fetch(`${API_BASE_URL}/api/v1/points`);
    if (!response.ok) {
      throw new Error(`Failed to fetch points: ${response.statusText}`);
    }
    return await response.json();
  }

  static async getPointDetail(model: number, point: string): Promise<void> {
    const response = await fetch(`${API_BASE_URL}/api/v1/points/${model}/${point}`);
    if (!response.ok) {
      throw new Error(`Failed to fetch point detail: ${response.statusText}`);
    }
  }

  // Dashboard API
  static async getDashboardDevices(): Promise<DashboardDevices> {
    const response = await fetch(`${API_BASE_URL}/api/v1/dashboard/devices`);
    if (!response.ok) {
      throw new Error(`Failed to fetch dashboard devices: ${response.statusText}`);
    }
    return await response.json();
  }

  static async getDashboardMetrics(): Promise<DashboardMetrics> {
    const timezoneOffset = new Date().getTimezoneOffset();
    const response = await fetch(`${API_BASE_URL}/api/v1/dashboard/metrics?timezone_offset=${timezoneOffset}`);
    if (!response.ok) {
      throw new Error(`Failed to fetch dashboard metrics: ${response.statusText}`);
    }
    return await response.json();
  }

  static async getPowerFlow(): Promise<PowerFlow> {
    const response = await fetch(`${API_BASE_URL}/api/v1/dashboard/power-flow`);
    if (!response.ok) {
      throw new Error(`Failed to fetch power flow: ${response.statusText}`);
    }
    return await response.json();
  }

  static async getHistory(period: string = 'today'): Promise<HistoryResponse> {
    const timezoneOffset = new Date().getTimezoneOffset();
    const response = await fetch(`${API_BASE_URL}/api/v1/dashboard/history?period=${period}&timezone_offset=${timezoneOffset}`);

    if (!response.ok) {
      throw new Error(`Failed to fetch history: ${response.statusText}`);
    }
    return await response.json();
  }

  // SSE Stream for real-time updates
  static createDashboardStream(
    onMessage: (state: DashboardState) => void,
    onError?: (error: Event) => void
  ): EventSource {
    const eventSource = new EventSource(`${API_BASE_URL}/api/v1/dashboard/stream`);

    eventSource.addEventListener('dashboard', (event) => {
      try {
        const data = JSON.parse(event.data) as DashboardState;
        onMessage(data);
      } catch (e) {
        console.error('Failed to parse dashboard event:', e);
      }
    });

    eventSource.onerror = (error) => {
      console.error('Dashboard SSE error:', error);
      if (onError) {
        onError(error);
      }
    };

    return eventSource;
  }

  // Controls API
  static async getControlPoints(): Promise<{ points: any[] }> {
    const response = await fetch(`${API_BASE_URL}/api/v1/controls/points`);
    if (!response.ok) {
      throw new Error(`Failed to fetch control points: ${response.statusText}`);
    }
    return await response.json();
  }

  static async writeControlPoint(request: {
    serial_number: string;
    model_id: number;
    point_name: string;
    value: string;
  }): Promise<{ success: boolean; message: string }> {
    const response = await fetch(`${API_BASE_URL}/api/v1/controls/write`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(request),
    });

    const result = await response.json();
    if (!response.ok) {
      throw new Error(result.message || 'Write failed');
    }
    return result;
  }
}
