import React, { useState, useMemo } from 'react';
import { ChevronUp, ChevronDown, BarChart3, Settings, Cpu } from 'lucide-react';

interface PointItem {
  point: {
    model: number;
    name: string;
    description: string;
  };
  models: number[];
  units: string[];
}

interface PointsTableProps {
  points: PointItem[];
  onPointClick: (modelId: number, pointId: string) => void;
}

type SortField = 'name' | 'description' | 'models' | 'units';
type SortDirection = 'asc' | 'desc';

interface ModelTableProps {
  modelId: number;
  modelName: string;
  modelDescription: string;
  points: PointItem[];
  onPointClick: (modelId: number, pointId: string) => void;
  isExpanded: boolean;
  onToggle: () => void;
}

const ModelTable: React.FC<ModelTableProps> = ({ 
  modelId, 
  modelName, 
  modelDescription,
  points, 
  onPointClick, 
  isExpanded, 
  onToggle 
}) => {
  const [sortField, setSortField] = useState<SortField>('name');
  const [sortDirection, setSortDirection] = useState<SortDirection>('asc');

  const handleSort = (field: SortField) => {
    if (sortField === field) {
      setSortDirection(sortDirection === 'asc' ? 'desc' : 'asc');
    } else {
      setSortField(field);
      setSortDirection('asc');
    }
  };

  const sortedPoints = useMemo(() => {
    return [...points].sort((a, b) => {
      let aValue: string | number;
      let bValue: string | number;

      switch (sortField) {
        case 'name':
          aValue = a.point.name.toLowerCase();
          bValue = b.point.name.toLowerCase();
          break;
        case 'description':
          aValue = a.point.description.toLowerCase();
          bValue = b.point.description.toLowerCase();
          break;
        case 'models':
          aValue = a.models.length;
          bValue = b.models.length;
          break;
        case 'units':
          aValue = a.units.length;
          bValue = b.units.length;
          break;
        default:
          return 0;
      }

      if (aValue < bValue) return sortDirection === 'asc' ? -1 : 1;
      if (aValue > bValue) return sortDirection === 'asc' ? 1 : -1;
      return 0;
    });
  }, [points, sortField, sortDirection]);

  const SortIcon = ({ field }: { field: SortField }) => {
    if (sortField !== field) {
      return <div className="w-4 h-4" />; // Placeholder for alignment
    }
    return sortDirection === 'asc' ? 
      <ChevronUp className="w-4 h-4 text-blue-600" /> : 
      <ChevronDown className="w-4 h-4 text-blue-600" />;
  };

  return (
    <div className="bg-white border border-slate-200 rounded-lg shadow-sm overflow-hidden">
      {/* Model Header */}
      <div 
        className="flex items-center justify-between p-4 bg-slate-50 border-b border-slate-200 cursor-pointer hover:bg-slate-100 transition-colors duration-200"
        onClick={onToggle}
      >
        <div className="flex items-center space-x-3">
          <div className="w-10 h-10 bg-blue-100 rounded-lg flex items-center justify-center">
            <Cpu className="w-5 h-5 text-blue-600" />
          </div>
          <div>
            <h3 className="text-lg font-semibold text-slate-800">{modelName}</h3>
            <p className="text-sm text-slate-500">{modelDescription}</p>
            <p className="text-xs text-slate-400 mt-1">Model {modelId} • {points.length} points</p>
          </div>
        </div>
        <div className="flex items-center space-x-2">
          <span className="text-xs bg-blue-100 text-blue-600 px-3 py-1 rounded-full font-medium">
            {points.length} points
          </span>
          {isExpanded ? (
            <ChevronUp className="w-5 h-5 text-slate-400" />
          ) : (
            <ChevronDown className="w-5 h-5 text-slate-400" />
          )}
        </div>
      </div>

      {/* Table Content */}
      {isExpanded && (
        <div className="overflow-x-auto">
          <table className="w-full">
            <thead className="bg-slate-50 border-b border-slate-200">
              <tr>
                <th className="px-6 py-3 text-left">
                  <button
                    onClick={() => handleSort('name')}
                    className="flex items-center space-x-2 text-sm font-semibold text-slate-700 hover:text-slate-900 transition-colors duration-200"
                  >
                    <BarChart3 className="w-4 h-4" />
                    <span>Point Name</span>
                    <SortIcon field="name" />
                  </button>
                </th>
                <th className="px-6 py-3 text-left">
                  <button
                    onClick={() => handleSort('description')}
                    className="flex items-center space-x-2 text-sm font-semibold text-slate-700 hover:text-slate-900 transition-colors duration-200"
                  >
                    <span>Description</span>
                    <SortIcon field="description" />
                  </button>
                </th>
                <th className="px-6 py-3 text-left">
                  <button
                    onClick={() => handleSort('units')}
                    className="flex items-center space-x-2 text-sm font-semibold text-slate-700 hover:text-slate-900 transition-colors duration-200"
                  >
                    <span>Units</span>
                    <SortIcon field="units" />
                  </button>
                </th>
                <th className="px-6 py-3 text-center">
                  <span className="text-sm font-semibold text-slate-700">Actions</span>
                </th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-200">
              {sortedPoints.map((item, index) => (
                <tr 
                  key={`${item.point.name}-${index}`}
                  className="hover:bg-slate-50 transition-colors duration-200"
                >
                  <td className="px-6 py-3">
                    <div className="font-medium text-slate-800 text-sm">
                      {item.point.name}
                    </div>
                  </td>
                  <td className="px-6 py-3">
                    <div className="text-sm text-slate-600 max-w-md">
                      {item.point.description}
                    </div>
                  </td>
                  <td className="px-6 py-3">
                    <div className="flex flex-wrap gap-1">
                      {item.units.slice(0, 3).map(unit => (
                        <span 
                          key={unit}
                          className="inline-flex items-center px-2 py-1 rounded-full text-xs bg-green-100 text-green-600"
                        >
                          {unit}
                        </span>
                      ))}
                      {item.units.length > 3 && (
                        <span className="inline-flex items-center px-2 py-1 rounded-full text-xs bg-slate-100 text-slate-600">
                          +{item.units.length - 3} more
                        </span>
                      )}
                    </div>
                  </td>
                  <td className="px-6 py-3 text-center">
                    <button
                      onClick={() => onPointClick(item.point.model, item.point.name)}
                      className="inline-flex items-center space-x-1 px-3 py-1.5 text-sm bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors duration-200"
                    >
                      <Settings className="w-4 h-4" />
                      <span>Configure</span>
                    </button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
};

interface PointsTableProps {
  points: PointItem[];
  onPointClick: (modelId: number, pointId: string) => void;
  data?: any; // Add data prop to access model information
}

export const PointsTable: React.FC<PointsTableProps> = ({ points, onPointClick, data }) => {
  const [expandedModels, setExpandedModels] = useState<Set<number>>(new Set());

  const toggleModel = (modelId: number) => {
    const newExpanded = new Set(expandedModels);
    if (newExpanded.has(modelId)) {
      newExpanded.delete(modelId);
    } else {
      newExpanded.add(modelId);
    }
    setExpandedModels(newExpanded);
  };

  const categorizedPoints = useMemo(() => {
    const categorized = new Map<number, { modelName: string; modelDescription: string; points: typeof points }>();
    
    points.forEach(item => {
      item.models.forEach(modelId => {
        if (!categorized.has(modelId)) {
          // Find the model name and description from the original data
          let modelName = `Model ${modelId}`;
          let modelDescription = 'No description available';
          
          if (data) {
            for (const unit of data.units) {
              const model = unit.models.find(m => m.model === modelId);
              if (model) {
                modelName = model.name;
                modelDescription = model.description;
                break;
              }
            }
          }
          
          categorized.set(modelId, { modelName, modelDescription, points: [] });
        }
        
        const category = categorized.get(modelId)!;
        if (!category.points.some(p => p.point.name === item.point.name)) {
          category.points.push(item);
        }
      });
    });
    
    // Sort categories by model ID
    return new Map([...categorized.entries()].sort(([a], [b]) => a - b));
  }, [points, data]);

  if (categorizedPoints.size === 0) {
    return (
      <div className="bg-white border border-slate-200 rounded-lg shadow-sm p-12 text-center">
        <BarChart3 className="w-12 h-12 text-slate-300 mx-auto mb-4" />
        <h3 className="text-lg font-medium text-slate-500 mb-2">No points found</h3>
        <p className="text-slate-400">
          Try adjusting your search or filter criteria
        </p>
      </div>
    );
  }

  return (
    <div className="space-y-4">
      {Array.from(categorizedPoints.entries()).map(([modelId, category]) => (
        <ModelTable
          key={modelId}
          modelId={modelId}
          modelName={category.modelName}
          modelDescription={category.modelDescription}
          points={category.points}
          onPointClick={onPointClick}
          isExpanded={expandedModels.has(modelId)}
          onToggle={() => toggleModel(modelId)}
        />
      ))}
    </div>
  );
};