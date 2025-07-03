import { UnitList, AppAPIResponse } from '../types/api';

const API_BASE_URL = ''; // Empty string since we're connecting to our own instance, but could be set with a remote instance if needed

export class ApiService {
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
}
