import React, { useEffect, useState, useCallback, useMemo } from 'react';
import {
    Sun,
    Zap,
    Battery,
    Home,
    Calendar,
    Settings,
    RefreshCw
} from 'lucide-react';
import {
    LineChart,
    Line,
    XAxis,
    YAxis,
    CartesianGrid,
    Tooltip,
    Legend,
    ResponsiveContainer,
    ReferenceLine
} from 'recharts';
import { ApiService } from '../services/api';
import {
    DashboardState,
    DeviceData,
    HistoryDataPoint
} from '../types/api';
import { LoadingSpinner } from '../components/LoadingSpinner';
import { ErrorMessage } from '../components/ErrorMessage';

// Metric Card Component
interface MetricCardProps {
    icon: React.ReactNode;
    label: string;
    value: number;
    unit: string;
    color: string;
    secondaryValue?: React.ReactNode;
}

const MetricCard: React.FC<MetricCardProps> = ({ icon, label, value, unit, color, secondaryValue }) => (
    <div className="relative overflow-hidden bg-white/50 dark:bg-slate-800/50 backdrop-blur-md rounded-2xl p-5 shadow-lg shadow-slate-200/50 dark:shadow-black/20 border border-white/20 dark:border-white/5 hover:scale-[1.02] transition-all duration-300 group">
        <div className="absolute inset-0 bg-gradient-to-br from-white/40 to-transparent dark:from-white/5 pointer-events-none" />
        <div className="relative flex items-center space-x-4">
            <div className={`p-3 rounded-xl ${color} shadow-inner`}>
                {icon}
            </div>
            <div className="flex-1">
                <div className="flex items-baseline justify-between w-full">
                    <div className="flex items-baseline space-x-1">
                        <span className="text-2xl font-bold text-slate-800 dark:text-white tracking-tight">
                            {value.toFixed(1)}
                        </span>
                        <span className="text-sm font-medium text-slate-500 dark:text-slate-400">{unit}</span>
                    </div>
                </div>
                <p className="text-sm font-medium text-slate-500 dark:text-slate-400 group-hover:text-slate-700 dark:group-hover:text-slate-200 transition-colors">{label}</p>
                {secondaryValue && (
                    <div className="mt-1">
                        {secondaryValue}
                    </div>
                )}
            </div>
        </div>
    </div>
);

// Performance Chart Component
interface PerformanceChartProps {
    data: HistoryDataPoint[];
    period: string;
    onPeriodChange: (period: string) => void;
}

