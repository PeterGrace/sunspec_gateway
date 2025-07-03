import React from 'react';
import { Filter, X } from 'lucide-react';
import { UnitList } from '../types/api';

interface FilterPanelProps {
  isOpen: boolean;
  onToggle: () => void;
  filters: {
    model: string;
  };
  onFilterChange: (key: string, value: string) => void;
  data: UnitList | null;
}

export const FilterPanel: React.FC<FilterPanelProps> = ({ 
  isOpen, 
  onToggle, 
  filters, 
  onFilterChange,
  data
}) => {
  // Extract unique models from the data
  const availableModels = React.useMemo(() => {
    if (!data) return [];
    
    const modelsMap = new Map<number, { id: number; name: string }>();
    
    data.units.forEach(unit => {
      unit.models.forEach(model => {
        if (!modelsMap.has(model.model)) {
          modelsMap.set(model.model, {
            id: model.model,
            name: model.name
          });
        }
      });
    });
    
    return Array.from(modelsMap.values()).sort((a, b) => a.id - b.id);
  }, [data]);

  if (!isOpen) {
    return (
      <button
        onClick={onToggle}
        className="flex items-center space-x-2 px-4 py-2 bg-white border border-slate-200 rounded-lg hover:bg-slate-50 transition-colors duration-200"
      >
        <Filter className="w-4 h-4 text-slate-500" />
        <span className="text-sm text-slate-600">Filters</span>
      </button>
    );
  }

  return (
    <div className="bg-white border border-slate-200 rounded-lg p-4 shadow-sm">
      <div className="flex items-center justify-between mb-4">
        <h3 className="font-semibold text-slate-800">Filters</h3>
        <button
          onClick={onToggle}
          className="p-1 hover:bg-slate-100 rounded transition-colors duration-200"
        >
          <X className="w-4 h-4 text-slate-500" />
        </button>
      </div>
      
      <div className="space-y-4">
        <div>
          <label className="block text-sm font-medium text-slate-700 mb-2">
            Model Type
          </label>
          <select
            value={filters.model}
            onChange={(e) => onFilterChange('model', e.target.value)}
            className="w-full px-3 py-2 border border-slate-200 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
          >
            <option value="">All Models</option>
            {availableModels.map(model => (
              <option key={model.id} value={model.id.toString()}>
                {model.name} (Model {model.id})
              </option>
            ))}
          </select>
        </div>
      </div>
    </div>
  );
};