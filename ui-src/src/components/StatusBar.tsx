import React, { useState, useEffect } from 'react';
import { Wifi, WifiOff, Server, AlertCircle, RefreshCw } from 'lucide-react';
import { SystemStatus } from '../types/api';

export const StatusBar: React.FC = () => {
    const [status, setStatus] = useState<SystemStatus | null>(null);

    useEffect(() => {
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

        fetchStatus();
        const interval = setInterval(fetchStatus, 5000);
        return () => clearInterval(interval);
    }, []);

    if (!status) return null;

    // Calculate aggregated stats
    const totalDevices = status.devices.length;

    // Count actually connected devices (not stale)
    const connectedDevices = status.devices.filter(d => {
        const isStale = (Date.now() / 1000) - d.last_seen > 120;
        return d.connected && !isStale;
    }).length;

    const mqttConnected = status.mqtt_connected;

    return (
        <div className="fixed bottom-0 left-16 right-0 bg-white/80 dark:bg-slate-900/90 backdrop-blur-md border-t border-slate-200 dark:border-slate-800 px-4 py-1.5 flex items-center justify-between text-xs z-40 transition-colors duration-300">
            {/* Left: Overall Health */}
            <div className="flex items-center space-x-6">
                {/* MQTT Status */}
                <div className="flex items-center space-x-2">
                    <span className="text-slate-500 dark:text-slate-400 font-medium">MQTT</span>
                    {status.mqtt_enabled === false ? (
                        <div className="flex items-center text-slate-500 dark:text-slate-400 bg-slate-100 dark:bg-slate-800 px-2 py-0.5 rounded-full border border-slate-200 dark:border-slate-700">
                            <WifiOff className="w-3 h-3 mr-1.5" />
                            <span className="font-medium">Disabled</span>
                        </div>
                    ) : mqttConnected ? (
                        <div className="flex items-center text-green-600 dark:text-green-400 bg-green-50 dark:bg-green-900/20 px-2 py-0.5 rounded-full border border-green-100 dark:border-green-800/30">
                            <Wifi className="w-3 h-3 mr-1.5" />
                            <span className="font-medium">Connected</span>
                        </div>
                    ) : (
                        <div className="flex items-center text-red-600 dark:text-red-400 bg-red-50 dark:bg-red-900/20 px-2 py-0.5 rounded-full border border-red-100 dark:border-red-800/30">
                            <WifiOff className="w-3 h-3 mr-1.5" />
                            <span className="font-medium">Disconnected</span>
                        </div>
                    )}
                </div>

                <div className="h-4 w-px bg-slate-200 dark:bg-slate-700" />

                {/* Device Status */}
                <div className="flex items-center space-x-2">
                    <span className="text-slate-500 dark:text-slate-400 font-medium">Devices</span>
                    {totalDevices === 0 ? (
                        <div className="flex items-center text-blue-600 dark:text-blue-400 bg-blue-50 dark:bg-blue-900/20 px-2 py-0.5 rounded-full border border-blue-100 dark:border-blue-800/30">
                            <RefreshCw className="w-3 h-3 mr-1.5 animate-spin" />
                            <span className="font-medium">Scanning...</span>
                        </div>
                    ) : (
                        <div className="flex items-center space-x-2">
                            <div className={`flex items-center px-2 py-0.5 rounded-full border ${connectedDevices === totalDevices
                                ? 'text-green-600 dark:text-green-400 bg-green-50 dark:bg-green-900/20 border-green-100 dark:border-green-800/30'
                                : 'text-yellow-600 dark:text-yellow-400 bg-yellow-50 dark:bg-yellow-900/20 border-yellow-100 dark:border-yellow-800/30'
                                }`}>
                                <Server className="w-3 h-3 mr-1.5" />
                                <span className="font-medium">{connectedDevices}/{totalDevices} Online</span>
                            </div>
                        </div>
                    )}
                </div>
            </div>

            {/* Right: Version / Info */}
            <div className="flex items-center text-slate-400 text-[10px]">
                SunSpec Gateway v1.0
            </div>
        </div>
    );
};