const PerformanceChart: React.FC<PerformanceChartProps> = ({ data, period, onPeriodChange }) => {
    const periods = [
        { key: 'today', label: 'TODAY' },
        { key: 'yesterday', label: 'YESTERDAY' },
        { key: '7days', label: '7 DAYS' },
        { key: '30days', label: '30 DAYS' },
        { key: '12months', label: '12 MONTHS' },
    ];

    // Enhance data with numeric timestamp for continuous XAxis scaling and better scrubbing
    const chartData = useMemo(() => {
        return data.map(d => ({
            ...d,
            timestampMs: new Date(d.timestamp).getTime()
        }));
    }, [data]);

    const formatXAxis = (tickItem: number) => {
        const date = new Date(tickItem);
        if (period === 'today' || period === 'yesterday') {
            return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
        }
        return date.toLocaleDateString([], { month: 'short', day: 'numeric' });
    };

    return (
        <div className="bg-white/50 dark:bg-slate-800/50 backdrop-blur-md rounded-2xl p-6 shadow-lg shadow-slate-200/50 dark:shadow-black/20 border border-white/20 dark:border-white/5 transition-all duration-300">
            {/* Header with period buttons */}
            <div className="flex items-center justify-between mb-8">
                <div className="flex items-center space-x-3">
                    <div className="p-2 rounded-lg bg-white/50 dark:bg-slate-700/50 shadow-sm ring-1 ring-slate-900/5">
                        <Calendar className="w-5 h-5 text-slate-500 dark:text-slate-400" />
                    </div>
                    <div className="flex p-1 bg-slate-100/80 dark:bg-slate-900/50 rounded-xl border border-slate-200/50 dark:border-white/5 backdrop-blur-sm">
                        {periods.map((p) => (
                            <button
                                key={p.key}
                                onClick={() => onPeriodChange(p.key)}
                                className={`px-4 py-1.5 text-xs font-semibold rounded-lg transition-all duration-200 ${period === p.key
                                    ? 'bg-white dark:bg-slate-700 text-slate-900 dark:text-white shadow-sm ring-1 ring-slate-900/5'
                                    : 'text-slate-500 hover:text-slate-900 dark:text-slate-400 dark:hover:text-slate-200 hover:bg-white/50 dark:hover:bg-white/5'
                                    }`}
                            >
                                {p.label}
                            </button>
                        ))}
                    </div>
                </div>
            </div>

            {/* Chart */}
            <div className="h-72">
                <ResponsiveContainer width="100%" height="100%">
                    <LineChart data={chartData} margin={{ top: 10, right: 30, left: 0, bottom: 0 }}>
                        <defs>
                            <linearGradient id="gridGradient" x1="0" y1="0" x2="0" y2="1">
                                <stop offset="0%" stopColor="#94a3b8" stopOpacity={0.1} />
                                <stop offset="100%" stopColor="#94a3b8" stopOpacity={0} />
                            </linearGradient>
                        </defs>
                        <CartesianGrid strokeDasharray="3 3" stroke="url(#gridGradient)" vertical={false} />
                        <XAxis
                            dataKey="timestampMs"
                            type="number"
                            domain={['dataMin', 'dataMax']}
                            scale="time"
                            tickFormatter={formatXAxis}
                            stroke="#94a3b8"
                            fontSize={11}
                            tickLine={false}
                            axisLine={false}
                            dy={10}
                        />
                        <YAxis
                            stroke="#94a3b8"
                            fontSize={11}
                            tickFormatter={(value) => `${value}`}
                            domain={['auto', 'auto']}
                            tickLine={false}
                            axisLine={false}
                            dx={-10}
                        />
                        <Tooltip
                            formatter={(value: number, name: string) => {
                                const absValue = Math.abs(value).toFixed(2);
                                if (name === 'Grid') {
                                    return value > 0 ? [`${absValue} kW (Import)`, name] : [`${absValue} kW (Export)`, name];
                                }
                                if (name === 'Battery') {
                                    return value > 0 ? [`${absValue} kW (Discharge)`, name] : [`${absValue} kW (Charge)`, name];
                                }
                                return [`${absValue} kW`, name];
                            }}
                            labelFormatter={(label) => new Date(label).toLocaleString()}
                            contentStyle={{
                                backgroundColor: 'rgba(15, 23, 42, 0.9)',
                                border: '1px solid rgba(255, 255, 255, 0.1)',
                                borderRadius: '12px',
                                color: '#F8FAFC',
                                boxShadow: '0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -1px rgba(0, 0, 0, 0.06)',
                                backdropFilter: 'blur(8px)',
                                padding: '12px'
                            }}
                            itemStyle={{ color: '#E2E8F0', fontSize: '12px', padding: '2px 0' }}
                            cursor={{ stroke: '#94a3b8', strokeWidth: 1, strokeDasharray: '3 3' }}
                        />
                        <Legend iconType="circle" wrapperStyle={{ paddingTop: '20px' }} />
                        <ReferenceLine y={0} stroke="#475569" strokeDasharray="3 3" />
                        <Line
                            type="monotone"
                            dataKey="solar"
                            stroke="#F59E0B"
                            strokeWidth={3}
                            dot={false}
                            activeDot={{ r: 6, strokeWidth: 4, stroke: '#F59E0B', fill: '#FFF' }}
                            name="Solar"
                            isAnimationActive={false}
                        />
                        <Line
                            type="monotone"
                            dataKey="battery"
                            stroke="#10B981"
                            strokeWidth={3}
                            dot={false}
                            activeDot={{ r: 6, strokeWidth: 4, stroke: '#10B981', fill: '#FFF' }}
                            name="Battery"
                            isAnimationActive={false}
                        />
                        <Line
                            type="monotone"
                            dataKey="grid"
                            stroke="#8B5CF6"
                            strokeWidth={3}
                            dot={false}
                            activeDot={{ r: 6, strokeWidth: 4, stroke: '#8B5CF6', fill: '#FFF' }}
                            name="Grid"
                            isAnimationActive={false}
                        />
                        <Line
                            type="monotone"
                            dataKey="consumption"
                            stroke="#06B6D4"
                            strokeWidth={3}
                            dot={false}
                            activeDot={{ r: 6, strokeWidth: 4, stroke: '#06B6D4', fill: '#FFF' }}
                            name="Consumption"
                            isAnimationActive={false}
                        />
                    </LineChart>
                </ResponsiveContainer>
            </div>
        </div>
    );
};

