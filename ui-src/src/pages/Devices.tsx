import React, { useState, useEffect } from 'react';
import { Server, Database, Code, Activity, RefreshCw, Zap, Thermometer, Gauge, Battery, Clock, Info } from 'lucide-react';
import { PayloadValueType } from '../types/api';

// Point name lookup for human-readable labels
const POINT_LABELS: Record<string, { label: string; unit?: string; category?: string }> = {
    // Inverter Status
    'St': { label: 'Status', category: 'status' },
    'Evt1': { label: 'Event 1', category: 'status' },
    'Evt2': { label: 'Event 2', category: 'status' },
    'Ev': { label: 'Event Code', category: 'status' },
    'SysMd': { label: 'System Mode', category: 'status' },

    // Power & Energy
    'W': { label: 'AC Power', unit: 'W', category: 'power' },
    'DCW': { label: 'DC Power', unit: 'W', category: 'power' },
    'P': { label: 'Power', unit: 'W', category: 'power' },
    'WH': { label: 'Energy (Lifetime)', unit: 'Wh', category: 'energy' },
    'Whin': { label: 'Energy In', unit: 'Wh', category: 'energy' },
    'WhIn': { label: 'Energy In', unit: 'Wh', category: 'energy' },
    'WhOut': { label: 'Energy Out', unit: 'Wh', category: 'energy' },
    'Whx': { label: 'Energy Export', unit: 'Wh', category: 'energy' },
    'E': { label: 'Total Energy', unit: 'Wh', category: 'energy' },
    'CTPow': { label: 'CT Power', unit: 'W', category: 'power' },

    // Voltage
    'V': { label: 'Voltage', unit: 'V', category: 'voltage' },
    'DCV': { label: 'DC Voltage', unit: 'V', category: 'voltage' },
    'PhVphA': { label: 'Phase A Voltage', unit: 'V', category: 'voltage' },
    'PhVphB': { label: 'Phase B Voltage', unit: 'V', category: 'voltage' },
    'PhVphC': { label: 'Phase C Voltage', unit: 'V', category: 'voltage' },
    'Vin': { label: 'Input Voltage', unit: 'V', category: 'voltage' },
    'Px1': { label: 'Power Export L1', unit: 'W', category: 'power' },
    'Px2': { label: 'Power Export L2', unit: 'W', category: 'power' },

    // Current
    'A': { label: 'Current', unit: 'A', category: 'current' },
    'DCA': { label: 'DC Current', unit: 'A', category: 'current' },
    'I': { label: 'Current', unit: 'A', category: 'current' },
    'Iin': { label: 'Input Current', unit: 'A', category: 'current' },

    // Frequency
    'Hz': { label: 'Frequency', unit: 'Hz', category: 'frequency' },

    // Temperature
    'Tmp': { label: 'Temperature', unit: '°C', category: 'temperature' },
    'TmpCab': { label: 'Cabinet Temp', unit: '°C', category: 'temperature' },
    'TmpSnk': { label: 'Heatsink Temp', unit: '°C', category: 'temperature' },

    // Battery
    'SoC': { label: 'State of Charge', unit: '%', category: 'battery' },
    'SoCRsvMax': { label: 'Max Reserve SoC', unit: '%', category: 'battery' },
    'SoCRsvMin': { label: 'Min Reserve SoC', unit: '%', category: 'battery' },
    'SoH': { label: 'State of Health', unit: '%', category: 'battery' },
    'ChaSt': { label: 'Charge State', category: 'battery' },
    'SetOp': { label: 'Battery Operation', category: 'battery' },
    'SetInvState': { label: 'Inverter State', category: 'status' },

    // PV Link / Enable
    'Ena': { label: 'Enable Status', category: 'status' },

    // Battery Modules
    'ModSoC': { label: 'State of Charge', unit: '%', category: 'battery' },
    'ModSoH': { label: 'State of Health', unit: '%', category: 'battery' },
    'ModV': { label: 'Voltage', unit: 'V', category: 'voltage' },
    'ModA': { label: 'Current', unit: 'A', category: 'current' },
    'ModTmp': { label: 'Temperature', unit: '°C', category: 'temperature' },
    'ModCellVMax': { label: 'Max Cell Voltage', unit: 'V', category: 'voltage' },
    'ModCellVMin': { label: 'Min Cell Voltage', unit: 'V', category: 'voltage' },
    'ModCellTmpMax': { label: 'Max Cell Temp', unit: '°C', category: 'temperature' },
    'ModCellTmpMin': { label: 'Min Cell Temp', unit: '°C', category: 'temperature' },
};

