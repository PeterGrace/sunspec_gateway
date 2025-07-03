import React, { useState } from 'react';
import { ChevronRight, ChevronDown, Info } from 'lucide-react';
import { PointResponse } from '../types/api';
import { YamlModal } from './YamlModal';

interface PointCardProps {
  point: PointResponse;
  onPointClick?: (modelId: number, pointId: string) => void;
}

export const PointCard: React.FC<PointCardProps> = ({ point, onPointClick }) => {
  const [isExpanded, setIsExpanded] = useState(false);
  const [showModal, setShowModal] = useState(false);

  const handleExpandClick = () => {
    setIsExpanded(!isExpanded);
  };

  const handlePointClick = (e: React.MouseEvent) => {
    e.stopPropagation();
    setShowModal(true);
    if (onPointClick) {
      onPointClick(point.model, point.name);
    }
  };

  return (
    <>
      <div className="bg-white border border-slate-200 rounded-lg shadow-sm hover:shadow-md transition-all duration-200">
        <div 
          className="p-4 flex items-center justify-between cursor-pointer"
          onClick={handleExpandClick}
        >
          <div className="flex items-center space-x-3">
            <div className="w-10 h-10 bg-blue-100 rounded-lg flex items-center justify-center">
              <Info className="w-5 h-5 text-blue-600" />
            </div>
            <div>
              <h4 className="font-semibold text-slate-800">{point.name}</h4>
              <p className="text-sm text-slate-500">Model {point.model}</p>
            </div>
          </div>
          <div className="flex items-center space-x-2">
            <button
              onClick={handlePointClick}
              className="text-xs bg-blue-600 text-white px-3 py-1 rounded-full font-medium hover:bg-blue-700 transition-colors duration-200"
            >
              Configure
            </button>
            {isExpanded ? (
              <ChevronDown className="w-5 h-5 text-slate-400" />
            ) : (
              <ChevronRight className="w-5 h-5 text-slate-400" />
            )}
          </div>
        </div>
        
        {isExpanded && (
          <div className="px-4 pb-4 border-t border-slate-100">
            <div className="pt-3">
              <p className="text-sm text-slate-600 leading-relaxed">
                {point.description}
              </p>
              <div className="mt-3 flex items-center space-x-4 text-xs text-slate-500">
                <span>Model: {point.model}</span>
                <span>•</span>
                <span>Name: {point.name}</span>
              </div>
            </div>
          </div>
        )}
      </div>

      <YamlModal
        isOpen={showModal}
        onClose={() => setShowModal(false)}
        modelId={point.model}
        pointId={point.name}
      />
    </>
  );
};