// Device Table Component
interface DeviceTableProps {
    title: string;
    icon: React.ReactNode;
    devices: DeviceData[];
    columns: { key: keyof DeviceData | 'actions'; label: string; format?: (v: any) => string }[];
}

const DeviceTable: React.FC<DeviceTableProps> = ({ title, icon, devices, columns }) => {
    const formatValue = (value: any, format?: (v: any) => string) => {
        if (value === undefined || value === null) return '--';
        if (format) return format(value);
        if (typeof value === 'number') return value.toFixed(1);
        return String(value);
    };

    // Calculate summaries
    const totalPower = devices.reduce((sum, d) => sum + (d.power || 0), 0) / 1000; // kW
    const totalEnergy = devices.reduce((sum, d) => sum + (d.energy_today || 0), 0); // kWh (backend returns kWh)

    const showSoC = title.toLowerCase().includes('batteries') || devices.some(d => d.device_type === 'battery');
    const batteryDevices = devices.filter(d => d.device_type === 'battery' || d.soc !== undefined);
    const avgSoC = batteryDevices.length > 0
        ? batteryDevices.reduce((sum, d) => sum + (d.soc || 0), 0) / batteryDevices.length
        : 0;

    return (
        <div className="bg-white/50 dark:bg-slate-800/50 backdrop-blur-md rounded-2xl shadow-lg shadow-slate-200/50 dark:shadow-black/20 border border-white/20 dark:border-white/5 overflow-hidden transition-all duration-300">
            <div className="px-6 py-4 border-b border-white/10 dark:border-slate-700/50 flex items-center bg-white/40 dark:bg-slate-800/40">

                {/* Left: Title */}
                <div className="flex items-center space-x-3 w-1/4">
                    <div className="p-2 rounded-lg bg-white/50 dark:bg-slate-700/50 shadow-sm ring-1 ring-slate-900/5">
                        {icon}
                    </div>
                    <h3 className="text-lg font-semibold text-slate-800 dark:text-white tracking-tight">{title}</h3>
                    <span className="px-2 py-0.5 rounded-full text-xs font-medium bg-slate-100 dark:bg-slate-700 text-slate-500 dark:text-slate-400">
                        {devices.length}
                    </span>
                </div>

                {/* Center: Summaries */}
                <div className="flex-1 flex justify-center space-x-12">
                    <div className="flex flex-col items-center group">
                        <span className="text-[10px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-widest mb-1 group-hover:text-slate-600 dark:group-hover:text-slate-300 transition-colors">Total Power</span>
                        <span className="text-xl font-bold bg-gradient-to-r from-slate-700 to-slate-900 dark:from-white dark:to-slate-300 bg-clip-text text-transparent font-mono">
                            {totalPower.toFixed(2)} <span className="text-sm font-sans text-slate-400">kW</span>
                        </span>
                    </div>

                    <div className="flex flex-col items-center group">
                        <span className="text-[10px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-widest mb-1 group-hover:text-slate-600 dark:group-hover:text-slate-300 transition-colors">Energy Today</span>
                        <span className="text-xl font-bold bg-gradient-to-r from-slate-700 to-slate-900 dark:from-white dark:to-slate-300 bg-clip-text text-transparent font-mono">
                            {totalEnergy.toFixed(1)} <span className="text-sm font-sans text-slate-400">kWh</span>
                        </span>
                    </div>

                    {showSoC && (
                        <div className="flex flex-col items-center group">
                            <span className="text-[10px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-widest mb-1 group-hover:text-slate-600 dark:group-hover:text-slate-300 transition-colors">Avg SoC</span>
                            <span className="text-xl font-bold bg-gradient-to-r from-slate-700 to-slate-900 dark:from-white dark:to-slate-300 bg-clip-text text-transparent font-mono">
                                {avgSoC.toFixed(1)} <span className="text-sm font-sans text-slate-400">%</span>
                            </span>
                        </div>
                    )}
                </div>

                {/* Right: Spacer/Actions */}
                <div className="w-1/4 flex justify-end">
                </div>
            </div>

            <div className="overflow-x-auto">
                <table className="w-full">
                    <thead className="bg-slate-50/50 dark:bg-slate-700/20 text-xs uppercase tracking-wider font-semibold text-slate-500 dark:text-slate-400 border-b border-slate-100 dark:border-white/5">
                        <tr>
                            {columns.map((col) => (
                                <th key={col.key} className="px-6 py-4 text-left first:pl-8 last:pr-8">
                                    {col.label}
                                </th>
                            ))}
                        </tr>
                    </thead>
                    <tbody className="divide-y divide-slate-100 dark:divide-white/5">
                        {devices.length === 0 ? (
                            <tr>
                                <td colSpan={columns.length} className="px-6 py-12">
                                    <div className="flex items-center justify-center">
                                        <RefreshCw className="w-5 h-5 mr-3 text-blue-500 animate-spin" />
                                        <span className="text-slate-500 dark:text-slate-400">Waiting for device data...</span>
                                    </div>
                                </td>
                            </tr>
                        ) : (
                            devices.map((device, idx) => (
                                <tr key={device.serial_number + idx} className="hover:bg-blue-50/50 dark:hover:bg-blue-900/10 transition-colors duration-150 group">
                                    {columns.map((col) => (
                                        <td key={col.key} className="px-6 py-4 text-sm text-slate-600 dark:text-slate-300 first:pl-8 last:pr-8">
                                            {col.key === 'actions' ? (
                                                <button className="p-1.5 rounded-lg text-slate-400 hover:text-blue-600 hover:bg-blue-50 dark:hover:bg-blue-900/30 dark:hover:text-blue-400 transition-all opacity-0 group-hover:opacity-100">
                                                    <Settings className="w-4 h-4" />
                                                </button>
                                            ) : (
                                                <span className="font-medium">{formatValue(device[col.key], col.format)}</span>
                                            )}
                                        </td>
                                    ))}
                                </tr>
                            ))
                        )}
                    </tbody>
                </table>
            </div>
        </div>
    );
};

