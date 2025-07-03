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