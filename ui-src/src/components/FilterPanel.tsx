import React from 'react';
import { Filter, X, CheckCircle } from 'lucide-react';
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
  const handleFilterChange = (key: string, value: string) => {
    onFilterChange(key, value);
    // Auto-minimize the filter panel after selection
    if (value !== '') {
      onToggle();
    }
  };

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

  // Check if any filters are active
  const hasActiveFilters = filters.model !== '';
  const activeFilterCount = Object.values(filters).filter(value => value !== '').length;

  if (!isOpen) {
    return (
      <div className="relative">
        <button
          onClick={onToggle}
          className={`flex items-center space-x-2 px-4 py-2 rounded-lg transition-colors duration-200 ${
            hasActiveFilters 
              ? 'bg-blue-100 border border-blue-200 text-blue-700 hover:bg-blue-200' 
              : 'bg-white border border-slate-200 text-slate-600 hover:bg-slate-50'
          }`}
        >
          <Filter className={`w-4 h-4 ${hasActiveFilters ? 'text-blue-600' : 'text-slate-500'}`} />
          <span className="text-sm">Filters</span>
          {hasActiveFilters && (
            <span className="text-xs bg-blue-600 text-white px-2 py-0.5 rounded-full font-medium">
              {activeFilterCount}
            </span>
          )}
        </button>
        {hasActiveFilters && (
          <div className="absolute -top-1 -right-1 w-3 h-3 bg-blue-600 rounded-full border-2 border-white"></div>
        )}
      </div>
    );
  }

  return (
    <div className="absolute right-0 top-12 bg-white border border-slate-200 rounded-lg p-4 shadow-lg z-10 min-w-64">
      <div className="flex items-center justify-between mb-4">
        <div className="flex items-center space-x-2">
          <h3 className="font-semibold text-slate-800">Filters</h3>
          {hasActiveFilters && (
            <span className="text-xs bg-blue-100 text-blue-600 px-2 py-1 rounded-full font-medium">
              {activeFilterCount} active
            </span>
          )}
        </div>
        <button
          onClick={onToggle}
          className="p-1.5 hover:bg-slate-100 rounded-lg transition-colors duration-200 flex items-center justify-center"
        >
          <X className="w-4 h-4 text-slate-500 hover:text-slate-700" />
        </button>
      </div>
      
      <div className="space-y-4">
        {hasActiveFilters && (
          <div className="flex items-center justify-between p-3 bg-blue-50 rounded-lg border border-blue-200">
            <div className="flex items-center space-x-2">
              <CheckCircle className="w-4 h-4 text-blue-600" />
              <span className="text-sm text-blue-700 font-medium">Filters Applied</span>
            </div>
            <button
              onClick={() => {
                onFilterChange('model', '');
              }}
              className="text-xs text-blue-600 hover:text-blue-800 font-medium"
            >
              Clear All
            </button>
          </div>
        )}
        
        <div>
          <label className="block text-sm font-medium text-slate-700 mb-2">
            Model Type
          </label>
          <select
            value={filters.model}
            onChange={(e) => handleFilterChange('model', e.target.value)}
            className={`w-full px-3 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent ${
              filters.model !== '' 
                ? 'border-blue-300 bg-blue-50' 
                : 'border-slate-200'
            }`}
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