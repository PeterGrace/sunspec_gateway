import React, { useState } from 'react';
import { Plus, Trash2, ChevronDown, ChevronRight, Edit2, Check, X } from 'lucide-react';
import { GatewayConfig, PointConfig } from '../types/api';

interface ModelSettingsProps {
    config: GatewayConfig;
    updateConfig: (newConfig: GatewayConfig) => void;
}

export const ModelSettings: React.FC<ModelSettingsProps> = ({ config, updateConfig }) => {
    const [expandedModels, setExpandedModels] = useState<Record<string, boolean>>({});
    const [editingPoint, setEditingPoint] = useState<{ modelId: string, index: number } | null>(null);
    const [tempPoint, setTempPoint] = useState<PointConfig | null>(null);

    const toggleModel = (modelId: string) => {
        setExpandedModels(prev => ({ ...prev, [modelId]: !prev[modelId] }));
    };

    const addModel = () => {
        const modelId = prompt("Enter Model ID (e.g., '101', '103', '64110'):");
        if (modelId && !config.models[modelId]) {
            updateConfig({
                ...config,
                models: {
                    ...config.models,
                    [modelId]: []
                }
            });
            setExpandedModels(prev => ({ ...prev, [modelId]: true }));
        }
    };

    const removeModel = (modelId: string) => {
        if (confirm(`Are you sure you want to delete Model ${modelId} configuration?`)) {
            const newModels = { ...config.models };
            delete newModels[modelId];
            updateConfig({ ...config, models: newModels });
        }
    };

    const addPoint = (modelId: string) => {
        const newPoint: PointConfig = {
            interval: 60,
            point: "New Point" // Default
        };
        const newModels = { ...config.models };
        newModels[modelId] = [...newModels[modelId], newPoint];
        updateConfig({ ...config, models: newModels });

        // Immediately edit the new point
        setEditingPoint({ modelId, index: newModels[modelId].length - 1 });
        setTempPoint(newPoint);
    };

    const removePoint = (modelId: string, index: number) => {
        const newModels = { ...config.models };
        newModels[modelId] = [...newModels[modelId]];
        newModels[modelId].splice(index, 1);
        updateConfig({ ...config, models: newModels });
    };

    const startEditPoint = (modelId: string, index: number, point: PointConfig) => {
        setEditingPoint({ modelId, index });
        setTempPoint({ ...point });
    };

    const savePoint = () => {
        if (editingPoint && tempPoint) {
            const newModels = { ...config.models };
            newModels[editingPoint.modelId][editingPoint.index] = tempPoint;
            updateConfig({ ...config, models: newModels });
            setEditingPoint(null);
            setTempPoint(null);
        }
    };

    const cancelEdit = () => {
        setEditingPoint(null);
        setTempPoint(null);
    };

    const updateTempPoint = (field: keyof PointConfig, value: any) => {
        if (tempPoint) {
            setTempPoint({ ...tempPoint, [field]: value });
        }
    };

    return (
        <div className="bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-200 dark:border-slate-700 p-6">
            <div className="flex items-center justify-between mb-4">
                <h2 className="text-lg font-semibold text-slate-800 dark:text-white">Model Configuration</h2>
                <button
                    onClick={addModel}
                    className="text-sm flex items-center space-x-1 text-blue-600 dark:text-blue-400 hover:text-blue-700 font-medium"
                >
                    <Plus className="w-4 h-4" />
                    <span>Add Model Profile</span>
                </button>
            </div>

            <div className="space-y-4">
                {Object.entries(config.models).map(([modelId, points]) => (
                    <div key={modelId} className="border border-slate-200 dark:border-slate-700 rounded-lg overflow-hidden">
                        <div
                            className="flex items-center justify-between p-3 bg-slate-50 dark:bg-slate-900/50 cursor-pointer hover:bg-slate-100 dark:hover:bg-slate-900 transition-colors"
                            onClick={() => toggleModel(modelId)}
                        >
                            <div className="flex items-center space-x-2">
                                {expandedModels[modelId] ? <ChevronDown className="w-4 h-4" /> : <ChevronRight className="w-4 h-4" />}
                                <span className="font-medium text-slate-700 dark:text-slate-200">Model {modelId}</span>
                                <span className="text-xs text-slate-500 bg-slate-200 dark:bg-slate-800 px-2 py-0.5 rounded-full">
                                    {points.length} points
                                </span>
                            </div>
                            <div className="flex items-center space-x-2" onClick={e => e.stopPropagation()}>
                                <button
                                    onClick={() => addPoint(modelId)}
                                    className="p-1.5 text-blue-600 hover:bg-blue-100 dark:hover:bg-blue-900/30 rounded"
                                    title="Add Point"
                                >
                                    <Plus className="w-4 h-4" />
                                </button>
                                <button
                                    onClick={() => removeModel(modelId)}
                                    className="p-1.5 text-red-500 hover:bg-red-100 dark:hover:bg-red-900/30 rounded"
                                    title="Delete Model Profile"
                                >
                                    <Trash2 className="w-4 h-4" />
                                </button>
                            </div>
                        </div>

                        {expandedModels[modelId] && (
                            <div className="p-3 bg-white dark:bg-slate-900 border-t border-slate-200 dark:border-slate-700">
                                {points.length === 0 ? (
                                    <div className="text-center py-4 text-slate-400 text-sm">No points configured for this model</div>
                                ) : (
                                    <div className="space-y-2">
                                        {points.map((point, idx) => (
                                            <div key={idx} className="bg-slate-50 dark:bg-slate-800/50 rounded p-3 border border-slate-100 dark:border-slate-800">
                                                {editingPoint?.modelId === modelId && editingPoint?.index === idx && tempPoint ? (
                                                    // Edit Interface
                                                    <div className="space-y-3">
                                                        <div className="grid grid-cols-2 gap-3">
                                                            <div>
                                                                <label className="block text-xs text-slate-500 dark:text-slate-400 mb-1">Point ID</label>
                                                                <input
                                                                    className="w-full text-sm px-2 py-1 rounded border border-slate-300 dark:border-slate-600 bg-white dark:bg-slate-900 text-slate-900 dark:text-white focus:ring-2 focus:ring-blue-500 outline-none"
                                                                    value={tempPoint.point || ''}
                                                                    onChange={e => updateTempPoint('point', e.target.value)}
                                                                    placeholder="e.g. W"
                                                                />
                                                            </div>
                                                            <div>
                                                                <label className="block text-xs text-slate-500 dark:text-slate-400 mb-1">Catalog Ref (Alt)</label>
                                                                <input
                                                                    className="w-full text-sm px-2 py-1 rounded border border-slate-300 dark:border-slate-600 bg-white dark:bg-slate-900 text-slate-900 dark:text-white focus:ring-2 focus:ring-blue-500 outline-none"
                                                                    value={tempPoint.catalog_ref || ''}
                                                                    onChange={e => updateTempPoint('catalog_ref', e.target.value)}
                                                                    placeholder="e.g. [1..10]"
                                                                />
                                                            </div>
                                                            <div>
                                                                <label className="block text-xs text-slate-500 dark:text-slate-400 mb-1">Display Name</label>
                                                                <input
                                                                    className="w-full text-sm px-2 py-1 rounded border border-slate-300 dark:border-slate-600 bg-white dark:bg-slate-900 text-slate-900 dark:text-white focus:ring-2 focus:ring-blue-500 outline-none"
                                                                    value={tempPoint.display_name || ''}
                                                                    onChange={e => updateTempPoint('display_name', e.target.value)}
                                                                />
                                                            </div>
                                                            <div>
                                                                <label className="block text-xs text-slate-500 dark:text-slate-400 mb-1">Interval (s)</label>
                                                                <input
                                                                    type="number"
                                                                    className="w-full text-sm px-2 py-1 rounded border border-slate-300 dark:border-slate-600 bg-white dark:bg-slate-900 text-slate-900 dark:text-white focus:ring-2 focus:ring-blue-500 outline-none"
                                                                    value={tempPoint.interval}
                                                                    onChange={e => updateTempPoint('interval', parseInt(e.target.value))}
                                                                />
                                                            </div>
                                                            <div>
                                                                <label className="block text-xs text-slate-500 dark:text-slate-400 mb-1">Value Min</label>
                                                                <input
                                                                    type="number"
                                                                    className="w-full text-sm px-2 py-1 rounded border border-slate-300 dark:border-slate-600 bg-white dark:bg-slate-900 text-slate-900 dark:text-white focus:ring-2 focus:ring-blue-500 outline-none"
                                                                    value={tempPoint.value_min || ''}
                                                                    onChange={e => updateTempPoint('value_min', parseFloat(e.target.value))}
                                                                />
                                                            </div>
                                                            <div>
                                                                <label className="block text-xs text-slate-500 mb-1">Value Max</label>
                                                                <input
                                                                    type="number"
                                                                    className="w-full text-sm px-2 py-1 rounded border dark:bg-slate-900 dark:border-slate-700"
                                                                    value={tempPoint.value_max || ''}
                                                                    onChange={e => updateTempPoint('value_max', parseFloat(e.target.value))}
                                                                />
                                                            </div>
                                                        </div>

                                                        <div className="flex justify-end space-x-2 mt-2">
                                                            <button
                                                                onClick={cancelEdit}
                                                                className="px-2 py-1 text-xs text-slate-600 dark:text-slate-400 hover:bg-slate-200 rounded flex items-center"
                                                            >
                                                                <X className="w-3 h-3 mr-1" /> Cancel
                                                            </button>
                                                            <button
                                                                onClick={savePoint}
                                                                className="px-2 py-1 text-xs bg-green-600 text-white hover:bg-green-700 rounded flex items-center"
                                                            >
                                                                <Check className="w-3 h-3 mr-1" /> Save
                                                            </button>
                                                        </div>
                                                    </div>
                                                ) : (
                                                    // View Interface
                                                    <div className="flex items-center justify-between">
                                                        <div className="flex-1">
                                                            <div className="flex items-center space-x-2">
                                                                <span className="font-semibold text-sm text-slate-700 dark:text-slate-200">
                                                                    {point.point || point.catalog_ref || <span className="text-red-500 italic">Undefined</span>}
                                                                </span>
                                                                {point.display_name && <span className="text-xs text-slate-500">({point.display_name})</span>}
                                                            </div>
                                                            <div className="text-xs text-slate-500 mt-1 flex space-x-4">
                                                                <span>Interval: {point.interval}s</span>
                                                                {point.device_class && <span>Class: {point.device_class}</span>}
                                                            </div>
                                                        </div>
                                                        <div className="flex items-center space-x-1">
                                                            <button
                                                                onClick={() => startEditPoint(modelId, idx, point)}
                                                                className="p-1.5 text-slate-400 hover:text-blue-500 hover:bg-blue-50 dark:hover:bg-blue-900/20 rounded"
                                                            >
                                                                <Edit2 className="w-3.5 h-3.5" />
                                                            </button>
                                                            <button
                                                                onClick={() => removePoint(modelId, idx)}
                                                                className="p-1.5 text-slate-400 hover:text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20 rounded"
                                                            >
                                                                <Trash2 className="w-3.5 h-3.5" />
                                                            </button>
                                                        </div>
                                                    </div>
                                                )}
                                            </div>
                                        ))}
                                    </div>
                                )}
                            </div>
                        )}
                    </div>
                ))}
                {Object.keys(config.models).length === 0 && (
                    <div className="text-center py-8 border-2 border-dashed border-slate-200 dark:border-slate-700 rounded-lg">
                        <p className="text-slate-500">No model profiles defined</p>
                        <button onClick={addModel} className="mt-2 text-sm text-blue-600 hover:underline">Create your first model profile</button>
                    </div>
                )}
            </div>
        </div>
    );
};
