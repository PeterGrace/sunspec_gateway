import React, { useEffect, useState, useCallback } from 'react';
import { Sliders, AlertTriangle, Check, X, RefreshCw, Zap, Sun, Battery, Cpu } from 'lucide-react';
import { ApiService } from '../services/api';
import { LoadingSpinner } from '../components/LoadingSpinner';

interface ControlSymbol {
    name: string;
    value: number;
}

interface ControlPointState {
    serial_number: string;
    current_value?: { Float?: number; Int?: number; String?: string } | number | string;
    model_id: number;
    point_name: string;
    label: string;
    description: string;
    data_type: string;
    symbols?: ControlSymbol[];
    units?: string;
}

// Helper to extract numeric value from PayloadValueType
const extractValue = (value: any): number | string | null => {
    if (value === null || value === undefined) return null;
    if (typeof value === 'number') return value;
    if (typeof value === 'string') return value;
    if (typeof value === 'object') {
        if ('Float' in value) return value.Float;
        if ('Int' in value) return value.Int;
        if ('String' in value) return value.String;
    }
    return null;
};

// Group points by category
const groupPoints = (points: ControlPointState[]) => {
    const groups: { [key: string]: ControlPointState[] } = {
        'Inverter': [],
        'PV Links': [],
        'Battery Storage': [],
    };

    for (const point of points) {
        // System Mode and Inverter State both go in Inverter group
        if (point.model_id === 64200 && point.point_name === 'SysMd') {
            groups['Inverter'].push(point);
        } else if (point.model_id === 802 && point.point_name === 'SetInvState') {
            groups['Inverter'].push(point);
        } else if (point.model_id === 64251) {
            groups['PV Links'].push(point);
        } else if (point.model_id === 802) {
            // Battery-specific controls (SoC reserves, SetOp)
            groups['Battery Storage'].push(point);
        }
    }

    return groups;
};

// Confirmation Modal
interface ConfirmModalProps {
    isOpen: boolean;
    point: ControlPointState | null;
    newValue: string;
    onConfirm: () => void;
    onCancel: () => void;
    loading: boolean;
}

const ConfirmModal: React.FC<ConfirmModalProps> = ({ isOpen, point, newValue, onConfirm, onCancel, loading }) => {
    if (!isOpen || !point) return null;

    const currentVal = extractValue(point.current_value);
    const newSymbol = point.symbols?.find(s => s.value === parseInt(newValue));

    return (
        <div className="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-50">
            <div className="bg-white dark:bg-slate-800 rounded-2xl p-6 max-w-md w-full mx-4 shadow-2xl border border-white/20">
                <div className="flex items-center space-x-3 mb-4">
                    <div className="p-2 rounded-lg bg-amber-100 dark:bg-amber-900/30">
                        <AlertTriangle className="w-6 h-6 text-amber-600" />
                    </div>
                    <h3 className="text-lg font-semibold text-slate-800 dark:text-white">Confirm Write Operation</h3>
                </div>

                <div className="bg-slate-50 dark:bg-slate-900/50 rounded-xl p-4 mb-4 space-y-2">
                    <div className="flex justify-between text-sm">
                        <span className="text-slate-500">Device:</span>
                        <span className="font-medium text-slate-800 dark:text-white">{point.serial_number}</span>
                    </div>
                    <div className="flex justify-between text-sm">
                        <span className="text-slate-500">Control:</span>
                        <span className="font-medium text-slate-800 dark:text-white">{point.label}</span>
                    </div>
                    <div className="flex justify-between text-sm">
                        <span className="text-slate-500">Current:</span>
                        <span className="font-medium text-slate-800 dark:text-white">
                            {point.symbols?.find(s => s.value === currentVal)?.name || currentVal || 'N/A'}
                        </span>
                    </div>
                    <div className="flex justify-between text-sm">
                        <span className="text-slate-500">New Value:</span>
                        <span className="font-bold text-blue-600 dark:text-blue-400">
                            {newSymbol?.name || newValue}
                        </span>
                    </div>
                </div>

                <div className="bg-amber-50 dark:bg-amber-900/20 border border-amber-200 dark:border-amber-800 rounded-lg p-3 mb-4">
                    <p className="text-sm text-amber-800 dark:text-amber-200">
                        <strong>Warning:</strong> Writing incorrect values can damage equipment.
                        Ensure you understand the implications of this change.
                    </p>
                </div>

                <div className="flex space-x-3">
                    <button
                        onClick={onCancel}
                        disabled={loading}
                        className="flex-1 px-4 py-2 rounded-lg border border-slate-200 dark:border-slate-700 text-slate-600 dark:text-slate-300 hover:bg-slate-50 dark:hover:bg-slate-700 transition-colors"
                    >
                        Cancel
                    </button>
                    <button
                        onClick={onConfirm}
                        disabled={loading}
                        className="flex-1 px-4 py-2 rounded-lg bg-blue-600 text-white hover:bg-blue-700 transition-colors flex items-center justify-center space-x-2"
                    >
                        {loading ? (
                            <RefreshCw className="w-4 h-4 animate-spin" />
                        ) : (
                            <>
                                <Check className="w-4 h-4" />
                                <span>Confirm</span>
                            </>
                        )}
                    </button>
                </div>
            </div>
        </div>
    );
};