// Get category icon
const getCategoryIcon = (category?: string) => {
    switch (category) {
        case 'power':
        case 'energy':
            return <Zap className="w-3.5 h-3.5" />;
        case 'voltage':
        case 'current':
        case 'frequency':
            return <Gauge className="w-3.5 h-3.5" />;
        case 'temperature':
            return <Thermometer className="w-3.5 h-3.5" />;
        case 'battery':
            return <Battery className="w-3.5 h-3.5" />;
        case 'status':
            return <Activity className="w-3.5 h-3.5" />;
        default:
            return <Info className="w-3.5 h-3.5" />;
    }
};

// Get category color
const getCategoryColor = (category?: string) => {
    switch (category) {
        case 'power':
        case 'energy':
            return 'text-yellow-500 bg-yellow-500/10';
        case 'voltage':
        case 'current':
        case 'frequency':
            return 'text-blue-500 bg-blue-500/10';
        case 'temperature':
            return 'text-red-500 bg-red-500/10';
        case 'battery':
            return 'text-green-500 bg-green-500/10';
        case 'status':
            return 'text-purple-500 bg-purple-500/10';
        default:
            return 'text-slate-500 bg-slate-500/10';
    }
};

interface FullModelData {
    model_id: number;
    model_name: string;
    points: Record<string, PayloadValueType>;
}

interface FullDeviceData {
    serial_number: string;
    models: FullModelData[];
}

interface AllDevicesResponse {
    devices: FullDeviceData[];
}

