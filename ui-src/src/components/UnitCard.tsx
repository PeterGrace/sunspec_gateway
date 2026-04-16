import React from 'react';
import { Server } from 'lucide-react';
import { UnitResponse } from '../types/api';
import { ModelCard } from './ModelCard';

interface UnitCardProps {
  unit: UnitResponse;
  onPointClick?: (modelId: number, pointId: string) => void;
}

export const UnitCard: React.FC<UnitCardProps> = ({ unit, onPointClick }) => {
  const getTotalPoints = () => {
    return unit.models.reduce((total, model) => total + model.points.length, 0);
  };

  return (
    <div className="bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-lg shadow-sm transition-all duration-300">
      {/* Unit Header */}
      <div className="p-5 border-b border-slate-100 dark:border-slate-700">
        <div className="flex items-center space-x-4">
          <div className="w-14 h-14 bg-blue-100 dark:bg-blue-900/30 rounded-lg flex items-center justify-center">
            <Server className="w-6 h-6 text-blue-600 dark:text-blue-400" />
          </div>
          <div>
            <h2 className="font-semibold text-slate-800 dark:text-white text-lg">{unit.unit}</h2>
            <span className="text-sm text-slate-500 dark:text-slate-400 mt-1 block">
              {unit.models.length} models • {getTotalPoints()} points
            </span>
          </div>
        </div>
      </div>

      {/* Models List */}
      <div className="p-5">
        <div className="space-y-3">
          {unit.models.map((model, modelIndex) => (
            <ModelCard
              key={`${model.model}-${modelIndex}`}
              model={model}
              onPointClick={onPointClick}
            />
          ))}
        </div>
      </div>
    </div>
  );
};