// Main Dashboard Component
export const Dashboard: React.FC = () => {
    const [dashboardState, setDashboardState] = useState<DashboardState | null>(null);
    const [historyData, setHistoryData] = useState<HistoryDataPoint[]>([]);
    const [historyPeriod, setHistoryPeriod] = useState('today');
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);
    const [connected, setConnected] = useState(false);

    const fetchInitialData = useCallback(async () => {
        try {
            setLoading(true);
            setError(null);

            const [devices, metrics, powerFlow, history] = await Promise.all([
                ApiService.getDashboardDevices(),
                ApiService.getDashboardMetrics(),
                ApiService.getPowerFlow(),
                ApiService.getHistory(historyPeriod)
            ]);

            setDashboardState({
                devices,
                metrics,
                power_flow: powerFlow,
                alerts: [],
                controls: [],
                timestamp: new Date().toISOString()
            });
            setHistoryData(history.data);
        } catch (err) {
            setError('Failed to load dashboard data. Please try again.');
            console.error(err);
        } finally {
            setLoading(false);
        }
    }, [historyPeriod]);

    const handlePeriodChange = async (period: string) => {
        setHistoryPeriod(period);
        try {
            const history = await ApiService.getHistory(period);
            setHistoryData(history.data);
        } catch (err) {
            console.error('Failed to fetch history:', err);
        }
    };

    useEffect(() => {
        fetchInitialData();

        // Set up SSE connection
        const eventSource = ApiService.createDashboardStream(
            (state) => {
                setDashboardState(state);
                setConnected(true);
                setError(null);

                // Update history with live point if viewing today
                if (historyPeriod === 'today') {
                    setHistoryData(prev => {
                        // Safety check if prev is undefined
                        if (!prev) return [];

                        const last = prev[prev.length - 1];
                        const newTs = new Date(state.timestamp).getTime();

                        // Prevent duplicate timestamps (if updates are too fast)
                        if (last && new Date(last.timestamp).getTime() >= newTs) return prev;

                        return [...prev, {
                            timestamp: state.timestamp,
                            solar: state.power_flow.solar_power / 1000,
                            battery: state.power_flow.battery_power / 1000,
                            grid: state.power_flow.grid_power / 1000,
                            consumption: state.power_flow.consumption_power / 1000
                        }];
                    });
                }
            },
            () => {
                setConnected(false);
            }
        );

        return () => {
            eventSource.close();
        };
    }, [fetchInitialData]);

    if (loading) {
        return (
            <div className="min-h-screen bg-slate-50 dark:bg-slate-900">
                <LoadingSpinner />
            </div>
        );
    }

    if (error && !dashboardState) {
        return (
            <div className="min-h-screen bg-slate-50 dark:bg-slate-900">
                <ErrorMessage message={error} onRetry={fetchInitialData} />
            </div>
        );
    }

    const { devices, metrics } = dashboardState || {
        devices: { inverters: [], pv_links: [], batteries: [] },
        metrics: { yield_today: 0, yield_yesterday: 0, consumption_today: 0, consumption_yesterday: 0, grid_net_today: 0, grid_net_month: 0, battery_in_today: 0, battery_out_today: 0 }
    };

    const inverterColumns = [
        { key: 'serial_number' as const, label: 'Device ID' },
        { key: 'model_name' as const, label: 'Model' },
        { key: 'operating_mode' as const, label: 'Mode' },
        { key: 'state' as const, label: 'State' },
        { key: 'power' as const, label: 'Power', format: (v: number) => `${v.toFixed(0)} W` },
        { key: 'voltage' as const, label: 'Voltage', format: (v: number) => `${v.toFixed(1)} V` },
        { key: 'frequency' as const, label: 'Freq', format: (v: number) => `${v.toFixed(1)} Hz` },
        { key: 'lifetime_energy' as const, label: 'Lifetime', format: (v: number) => `${v.toFixed(1)} kWh` },
        { key: 'actions' as const, label: '' }
    ];

    const pvLinkColumns = [
        { key: 'serial_number' as const, label: 'Device ID' },
        { key: 'model_name' as const, label: 'Model' },
        { key: 'state' as const, label: 'State' },
        { key: 'power' as const, label: 'Power', format: (v: number) => `${v.toFixed(0)} W` },
        { key: 'voltage' as const, label: 'Voltage', format: (v: number) => `${v.toFixed(1)} V` },
        { key: 'dc_current' as const, label: 'Current', format: (v: number) => `${v.toFixed(1)} A` },
        { key: 'lifetime_energy' as const, label: 'Lifetime', format: (v: number) => `${v.toFixed(1)} kWh` },
        { key: 'actions' as const, label: '' }
    ];

    const batteryColumns = [
        { key: 'serial_number' as const, label: 'Device ID' },
        { key: 'model_name' as const, label: 'Model' },
        { key: 'state' as const, label: 'State' },
        { key: 'soc' as const, label: 'SoC', format: (v: number) => `${v.toFixed(0)}%` },
        { key: 'soh' as const, label: 'SoH', format: (v: number) => `${v.toFixed(0)}%` },
        { key: 'power' as const, label: 'Power', format: (v: number) => `${v.toFixed(0)} W` },
        { key: 'voltage' as const, label: 'Voltage', format: (v: number) => `${v.toFixed(1)} V` },
        { key: 'actions' as const, label: '' }
    ];

    return (
        <div className="min-h-screen bg-slate-50 dark:bg-slate-900 pb-8">
            <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8 space-y-8">
                {/* Metrics Row */}
                <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-5 gap-4">
                    <MetricCard
                        icon={<Sun className="w-5 h-5 text-yellow-600" />}
                        label="Yield today"
                        value={metrics.yield_today}
                        unit="kWh"
                        color="bg-yellow-100 dark:bg-yellow-900/30"
                    />
                    <MetricCard
                        icon={<Sun className="w-5 h-5 text-yellow-600" />}
                        label="Yield yesterday"
                        value={metrics.yield_yesterday}
                        unit="kWh"
                        color="bg-yellow-100 dark:bg-yellow-900/30"
                    />
                    <MetricCard
                        icon={<Home className="w-5 h-5 text-blue-600" />}
                        label="Consumption today"
                        value={metrics.consumption_today}
                        unit="kWh"
                        color="bg-blue-100 dark:bg-blue-900/30"
                    />
                    <MetricCard
                        icon={<Zap className="w-5 h-5 text-green-600" />}
                        label="Grid net today"
                        value={metrics.grid_net_today}
                        unit="kWh"
                        color="bg-green-100 dark:bg-green-900/30"
                        secondaryValue={
                            <div className="flex space-x-2 text-[10px] text-slate-500 font-mono">
                                <span className={metrics.grid_export_today > 0 ? "text-purple-600 dark:text-purple-400" : ""}>Exp: {metrics.grid_export_today?.toFixed(1) || '0.0'}</span>
                                <span className="text-slate-300">|</span>
                                <span className={metrics.grid_import_today > 0 ? "text-blue-600 dark:text-blue-400" : ""}>Imp: {metrics.grid_import_today?.toFixed(1) || '0.0'}</span>
                            </div>
                        }
                    />
                    <MetricCard
                        icon={<Zap className="w-5 h-5 text-green-600" />}
                        label="Grid net month"
                        value={metrics.grid_net_month}
                        unit="kWh"
                        color="bg-green-100 dark:bg-green-900/30"
                    />
                </div>

                {/* Performance Chart */}
                <PerformanceChart
                    data={historyData}
                    period={historyPeriod}
                    onPeriodChange={handlePeriodChange}
                />

                {/* Device Tables */}
                <DeviceTable
                    title="Inverters"
                    icon={<Zap className="w-5 h-5 text-blue-500" />}
                    devices={devices.inverters}
                    columns={inverterColumns}
                />

                <DeviceTable
                    title="PV Links"
                    icon={<Sun className="w-5 h-5 text-yellow-500" />}
                    devices={devices.pv_links}
                    columns={pvLinkColumns}
                />

                <DeviceTable
                    title="Batteries"
                    icon={<Battery className="w-5 h-5 text-purple-500" />}
                    devices={devices.batteries}
                    columns={batteryColumns}
                />
            </div>
        </div>
    );
};

export default Dashboard;
