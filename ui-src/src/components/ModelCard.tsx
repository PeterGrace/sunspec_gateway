import React, { useState } from 'react';
import { ChevronRight, ChevronDown, Cpu, Zap } from 'lucide-react';
import { ModelResponse } from '../types/api';
import { PointCard } from './PointCard';

interface ModelCardProps {
  model: ModelResponse;
  onPointClick?: (modelId: number, pointId: string) => void;
}

export const ModelCard: React.FC<ModelCardProps> = ({ model, onPointClick }) => {
  const [isExpanded, setIsExpanded] = useState(false);

  const getModelIcon = (modelId: number) => {
    if (modelId === 1) return <Cpu className="w-5 h-5 text-slate-600" />;
    return <Zap className="w-5 h-5 text-green-600" />;
  };

  const getModelColor = (modelId: number) => {
    if (modelId === 1) return 'bg-slate-100';
    return 'bg-green-100';
  };

  return (
    <div className="bg-slate-50 border border-slate-200 rounded-lg hover:bg-slate-100 transition-all duration-200">
      <div 
        className="p-4 flex items-center justify-between cursor-pointer"
        onClick={() => setIsExpanded(!isExpanded)}
      >
        <div className="flex items-center space-x-3">
          <div className={`w-12 h-12 ${getModelColor(model.model)} rounded-lg flex items-center justify-center`}>
            {getModelIcon(model.model)}
          </div>
          <div>
            <h3 className="font-semibold text-slate-800 text-base">{model.name}</h3>
            <p className="text-sm text-slate-500">{model.description}</p>
            <p className="text-xs text-slate-400 mt-1">Model {model.model}</p>
          </div>
        </div>
        <div className="flex items-center space-x-2">
          <span className="text-xs bg-blue-100 text-blue-600 px-2 py-1 rounded-full">
            {model.points.length} points
          </span>
          {isExpanded ? (
            <ChevronDown className="w-5 h-5 text-slate-400" />
          ) : (
            <ChevronRight className="w-5 h-5 text-slate-400" />
          )}
        </div>
      </div>
      
      {isExpanded && (
        <div className="px-4 pb-4 border-t border-slate-200">
          <div className="pt-4 space-y-3">
            {model.points.map((point, index) => (
              <PointCard 
                key={`${point.model}-${point.name}-${index}`}
                point={point}
                onPointClick={onPointClick}
                showModelInfo={false}
              />
            ))}
          </div>
        </div>
      )}
    </div>
  );
};