// Control Card Component
interface ControlCardProps {
    point: ControlPointState;
    onWrite: (point: ControlPointState, value: string) => void;
}

const ControlCard: React.FC<ControlCardProps> = ({ point, onWrite }) => {
    const [pendingValue, setPendingValue] = useState<string>('');
    const rawCurrentVal = extractValue(point.current_value);

    // Convert current value (may be string enum name or number) to numeric string for dropdown
    const getNumericValue = (val: number | string | null): string => {
        if (val === null) return '';
        // If it's already a number, return as string
        if (typeof val === 'number') return String(val);
        // If it's a string enum name, look up the numeric value
        if (point.symbols && typeof val === 'string') {
            const sym = point.symbols.find(s => s.name === val);
            if (sym) return String(sym.value);
        }
        // Otherwise return as-is (for numeric strings from input)
        return String(val);
    };

    const currentNumericVal = getNumericValue(rawCurrentVal);

    // Get display name for current value
    const getCurrentDisplayName = (): string => {
        if (rawCurrentVal === null) return 'Unknown';
        if (typeof rawCurrentVal === 'string' && point.symbols) {
            // Already an enum name, return it
            const sym = point.symbols.find(s => s.name === rawCurrentVal);
            if (sym) return sym.name;
        }
        if (typeof rawCurrentVal === 'number' && point.symbols) {
            const sym = point.symbols.find(s => s.value === rawCurrentVal);
            if (sym) return sym.name;
        }
        return String(rawCurrentVal);
    };

    useEffect(() => {
        if (rawCurrentVal !== null) {
            setPendingValue(getNumericValue(rawCurrentVal));
        }
    }, [rawCurrentVal, point.symbols]);

    const hasChanged = pendingValue !== currentNumericVal;

    return (
        <div className="bg-white/50 dark:bg-slate-800/50 backdrop-blur-md rounded-xl p-4 border border-white/20 dark:border-white/5">
            <div className="flex items-start justify-between mb-3">
                <div>
                    <h4 className="font-medium text-slate-800 dark:text-white">{point.label}</h4>
                    <p className="text-xs text-slate-500 dark:text-slate-400">{point.serial_number}</p>
                </div>
                <span className="text-xs px-2 py-0.5 rounded-full bg-slate-100 dark:bg-slate-700 text-slate-500">
                    {point.point_name}
                </span>
            </div>

            {/* Current Value Display */}
            <div className="mb-3 flex items-center space-x-2">
                <span className="text-xs text-slate-500 dark:text-slate-400">Current:</span>
                <span className="text-sm font-medium text-blue-600 dark:text-blue-400">
                    {getCurrentDisplayName()}
                </span>
            </div>

            <div className="flex items-center space-x-2">
                {point.symbols ? (
                    <select
                        value={pendingValue}
                        onChange={(e) => setPendingValue(e.target.value)}
                        className="flex-1 px-3 py-2 rounded-lg bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 text-sm text-slate-800 dark:text-white focus:ring-2 focus:ring-blue-500 outline-none"
                    >
                        {point.symbols.map(sym => (
                            <option key={sym.value} value={sym.value}>
                                {sym.name}
                            </option>
                        ))}
                    </select>
                ) : (
                    <div className="flex-1 flex items-center space-x-2">
                        <input
                            type="number"
                            value={pendingValue}
                            onChange={(e) => setPendingValue(e.target.value)}
                            className="flex-1 px-3 py-2 rounded-lg bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 text-sm text-slate-800 dark:text-white focus:ring-2 focus:ring-blue-500 outline-none"
                            min={0}
                            max={100}
                        />
                        {point.units && (
                            <span className="text-xs text-slate-500">{point.units}</span>
                        )}
                    </div>
                )}

                <button
                    onClick={() => onWrite(point, pendingValue)}
                    disabled={!hasChanged}
                    className={`px-3 py-2 rounded-lg text-sm font-medium transition-all ${hasChanged
                        ? 'bg-blue-600 text-white hover:bg-blue-700'
                        : 'bg-slate-100 dark:bg-slate-700 text-slate-400 cursor-not-allowed'
                        }`}
                >
                    Apply
                </button>
            </div>

            {hasChanged && (
                <p className="text-xs text-amber-600 dark:text-amber-400 mt-2">
                    ⚠️ Will change from {getCurrentDisplayName()}
                </p>
            )}
        </div>
    );
};