export const Devices = () => {
    const [data, setData] = useState<AllDevicesResponse | null>(null);
    const [loading, setLoading] = useState(true);
    const [lastUpdated, setLastUpdated] = useState<Date>(new Date());
    const [expandedModels, setExpandedModels] = useState<Set<string>>(new Set());

    const fetchData = async () => {
        try {
            const response = await fetch('/api/v1/dashboard/all');
            if (response.ok) {
                const json = await response.json();
                setData(json);
                setLastUpdated(new Date());
            }
        } catch (error) {
            console.error("Failed to fetch all devices data:", error);
        } finally {
            setLoading(false);
        }
    };

    useEffect(() => {
        fetchData();
        const interval = setInterval(fetchData, 5000);
        return () => clearInterval(interval);
    }, []);

    const formatValue = (val: PayloadValueType, unit?: string) => {
        if (val === null || val === undefined) return '-';
        let formatted: string;
        if (typeof val === 'number') {
            if (Number.isInteger(val)) {
                formatted = val.toLocaleString();
            } else {
                formatted = val.toFixed(2);
            }
        } else if (typeof val === 'boolean') {
            formatted = val ? 'True' : 'False';
        } else {
            formatted = String(val);
        }
        return unit ? `${formatted} ${unit}` : formatted;
    };

    const getPointInfo = (pointName: string) => {
        // Check exact match first
        if (POINT_LABELS[pointName]) {
            return POINT_LABELS[pointName];
        }

        // Handle Battery Module arrays (Repeating messy path)
        // Pattern: ...lithium_ion_string_module[1].ModSoH
        const moduleMatch = pointName.match(/module\[(\d+)\]\.(\w+)$/);
        if (moduleMatch) {
            const [_, modNum, suffix] = moduleMatch;
            const suffixInfo = POINT_LABELS[suffix] || { label: suffix };
            return {
                label: `Module ${modNum} ${suffixInfo.label}`,
                unit: suffixInfo.unit,
                category: suffixInfo.category || 'battery'
            };
        }

        // Handle topic_name format: mod1_ModSoC, mod2_ModSoH, etc.
        // Pattern: mod{N}_{PointName}
        const topicNameMatch = pointName.match(/^mod(\d+)_(\w+)$/);
        if (topicNameMatch) {
            const [_, modNum, suffix] = topicNameMatch;
            const suffixInfo = POINT_LABELS[suffix] || { label: suffix };
            return {
                label: `Module ${modNum} ${suffixInfo.label}`,
                unit: suffixInfo.unit,
                category: suffixInfo.category || 'battery'
            };
        }

        return { label: pointName };
    };

    const toggleModel = (key: string) => {
        const newExpanded = new Set(expandedModels);
        if (newExpanded.has(key)) {
            newExpanded.delete(key);
        } else {
            newExpanded.add(key);
        }
        setExpandedModels(newExpanded);
    };

    if (loading && !data) {
        return (
            <div className="min-h-screen bg-slate-50 dark:bg-slate-900 flex justify-center items-center">
                <RefreshCw className="w-8 h-8 animate-spin text-blue-500" />
            </div>
        );
    }

    return (
        <div className="min-h-screen bg-slate-50 dark:bg-slate-900 pb-8">
            {/* Content */}
            <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-6 space-y-6">
                {data?.devices.map((device) => (
                    <div
                        key={device.serial_number}
                        className="bg-white/50 dark:bg-slate-800/50 backdrop-blur-md rounded-2xl shadow-sm border border-white/20 dark:border-slate-700/50 overflow-hidden"
                    >
                        {/* Device Header */}
                        <div className="bg-gradient-to-r from-slate-100 to-slate-50 dark:from-slate-800 dark:to-slate-800/50 p-4 border-b border-slate-200/50 dark:border-slate-700/50">
                            <div className="flex items-center space-x-3">
                                <div className="p-2 bg-gradient-to-br from-blue-500 to-blue-600 rounded-lg text-white shadow-sm">
                                    <Activity className="w-5 h-5" />
                                </div>
                                <div>
                                    <h2 className="text-lg font-semibold text-slate-900 dark:text-white">
                                        {device.serial_number}
                                    </h2>
                                    <div className="text-xs text-slate-500 flex items-center space-x-2">
                                        <Database className="w-3 h-3" />
                                        <span>{device.models.length} Models</span>
                                    </div>
                                </div>
                            </div>
                        </div>

                        {/* Models */}
                        <div className="p-4 space-y-4">
                            {device.models.map((model) => {
                                const modelKey = `${device.serial_number}-${model.model_id}`;
                                const isExpanded = expandedModels.has(modelKey);
                                const pointEntries = Object.entries(model.points).sort();
                                const displayPoints = isExpanded ? pointEntries : pointEntries.slice(0, 6);

                                return (
                                    <div
                                        key={model.model_id}
                                        className="bg-white dark:bg-slate-900/50 rounded-xl border border-slate-200 dark:border-slate-700/50 overflow-hidden"
                                    >
                                        {/* Model Header */}
                                        <div className="px-4 py-3 border-b border-slate-200 dark:border-slate-700/50 flex items-center justify-between bg-slate-50/50 dark:bg-slate-800/30">
                                            <div className="flex items-center space-x-2">
                                                <Code className="w-4 h-4 text-slate-400" />
                                                <span className="font-medium text-sm text-slate-700 dark:text-slate-300">
                                                    {model.model_name}
                                                </span>
                                            </div>
                                            <span className="text-xs font-mono px-2 py-1 rounded-md bg-slate-200 dark:bg-slate-700 text-slate-600 dark:text-slate-300">
                                                Model {model.model_id}
                                            </span>
                                        </div>

                                        {/* Points Grid */}
                                        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-px bg-slate-100 dark:bg-slate-700/30">
                                            {displayPoints.map(([pointName, value]) => {
                                                const info = getPointInfo(pointName);
                                                return (
                                                    <div
                                                        key={pointName}
                                                        className="bg-white dark:bg-slate-900/80 p-3 hover:bg-slate-50 dark:hover:bg-slate-800/60 transition-colors"
                                                    >
                                                        <div className="flex items-start justify-between">
                                                            <div className="flex items-start space-x-2">
                                                                <div className={`p-1 rounded ${getCategoryColor(info.category)}`}>
                                                                    {getCategoryIcon(info.category)}
                                                                </div>
                                                                <div>
                                                                    <div className="text-sm font-medium text-slate-700 dark:text-slate-300">
                                                                        {info.label}
                                                                    </div>
                                                                    <div className="text-xs text-slate-400 font-mono">
                                                                        {pointName}
                                                                    </div>
                                                                </div>
                                                            </div>
                                                            <div className="text-right">
                                                                <div className="text-sm font-semibold text-slate-900 dark:text-white">
                                                                    {formatValue(value, info.unit)}
                                                                </div>
                                                            </div>
                                                        </div>
                                                    </div>
                                                );
                                            })}
                                        </div>

                                        {/* Expand/Collapse */}
                                        {pointEntries.length > 6 && (
                                            <button
                                                onClick={() => toggleModel(modelKey)}
                                                className="w-full py-2 text-sm text-blue-600 dark:text-blue-400 hover:bg-blue-50 dark:hover:bg-blue-900/20 transition-colors"
                                            >
                                                {isExpanded ? 'Show Less' : `Show ${pointEntries.length - 6} More Points`}
                                            </button>
                                        )}
                                    </div>
                                );
                            })}
                        </div>
                    </div>
                ))}

                {data?.devices.length === 0 && (
                    <div className="text-center py-12 text-slate-500 dark:text-slate-400 bg-white/50 dark:bg-slate-800/50 rounded-xl">
                        No devices found. Ensure the gateway is connected to SunSpec units.
                    </div>
                )}
            </div>
        </div>
    );
};
