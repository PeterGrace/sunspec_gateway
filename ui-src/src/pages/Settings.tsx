import React, { useState, useEffect } from 'react';
import { Save, RefreshCw, AlertCircle, Plus, Trash2, Server, Power, Activity, Wifi, WifiOff } from 'lucide-react';
import { GatewayConfig, UnitConfig, SystemStatus, DeviceStatus } from '../types/api';
import { ModelSettings } from '../components/ModelSettings';

export const Settings = () => {
    const [config, setConfig] = useState<GatewayConfig | null>(null);
    const [status, setStatus] = useState<SystemStatus | null>(null);
    const [loading, setLoading] = useState(true);
    const [saving, setSaving] = useState(false);
    const [restarting, setRestarting] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [success, setSuccess] = useState<string | null>(null);

    useEffect(() => {
        fetchConfig();
        fetchStatus();
        const interval = setInterval(fetchStatus, 5000);
        return () => clearInterval(interval);
    }, []);

    const fetchConfig = async () => {
        try {
            const response = await fetch('/api/v1/settings');
            if (!response.ok) throw new Error('Failed to fetch config');
            const data = await response.json();
            setConfig(data);
        } catch (err) {
            setError('Error loading settings');
            console.error(err);
        } finally {
            setLoading(false);
        }
    };

    const fetchStatus = async () => {
        try {
            const response = await fetch('/api/v1/settings/status');
            if (response.ok) {
                const data = await response.json();
                setStatus(data);
            }
        } catch (err) {
            console.error("Error fetching status", err);
        }
    };

    const restartServer = async () => {
        if (!confirm("Are you sure you want to restart the server?")) return;
        setRestarting(true);
        try {
            await fetch('/api/v1/settings/restart', { method: 'POST' });
            setSuccess('Server restarting... Window will reload in 10 seconds.');
            setTimeout(() => window.location.reload(), 10000);
        } catch (err) {
            setError('Failed to trigger restart');
            setRestarting(false);
        }
    };

    const saveConfig = async () => {
        if (!config) return;
        setSaving(true);
        setError(null);
        setSuccess(null);

        try {
            const response = await fetch('/api/v1/settings', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(config),
            });
            if (!response.ok) throw new Error('Failed to save config');
            setSuccess('Settings saved successfully. Restart required to apply changes.');
        } catch (err) {
            setError('Error saving settings');
            console.error(err);
        } finally {
            setSaving(false);
        }
    };

    const updateField = (field: keyof GatewayConfig, value: any) => {
        if (!config) return;
        setConfig({ ...config, [field]: value });
    };

    const addUnit = () => {
        if (!config) return;
        const newUnit: UnitConfig = { addr: '192.168.1.x', slaves: [1] };
        setConfig({ ...config, units: [...config.units, newUnit] });
    };

    const removeUnit = (index: number) => {
        if (!config) return;
        const newUnits = [...config.units];
        newUnits.splice(index, 1);
        setConfig({ ...config, units: newUnits });
    };

    const updateUnit = (index: number, field: keyof UnitConfig, value: any) => {
        if (!config) return;
        const newUnits = [...config.units];
        newUnits[index] = { ...newUnits[index], [field]: value };
        setConfig({ ...config, units: newUnits });
    };

    if (loading) return <div className="p-8 text-center">Loading settings...</div>;
    if (!config) return <div className="p-8 text-center text-red-500">Failed to load configuration</div>;

    return (
        <div className="p-8 max-w-4xl mx-auto space-y-8 pb-20">
            <div className="flex justify-end space-x-3">
                <button
                    onClick={restartServer}
                    disabled={restarting}
                    className="flex items-center space-x-2 px-4 py-2 bg-red-600 hover:bg-red-700 text-white rounded-lg transition-colors disabled:opacity-50"
                >
                    <Power className="w-4 h-4" />
                    <span>{restarting ? 'Restarting...' : 'Restart Server'}</span>
                </button>
                <button
                    onClick={saveConfig}
                    disabled={saving}
                    className="flex items-center space-x-2 px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition-colors disabled:opacity-50"
                >
                    {saving ? <RefreshCw className="w-4 h-4 animate-spin" /> : <Save className="w-4 h-4" />}
                    <span>Save Changes</span>
                </button>
            </div>

            {error && (
                <div className="p-4 bg-red-50 dark:bg-red-900/20 text-red-600 dark:text-red-400 rounded-lg flex items-center space-x-2">
                    <AlertCircle className="w-5 h-5" />
                    <span>{error}</span>
                </div>
            )}

            {success && (
                <div className="p-4 bg-green-50 dark:bg-green-900/20 text-green-600 dark:text-green-400 rounded-lg flex items-center space-x-2 border border-green-200 dark:border-green-800">
                    <Server className="w-5 h-5" />
                    <span>{success}</span>
                </div>
            )}

            {/* System Status */}
            {status && (
                <div className="bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-200 dark:border-slate-700 p-6">
                    <h2 className="text-lg font-semibold text-slate-800 dark:text-white mb-4 flex items-center">
                        <Activity className="w-5 h-5 mr-2" /> System Status
                    </h2>
                    <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                        <div className={`p-4 rounded-lg border ${status.mqtt_enabled === false
                            ? 'bg-slate-50 border-slate-200 dark:bg-slate-800 dark:border-slate-700'
                            : status.mqtt_connected
                                ? 'bg-green-50 border-green-200 dark:bg-green-900/10 dark:border-green-800'
                                : 'bg-red-50 border-red-200 dark:bg-red-900/10 dark:border-red-800'
                            }`}>
                            <div className="flex items-center justify-between">
                                <span className={`font-medium ${status.mqtt_enabled === false
                                    ? 'text-slate-500 dark:text-slate-400'
                                    : status.mqtt_connected
                                        ? 'text-green-700 dark:text-green-400'
                                        : 'text-red-700 dark:text-red-400'
                                    }`}>
                                    MQTT Broker
                                </span>
                                {status.mqtt_enabled === false ? (
                                    <WifiOff className="w-4 h-4 text-slate-400" />
                                ) : status.mqtt_connected ? (
                                    <Wifi className="w-4 h-4 text-green-600" />
                                ) : (
                                    <WifiOff className="w-4 h-4 text-red-600" />
                                )}
                            </div>
                            <div className="text-xs mt-1 text-slate-500">
                                {status.mqtt_enabled === false
                                    ? 'Disabled (Standalone Mode)'
                                    : status.mqtt_connected
                                        ? 'Connected'
                                        : `Disconnected ${status.mqtt_last_error ? `(${status.mqtt_last_error})` : ''}`
                                }
                            </div>
                        </div>
                    </div>

                    <div className="mt-4">
                        {status.devices.length === 0 ? (
                            <div className="flex items-center justify-center p-8 bg-slate-50 dark:bg-slate-900/50 rounded-lg border border-slate-200 dark:border-slate-700">
                                <RefreshCw className="w-5 h-5 mr-3 text-blue-500 animate-spin" />
                                <span className="text-slate-500 dark:text-slate-400">Scanning for devices...</span>
                            </div>
                        ) : (
                            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                                {Object.entries(
                                    status.devices.reduce((acc, dev) => {
                                        const lastColonIndex = dev.name.lastIndexOf(':');
                                        const ip = dev.name.substring(0, lastColonIndex);
                                        const slave = dev.name.substring(lastColonIndex + 1);
                                        if (!acc[ip]) acc[ip] = [];
                                        acc[ip].push({ ...dev, slaveId: slave });
                                        return acc;
                                    }, {} as Record<string, (DeviceStatus & { slaveId: string })[]>)
                                ).map(([ip, slaves]) => (
                                    <div key={ip} className="bg-slate-50 dark:bg-slate-900/50 rounded-lg border border-slate-200 dark:border-slate-700 p-4">
                                        <div className="flex items-center justify-between mb-3">
                                            <div className="font-medium text-slate-700 dark:text-slate-200 flex items-center">
                                                <Server className="w-4 h-4 mr-2 text-slate-400" />
                                                {ip}
                                            </div>
                                            <div className="text-xs text-slate-500">
                                                {slaves.length} Slaves
                                            </div>
                                        </div>
                                        <div className="flex flex-wrap gap-2">
                                            {slaves.sort((a, b) => parseInt(a.slaveId) - parseInt(b.slaveId)).map((slave) => {
                                                const isStale = (Date.now() / 1000) - slave.last_seen > 120; // 2 minutes threshold
                                                const isConnected = slave.connected && !isStale;

                                                return (
                                                    <div
                                                        key={slave.slaveId}
                                                        className={`flex items-center space-x-1 px-2 py-1 rounded text-xs border ${!slave.connected // Explicit error
                                                            ? 'bg-red-100 border-red-200 text-red-800 dark:bg-red-900/30 dark:border-red-800 dark:text-red-300'
                                                            : isStale // Connected but stale
                                                                ? 'bg-yellow-100 border-yellow-200 text-yellow-800 dark:bg-yellow-900/30 dark:border-yellow-800 dark:text-yellow-300'
                                                                : 'bg-green-100 border-green-200 text-green-800 dark:bg-green-900/30 dark:border-green-800 dark:text-green-300'
                                                            }`}
                                                        title={
                                                            !slave.connected
                                                                ? `Error: ${slave.last_error}`
                                                                : isStale
                                                                    ? `Stale: Last seen ${new Date(slave.last_seen * 1000).toLocaleTimeString()}`
                                                                    : `Online: Last seen ${new Date(slave.last_seen * 1000).toLocaleTimeString()}`
                                                        }
                                                    >
                                                        <span className="font-mono font-semibold">ID {slave.slaveId}</span>
                                                        {isConnected ? <Wifi className="w-3 h-3" /> : <WifiOff className="w-3 h-3" />}
                                                    </div>
                                                );
                                            })}
                                        </div>
                                    </div>
                                ))}
                            </div>
                        )}
                    </div>
                </div>
            )}

            {/* MQTT Settings */}
            <div className="bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-200 dark:border-slate-700 p-6">
                <h2 className="text-lg font-semibold text-slate-800 dark:text-white mb-4">MQTT Configuration</h2>
                <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                    <div>
                        <label className="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-1">Broker Address</label>
                        <input
                            type="text"
                            value={config.mqtt_server_addr}
                            onChange={(e) => updateField('mqtt_server_addr', e.target.value)}
                            placeholder="Leave empty for standalone mode"
                            className="w-full px-3 py-2 border border-slate-300 dark:border-slate-600 rounded-lg bg-white dark:bg-slate-900 text-slate-900 dark:text-white focus:ring-2 focus:ring-blue-500 outline-none"
                        />
                    </div>
                    <div>
                        <label className="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-1">Port</label>
                        <input
                            type="number"
                            value={config.mqtt_server_port || 1883}
                            onChange={(e) => updateField('mqtt_server_port', parseInt(e.target.value))}
                            className="w-full px-3 py-2 border border-slate-300 dark:border-slate-600 rounded-lg bg-white dark:bg-slate-900 text-slate-900 dark:text-white focus:ring-2 focus:ring-blue-500 outline-none"
                        />
                    </div>
                    <div>
                        <label className="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-1">Username</label>
                        <input
                            type="text"
                            value={config.mqtt_username || ''}
                            onChange={(e) => updateField('mqtt_username', e.target.value)}
                            className="w-full px-3 py-2 border border-slate-300 dark:border-slate-600 rounded-lg bg-white dark:bg-slate-900 text-slate-900 dark:text-white focus:ring-2 focus:ring-blue-500 outline-none"
                        />
                    </div>
                    <div>
                        <label className="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-1">Password</label>
                        <input
                            type="password"
                            value={config.mqtt_password || ''}
                            onChange={(e) => updateField('mqtt_password', e.target.value)}
                            className="w-full px-3 py-2 border border-slate-300 dark:border-slate-600 rounded-lg bg-white dark:bg-slate-900 text-slate-900 dark:text-white focus:ring-2 focus:ring-blue-500 outline-none"
                        />
                    </div>
                    <div>
                        <label className="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-1">Client ID</label>
                        <input
                            type="text"
                            value={config.mqtt_client_id || ''}
                            onChange={(e) => updateField('mqtt_client_id', e.target.value)}
                            placeholder="sunspec_gateway"
                            className="w-full px-3 py-2 border border-slate-300 dark:border-slate-600 rounded-lg bg-white dark:bg-slate-900 text-slate-900 dark:text-white focus:ring-2 focus:ring-blue-500 outline-none"
                        />
                    </div>
                    <div className="flex items-center justify-between p-3 bg-slate-50 dark:bg-slate-900/50 rounded-lg border border-slate-200 dark:border-slate-700">
                        <div>
                            <label className="block text-sm font-medium text-slate-700 dark:text-slate-300">Home Assistant Discovery</label>
                            <p className="text-xs text-slate-500 dark:text-slate-400 mt-0.5">Auto-create entities in Home Assistant</p>
                        </div>
                        <button
                            type="button"
                            onClick={() => updateField('hass_enabled', !(config.hass_enabled ?? true))}
                            className={`relative inline-flex h-6 w-11 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 ${(config.hass_enabled ?? true) ? 'bg-blue-600' : 'bg-slate-300 dark:bg-slate-600'
                                }`}
                        >
                            <span
                                className={`pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out ${(config.hass_enabled ?? true) ? 'translate-x-5' : 'translate-x-0'
                                    }`}
                            />
                        </button>
                    </div>
                </div>
            </div>

            {/* SunSpec Units */}
            <div className="bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-200 dark:border-slate-700 p-6">
                <div className="flex items-center justify-between mb-4">
                    <h2 className="text-lg font-semibold text-slate-800 dark:text-white">SunSpec Devices</h2>
                    <button
                        onClick={addUnit}
                        className="text-sm flex items-center space-x-1 text-blue-600 dark:text-blue-400 hover:text-blue-700 font-medium"
                    >
                        <Plus className="w-4 h-4" />
                        <span>Add Device</span>
                    </button>
                </div>

                <div className="space-y-4">
                    {config.units.map((unit, idx) => (
                        <div key={idx} className="flex items-start space-x-4 p-4 bg-slate-50 dark:bg-slate-900/50 rounded-lg border border-slate-200 dark:border-slate-700">
                            <div className="flex-1 grid grid-cols-1 md:grid-cols-2 gap-4">
                                <div>
                                    <label className="block text-xs font-medium text-slate-500 dark:text-slate-400 mb-1">IP Address / Hostname</label>
                                    <input
                                        type="text"
                                        value={unit.addr}
                                        onChange={(e) => updateUnit(idx, 'addr', e.target.value)}
                                        className="w-full px-3 py-2 border border-slate-300 dark:border-slate-600 rounded-lg bg-white dark:bg-slate-800 text-slate-900 dark:text-white text-sm"
                                    />
                                </div>
                                <div>
                                    <label className="block text-xs font-medium text-slate-500 dark:text-slate-400 mb-1">Slave IDs (comma separated)</label>
                                    <input
                                        type="text"
                                        value={unit.slaves.join(', ')}
                                        onChange={(e) => {
                                            const slaves = e.target.value.split(',').map(s => parseInt(s.trim())).filter(n => !isNaN(n));
                                            updateUnit(idx, 'slaves', slaves);
                                        }}
                                        className="w-full px-3 py-2 border border-slate-300 dark:border-slate-600 rounded-lg bg-white dark:bg-slate-800 text-slate-900 dark:text-white text-sm"
                                    />
                                </div>
                            </div>
                            <button
                                onClick={() => removeUnit(idx)}
                                className="mt-6 p-2 text-slate-400 hover:text-red-500 transition-colors"
                                title="Remove Device"
                            >
                                <Trash2 className="w-4 h-4" />
                            </button>
                        </div>
                    ))}
                </div>
            </div>

            {/* Model Settings */}
            <ModelSettings config={config} updateConfig={(newConfig) => setConfig(newConfig)} />
        </div>
    );
};