// Main Controls Page
export const Controls: React.FC = () => {
    const [points, setPoints] = useState<ControlPointState[]>([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);
    const [confirmModal, setConfirmModal] = useState<{ point: ControlPointState; value: string } | null>(null);
    const [writeLoading, setWriteLoading] = useState(false);
    const [notification, setNotification] = useState<{ type: 'success' | 'error'; message: string } | null>(null);

    const fetchPoints = useCallback(async () => {
        try {
            setLoading(true);
            const response = await ApiService.getControlPoints();
            setPoints(response.points);
            setError(null);
        } catch (err) {
            setError('Failed to load control points');
            console.error(err);
        } finally {
            setLoading(false);
        }
    }, []);

    useEffect(() => {
        fetchPoints();
        const interval = setInterval(fetchPoints, 10000);
        return () => clearInterval(interval);
    }, [fetchPoints]);

    const handleWrite = (point: ControlPointState, value: string) => {
        setConfirmModal({ point, value });
    };

    const confirmWrite = async () => {
        if (!confirmModal) return;

        try {
            setWriteLoading(true);
            const result = await ApiService.writeControlPoint({
                serial_number: confirmModal.point.serial_number,
                model_id: confirmModal.point.model_id,
                point_name: confirmModal.point.point_name,
                value: confirmModal.value,
            });

            if (result.success) {
                setNotification({ type: 'success', message: result.message });
                fetchPoints();
            } else {
                setNotification({ type: 'error', message: result.message });
            }
        } catch (err: any) {
            setNotification({ type: 'error', message: err.message || 'Write failed' });
        } finally {
            setWriteLoading(false);
            setConfirmModal(null);
        }
    };

    useEffect(() => {
        if (notification) {
            const timer = setTimeout(() => setNotification(null), 5000);
            return () => clearTimeout(timer);
        }
    }, [notification]);

    const groups = groupPoints(points);
    const groupIcons: { [key: string]: React.ReactNode } = {
        'Inverter': <Cpu className="w-5 h-5 text-green-500" />,
        'PV Links': <Sun className="w-5 h-5 text-yellow-500" />,
        'Battery Storage': <Battery className="w-5 h-5 text-purple-500" />,
    };

    if (loading && points.length === 0) {
        return (
            <div className="min-h-screen bg-slate-50 dark:bg-slate-900">
                <LoadingSpinner />
            </div>
        );
    }

    return (
        <div className="min-h-screen bg-slate-50 dark:bg-slate-900 pb-8">
            <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8 space-y-6">
                {/* Notification Toast */}
                {notification && (
                    <div className={`fixed top-20 right-4 z-50 px-4 py-3 rounded-lg shadow-lg ${notification.type === 'success'
                        ? 'bg-green-500 text-white'
                        : 'bg-red-500 text-white'
                        }`}>
                        <div className="flex items-center space-x-2">
                            {notification.type === 'success' ? <Check className="w-4 h-4" /> : <X className="w-4 h-4" />}
                            <span>{notification.message}</span>
                        </div>
                    </div>
                )}

                {/* Warning Banner */}
                <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-4">
                    <div className="bg-amber-50 dark:bg-amber-900/20 border border-amber-200 dark:border-amber-800 rounded-xl p-4">
                        <div className="flex items-start space-x-3">
                            <AlertTriangle className="w-5 h-5 text-amber-600 flex-shrink-0 mt-0.5" />
                            <div>
                                <p className="text-sm font-medium text-amber-800 dark:text-amber-200">Caution Required</p>
                                <p className="text-xs text-amber-700 dark:text-amber-300 mt-1">
                                    Changes to these controls directly affect device operation. Incorrect settings may cause equipment damage or safety hazards.
                                </p>
                            </div>
                        </div>
                    </div>
                </div>

                {/* Control Groups */}
                <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 space-y-6">
                    {Object.entries(groups).map(([groupName, groupPoints]) => (
                        groupPoints.length > 0 && (
                            <div key={groupName} className="bg-white/50 dark:bg-slate-800/50 backdrop-blur-md rounded-2xl p-6 border border-white/20 dark:border-white/5">
                                <div className="flex items-center space-x-3 mb-4">
                                    <div className="p-2 rounded-lg bg-slate-100 dark:bg-slate-700">
                                        {groupIcons[groupName]}
                                    </div>
                                    <h2 className="text-lg font-semibold text-slate-800 dark:text-white">{groupName}</h2>
                                    <span className="text-xs px-2 py-0.5 rounded-full bg-slate-100 dark:bg-slate-700 text-slate-500">
                                        {groupPoints.length} control{groupPoints.length !== 1 ? 's' : ''}
                                    </span>
                                </div>
                                <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                                    {groupPoints.map((point, idx) => (
                                        <ControlCard
                                            key={`${point.serial_number}-${point.model_id}-${point.point_name}-${idx}`}
                                            point={point}
                                            onWrite={handleWrite}
                                        />
                                    ))}
                                </div>
                            </div>
                        )
                    ))}

                    {points.length === 0 && !loading && (
                        <div className="text-center py-12">
                            <Sliders className="w-12 h-12 text-slate-300 mx-auto mb-4" />
                            <p className="text-slate-500 dark:text-slate-400">No controllable devices found</p>
                        </div>
                    )}
                </div>

                {/* Confirmation Modal */}
                <ConfirmModal
                    isOpen={confirmModal !== null}
                    point={confirmModal?.point || null}
                    newValue={confirmModal?.value || ''}
                    onConfirm={confirmWrite}
                    onCancel={() => setConfirmModal(null)}
                    loading={writeLoading}
                />
            </div>
        </div>
    );
};

export default Controls;
