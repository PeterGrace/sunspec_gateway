import React, { useState } from 'react';
import { ChevronRight, ChevronDown, Server } from 'lucide-react';
import { UnitResponse } from '../types/api';
import { ModelCard } from './ModelCard';

interface UnitCardProps {
  unit: UnitResponse;
  onPointClick?: (modelId: number, pointId: string) => void;
}

export const UnitCard: React.FC<UnitCardProps> = ({ unit, onPointClick }) => {
  const [isExpanded, setIsExpanded] = useState(false);

  const getTotalPoints = () => {
    return unit.models.reduce((total, model) => total + model.points.length, 0);
  };

  return (
    <div className="bg-white border border-slate-200 rounded-lg shadow-sm hover:shadow-md transition-all duration-200">
      <div 
        className="p-5 flex items-center justify-between cursor-pointer"
        onClick={() => setIsExpanded(!isExpanded)}
      >
        <div className="flex items-center space-x-4">
          <div className="w-14 h-14 bg-blue-100 rounded-lg flex items-center justify-center">
            <Server className="w-6 h-6 text-blue-600" />
          </div>
          <div>
            <h2 className="font-semibold text-slate-800 text-lg">{unit.unit}</h2>
            <span className="text-sm text-slate-500 mt-1 block">
              {unit.models.length} models • {getTotalPoints()} points
            </span>
          </div>
        </div>
        <div className="flex items-center space-x-2">
          <span className="text-xs bg-blue-100 text-blue-600 px-3 py-1 rounded-full font-medium">
            Unit
          </span>
          {isExpanded ? (
            <ChevronDown className="w-5 h-5 text-slate-400" />
          ) : (
            <ChevronRight className="w-5 h-5 text-slate-400" />
          )}
        </div>
      </div>
      
      {isExpanded && (
        <div className="px-5 pb-5 border-t border-slate-100">
          <div className="pt-5 space-y-4">
            {unit.models.map((model, index) => (
              <ModelCard 
                key={`${model.model}-${index}`}
                model={model}
                onPointClick={onPointClick}
              />
            ))}
          </div>
        </div>
      )}
    </div>
  );
};