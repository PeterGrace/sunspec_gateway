import React, { useState } from 'react';
import { Info, Settings } from 'lucide-react';
import { PointResponse } from '../types/api';

interface PointCardProps {
  point: PointResponse;
  onPointClick?: (modelId: number, pointId: string) => void;
  showModelInfo?: boolean;
  modelName?: string;
}

export const PointCard: React.FC<PointCardProps> = ({ 
  point, 
  onPointClick, 
  showModelInfo = false, 
  modelName 
}) => {

  const handlePointClick = (e: React.MouseEvent) => {
    e.stopPropagation();
    if (onPointClick) {
      onPointClick(point.model, point.name);
    }
  };

  return (
    <div className="bg-slate-50 border border-slate-200 rounded-lg p-4 hover:bg-slate-100 transition-all duration-200">
      <div className="flex items-start justify-between">
        <div className="flex items-start space-x-3 flex-1 min-w-0">
          <div className="w-10 h-10 bg-purple-100 rounded-lg flex items-center justify-center flex-shrink-0">
            <Info className="w-5 h-5 text-purple-600" />
          </div>
          <div className="flex-1 min-w-0">
            <h4 className="font-semibold text-slate-800 text-sm">{point.name}</h4>
            {showModelInfo && modelName && (
              <p className="text-xs text-slate-500 mt-1">{modelName} (Model {point.model})</p>
            )}
            <p className="text-sm text-slate-600 mt-2 leading-relaxed">
              {point.description}
            </p>
          </div>
        </div>
        <div className="flex-shrink-0 ml-3">
          <button
            onClick={handlePointClick}
            className="inline-flex items-center space-x-1 px-3 py-1.5 text-xs bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors duration-200"
          >
            <Settings className="w-3 h-3" />
            <span>Configure</span>
          </button>
        </div>
      </div>
    </div>
  );
};