import React, { useState, useEffect, useMemo } from 'react';
import { Zap, Server, Cpu, BarChart3 } from 'lucide-react';
import { ApiService } from './services/api';
import { UnitList } from './types/api';
import { SearchBar } from './components/SearchBar';
import { LoadingSpinner } from './components/LoadingSpinner';
import { ErrorMessage } from './components/ErrorMessage';
import { UnitCard } from './components/UnitCard';
import { StatsCard } from './components/StatsCard';
import { FilterPanel } from './components/FilterPanel';
import { YamlModal } from './components/YamlModal';

function App() {
  const [data, setData] = useState<UnitList | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [searchTerm, setSearchTerm] = useState('');
  const [showFilters, setShowFilters] = useState(false);
  const [viewMode, setViewMode] = useState<'units' | 'points'>('units');
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

    const filtered = data.units.filter(unit => {
      // Search filter
      const matchesSearch = searchTerm === '' || 
        unit.unit.toLowerCase().includes(searchTerm.toLowerCase()) ||
        unit.models.some(model => 
          model.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
          model.description.toLowerCase().includes(searchTerm.toLowerCase()) ||
          model.points.some(point => 
            point.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
            point.description.toLowerCase().includes(searchTerm.toLowerCase())
          )
        );

      // Model filter
      const matchesModel = filters.model === '' || 
        unit.models.some(model => model.model.toString() === filters.model);

      return matchesSearch && matchesModel;
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
    <div className="min-h-screen bg-slate-50">
      {/* Header */}
      <div className="bg-white shadow-sm border-b border-slate-200">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex items-center justify-between h-16">
            <div className="flex items-center space-x-3">
              <div className="w-10 h-10 bg-blue-600 rounded-lg flex items-center justify-center">
                <Zap className="w-6 h-6 text-white" />
              </div>
              <div>
                <h1 className="text-xl font-bold text-slate-800">SunSpec Gateway</h1>
                <p className="text-sm text-slate-500">Equipment Monitoring Dashboard</p>
              </div>
            </div>
            <div className="flex items-center space-x-4">
              <div className="w-80">
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
          </div>
        </div>
      </div>

      {/* Main Content */}
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
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
            </div>
            <div className="flex items-center space-x-2">
              <button
                onClick={() => setViewMode('units')}
                className={`px-4 py-2 rounded-lg text-sm font-medium transition-colors duration-200 ${
                  viewMode === 'units'
                    ? 'bg-blue-600 text-white'
                    : 'bg-white text-slate-600 border border-slate-200 hover:bg-slate-50'
                }`}
              >
                Units View
              </button>
              <button
                onClick={() => setViewMode('points')}
                className={`px-4 py-2 rounded-lg text-sm font-medium transition-colors duration-200 ${
                  viewMode === 'points'
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
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
              {filteredDeduplicatedPoints.map((item, index) => (
                <div key={`${item.point.name}-${index}`} className="bg-white border border-slate-200 rounded-lg p-4 shadow-sm hover:shadow-md transition-all duration-200 cursor-pointer group">
                  <div className="flex items-start space-x-3">
                    <div className="w-10 h-10 bg-purple-100 rounded-lg flex items-center justify-center flex-shrink-0">
                      <BarChart3 className="w-5 h-5 text-purple-600" />
                    </div>
                    <div className="flex-1 min-w-0">
                      <h4 className="font-semibold text-slate-800 truncate">{item.point.name}</h4>
                      <p className="text-sm text-slate-600 mt-1 line-clamp-2">{item.point.description}</p>
                      <div className="mt-3 flex flex-wrap gap-2">
                        <span className="text-xs bg-blue-100 text-blue-600 px-2 py-1 rounded-full">
                          {item.models.length} model{item.models.length !== 1 ? 's' : ''}
                        </span>
                        <span className="text-xs bg-green-100 text-green-600 px-2 py-1 rounded-full">
                          {item.units.length} unit{item.units.length !== 1 ? 's' : ''}
                        </span>
                        <button
                          onClick={(e) => {
                            e.stopPropagation();
                            handlePointClick(item.point.model, item.point.name);
                          }}
                          className="text-xs bg-blue-600 text-white px-2 py-1 rounded-full opacity-0 group-hover:opacity-100 transition-opacity duration-200 hover:bg-blue-700"
                        >
                          Configure
                        </button>
                      </div>
                      <div className="mt-2 text-xs text-slate-500">
                        Models: {item.models.join(', ')}
                      </div>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          ) : (
            <div className="text-center py-12">
              <Server className="w-12 h-12 text-slate-300 mx-auto mb-4" />
              <h3 className="text-lg font-medium text-slate-500 mb-2">No units found</h3>
              <p className="text-slate-400">
                {searchTerm || filters.model || filters.connectionStatus 
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
}

export default App;