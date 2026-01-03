import React, { useState, useEffect, useMemo } from 'react';
import { Zap, Server, Cpu, BarChart3 } from 'lucide-react';
import { ApiService } from '../services/api';
import { UnitList } from '../types/api';
import { SearchBar } from '../components/SearchBar';
import { LoadingSpinner } from '../components/LoadingSpinner';
import { ErrorMessage } from '../components/ErrorMessage';
import { UnitCard } from '../components/UnitCard';
import { StatsCard } from '../components/StatsCard';
import { FilterPanel } from '../components/FilterPanel';
import { YamlModal } from '../components/YamlModal';
import { PointsTable } from '../components/PointsTable';

export const PointsBrowser: React.FC = () => {
  const [data, setData] = useState<UnitList | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [searchTerm, setSearchTerm] = useState('');
  const [showFilters, setShowFilters] = useState(false);
  const [viewMode, setViewMode] = useState<'units' | 'points'>('points');
  const [selectedPoint, setSelectedPoint] = useState<{ modelId: number; pointId: string } | null>(null);
  const [filters, setFilters] = useState({
    model: ''
  });

  const fetchData = async () => {
    try {
      setLoading(true);
      setError(null);
      const result = await ApiService.getAllPoints();
      setData(result);
    } catch (err) {
      setError('Failed to fetch points data. Please try again.');
      console.error(err);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchData();
  }, []);

  const handleFilterChange = (key: string, value: string) => {
    setFilters(prev => ({ ...prev, [key]: value }));
  };

  const handlePointClick = (modelId: number, pointId: string) => {
    setSelectedPoint({ modelId, pointId });
  };

  const filteredData = useMemo(() => {
    if (!data) return null;

    const filtered = data.units.map(unit => {
      // First filter models within the unit
      let filteredModels = unit.models;

      if (filters.model !== '') {
        filteredModels = unit.models.filter(model =>
          model.model.toString() === filters.model
        );
      }

      // Apply search filter to the filtered models
      if (searchTerm !== '') {
        filteredModels = filteredModels.filter(model =>
          model.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
          model.description.toLowerCase().includes(searchTerm.toLowerCase()) ||
          model.points.some(point =>
            point.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
            point.description.toLowerCase().includes(searchTerm.toLowerCase())
          )
        );
      }

      return {
        ...unit,
        models: filteredModels
      };
    }).filter(unit => {
      // Search filter
      const matchesSearch = searchTerm === '' ||
        unit.unit.toLowerCase().includes(searchTerm.toLowerCase()) ||
        unit.models.length > 0; // Unit matches if it has any models after filtering

      return matchesSearch;
    });

    return { units: filtered };
  }, [data, searchTerm, filters]);

  const stats = useMemo(() => {
    if (!data) return { totalUnits: 0, totalModels: 0, totalPoints: 0 };

    const totalUnits = data.units.length;
    const totalModels = data.units.reduce((sum, unit) => sum + unit.models.length, 0);
    const totalPoints = data.units.reduce((sum, unit) =>
      sum + unit.models.reduce((modelSum, model) => modelSum + model.points.length, 0), 0
    );

    return { totalUnits, totalModels, totalPoints };
  }, [data]);

  const deduplicatedPoints = useMemo(() => {
    if (!data) return [];

    const pointsMap = new Map<string, { point: any; models: number[]; units: string[] }>();

    data.units.forEach(unit => {
      unit.models.forEach(model => {
        model.points.forEach(point => {
          const key = `${point.name}-${point.description}`;

          if (pointsMap.has(key)) {
            const existing = pointsMap.get(key)!;
            if (!existing.models.includes(point.model)) {
              existing.models.push(point.model);
            }
            if (!existing.units.includes(unit.unit)) {
              existing.units.push(unit.unit);
            }
          } else {
            pointsMap.set(key, {
              point,
              models: [point.model],
              units: [unit.unit]
            });
          }
        });
      });
    });

    return Array.from(pointsMap.values()).sort((a, b) =>
      a.point.name.localeCompare(b.point.name)
    );
  }, [data]);

  const filteredDeduplicatedPoints = useMemo(() => {
    if (!deduplicatedPoints) return [];

    return deduplicatedPoints.filter(item => {
      // Search filter
      const matchesSearch = searchTerm === '' ||
        item.point.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
        item.point.description.toLowerCase().includes(searchTerm.toLowerCase());

      // Model filter
      const matchesModel = filters.model === '' ||
        item.models.includes(parseInt(filters.model));

      return matchesSearch && matchesModel;
    });
  }, [deduplicatedPoints, searchTerm, filters]);

  if (loading) {
    return (
      <div className="min-h-screen bg-slate-50">
        <LoadingSpinner />
      </div>
    );
  }

  if (error) {
    return (
      <div className="min-h-screen bg-slate-50">
        <ErrorMessage message={error} onRetry={fetchData} />
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-slate-50 dark:bg-slate-900 transition-colors duration-300">
      {/* Main Content */}
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">

        {/* Toolbar */}
        <div className="flex items-center justify-between mb-8 flex-wrap gap-4">
          <div className="flex-1 w-full md:w-auto md:max-w-md">
            <SearchBar
              value={searchTerm}
              onChange={setSearchTerm}
              placeholder="Search units, models, or points..."
            />
          </div>
          <FilterPanel
            isOpen={showFilters}
            onToggle={() => setShowFilters(!showFilters)}
            filters={filters}
            onFilterChange={handleFilterChange}
            data={data}
          />
        </div>
        {/* Stats Cards */}
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-8">
          <StatsCard
            title="Total Units"
            value={stats.totalUnits}
            icon={Server}
            color="blue"
            description="Active monitoring units"
          />
          <StatsCard
            title="Total Models"
            value={stats.totalModels}
            icon={Cpu}
            color="green"
            description="Device models configured"
          />
          <StatsCard
            title="Total Points"
            value={stats.totalPoints}
            icon={BarChart3}
            color="purple"
            description="Data points available"
          />
        </div>

        {/* Units List */}
        <div className="space-y-6">
          <div className="flex items-center justify-between">
            <div className="flex items-center space-x-4">
              <h2 className="text-lg font-semibold text-slate-800">
                {viewMode === 'units' ? 'Equipment Units' : 'All Available Points'}
                {viewMode === 'units' && filteredData && filteredData.units.length !== stats.totalUnits && (
                  <span className="ml-2 text-sm font-normal text-slate-500">
                    ({filteredData.units.length} of {stats.totalUnits})
                  </span>
                )}
                {viewMode === 'points' && (
                  <span className="ml-2 text-sm font-normal text-slate-500">
                    ({filteredDeduplicatedPoints.length} unique points)
                  </span>
                )}
              </h2>
              {/* Active Filter Indicators */}
              {(searchTerm || filters.model) && (
                <div className="flex items-center space-x-2">
                  {searchTerm && (
                    <span className="inline-flex items-center px-3 py-1 rounded-full text-xs bg-green-100 text-green-700 border border-green-200">
                      <span className="mr-1">🔍</span>
                      Search: "{searchTerm}"
                      <button
                        onClick={() => setSearchTerm('')}
                        className="ml-2 text-green-600 hover:text-green-800"
                      >
                        ×
                      </button>
                    </span>
                  )}
                  {filters.model && (
                    <span className="inline-flex items-center px-3 py-1 rounded-full text-xs bg-blue-100 text-blue-700 border border-blue-200">
                      <span className="mr-1">🏷️</span>
                      Model: {(() => {
                        if (data) {
                          for (const unit of data.units) {
                            const model = unit.models.find(m => m.model.toString() === filters.model);
                            if (model) return model.name;
                          }
                        }
                        return `Model ${filters.model}`;
                      })()}
                      <button
                        onClick={() => handleFilterChange('model', '')}
                        className="ml-2 text-blue-600 hover:text-blue-800"
                      >
                        ×
                      </button>
                    </span>
                  )}
                </div>
              )}
            </div>
            <div className="flex items-center space-x-2">
              <button
                onClick={() => setViewMode('units')}
                className={`px-4 py-2 rounded-lg text-sm font-medium transition-colors duration-200 ${viewMode === 'units'
                  ? 'bg-blue-600 text-white'
                  : 'bg-white text-slate-600 border border-slate-200 hover:bg-slate-50'
                  }`}
              >
                Units View
              </button>
              <button
                onClick={() => setViewMode('points')}
                className={`px-4 py-2 rounded-lg text-sm font-medium transition-colors duration-200 ${viewMode === 'points'
                  ? 'bg-blue-600 text-white'
                  : 'bg-white text-slate-600 border border-slate-200 hover:bg-slate-50'
                  }`}
              >
                Points View
              </button>
            </div>
          </div>

          {viewMode === 'units' && filteredData && filteredData.units.length > 0 ? (
            <div className="space-y-4">
              {filteredData.units.map((unit, index) => (
                <UnitCard
                  key={`${unit.unit}-${index}`}
                  unit={unit}
                  onPointClick={handlePointClick}
                />
              ))}
            </div>
          ) : viewMode === 'points' && filteredDeduplicatedPoints.length > 0 ? (
            <PointsTable
              points={filteredDeduplicatedPoints}
              onPointClick={handlePointClick}
              data={data}
            />
          ) : (
            <div className="text-center py-12">
              <Server className="w-12 h-12 text-slate-300 mx-auto mb-4" />
              <h3 className="text-lg font-medium text-slate-500 mb-2">No units found</h3>
              <p className="text-slate-400">
                {searchTerm || filters.model
                  ? 'Try adjusting your search or filter criteria'
                  : viewMode === 'units'
                    ? 'No equipment units are currently available'
                    : 'No points are currently available'
                }
              </p>
            </div>
          )}
        </div>
      </div>

      {selectedPoint && (
        <YamlModal
          isOpen={!!selectedPoint}
          onClose={() => setSelectedPoint(null)}
          modelId={selectedPoint.modelId}
          pointId={selectedPoint.pointId}
        />
      )}
    </div>
  );
};

export default PointsBrowser;