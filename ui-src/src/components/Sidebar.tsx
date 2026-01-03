import React from 'react';
import { NavLink } from 'react-router-dom';
import { Sun, Database, LayoutDashboard, Moon, Settings, Server, Sliders } from 'lucide-react';
import { useTheme } from '../context/ThemeContext';

export const Sidebar: React.FC = () => {
    const { theme, toggleTheme } = useTheme();

    return (
        <aside className="fixed left-0 top-0 h-full w-16 bg-slate-900 flex flex-col items-center py-4 z-50 border-r border-slate-800">
            {/* Logo - Modern circular design with gradient ring */}
            <div className="relative w-10 h-10 mb-8 group cursor-pointer">
                {/* Outer glow */}
                <div className="absolute inset-0 bg-gradient-to-br from-blue-500 via-purple-500 to-pink-500 rounded-xl blur-sm opacity-60 group-hover:opacity-80 transition-opacity" />
                {/* Main container */}
                <div className="relative w-full h-full bg-slate-900 rounded-xl flex items-center justify-center border border-white/10">
                    {/* Inner gradient circle */}
                    <div className="absolute inset-1 bg-gradient-to-br from-blue-500 via-purple-500 to-pink-500 rounded-lg opacity-20" />
                    <Sun className="w-5 h-5 text-blue-400 relative z-10" />
                </div>
            </div>

            {/* Nav Links */}
            <nav className="flex flex-col items-center space-y-4 flex-1">
                <NavLink
                    to="/"
                    className={({ isActive }) =>
                        `w-10 h-10 rounded-lg flex items-center justify-center transition-all duration-200 group relative ${isActive
                            ? 'bg-blue-600 text-white shadow-lg shadow-blue-500/30'
                            : 'text-slate-400 hover:text-white hover:bg-slate-800'
                        }`
                    }
                    title="Dashboard"
                >
                    <LayoutDashboard className="w-5 h-5" />
                    <span className="absolute left-14 bg-slate-800 text-white px-2 py-1 rounded text-sm opacity-0 group-hover:opacity-100 whitespace-nowrap pointer-events-none z-50">
                        Dashboard
                    </span>
                </NavLink>

                <NavLink
                    to="/devices"
                    className={({ isActive }) =>
                        `w-10 h-10 rounded-lg flex items-center justify-center transition-all duration-200 group relative ${isActive
                            ? 'bg-blue-600 text-white shadow-lg shadow-blue-500/30'
                            : 'text-slate-400 hover:text-white hover:bg-slate-800'
                        }`
                    }
                    title="Devices"
                >
                    <Server className="w-5 h-5" />
                    <span className="absolute left-14 bg-slate-800 text-white px-2 py-1 rounded text-sm opacity-0 group-hover:opacity-100 whitespace-nowrap pointer-events-none z-50">
                        Devices
                    </span>
                </NavLink>

                <NavLink
                    to="/controls"
                    className={({ isActive }) =>
                        `w-10 h-10 rounded-lg flex items-center justify-center transition-all duration-200 group relative ${isActive
                            ? 'bg-blue-600 text-white shadow-lg shadow-blue-500/30'
                            : 'text-slate-400 hover:text-white hover:bg-slate-800'
                        }`
                    }
                    title="Controls"
                >
                    <Sliders className="w-5 h-5" />
                    <span className="absolute left-14 bg-slate-800 text-white px-2 py-1 rounded text-sm opacity-0 group-hover:opacity-100 whitespace-nowrap pointer-events-none z-50">
                        Controls
                    </span>
                </NavLink>

                <NavLink
                    to="/points"
                    className={({ isActive }) =>
                        `w-10 h-10 rounded-lg flex items-center justify-center transition-all duration-200 group relative ${isActive
                            ? 'bg-blue-600 text-white shadow-lg shadow-blue-500/30'
                            : 'text-slate-400 hover:text-white hover:bg-slate-800'
                        }`
                    }
                    title="Points Browser"
                >
                    <Database className="w-5 h-5" />
                    <span className="absolute left-14 bg-slate-800 text-white px-2 py-1 rounded text-sm opacity-0 group-hover:opacity-100 whitespace-nowrap pointer-events-none z-50">
                        Points Browser
                    </span>
                </NavLink>


                <NavLink
                    to="/settings"
                    className={({ isActive }) =>
                        `w-10 h-10 rounded-lg flex items-center justify-center transition-all duration-200 group relative ${isActive
                            ? 'bg-blue-600 text-white shadow-lg shadow-blue-500/30'
                            : 'text-slate-400 hover:text-white hover:bg-slate-800'
                        }`
                    }
                    title="Settings"
                >
                    <Settings className="w-5 h-5" />
                    <span className="absolute left-14 bg-slate-800 text-white px-2 py-1 rounded text-sm opacity-0 group-hover:opacity-100 whitespace-nowrap pointer-events-none z-50">
                        Settings
                    </span>
                </NavLink>
            </nav>

            {/* Theme Toggle */}
            <button
                onClick={toggleTheme}
                className="w-10 h-10 rounded-lg flex items-center justify-center text-slate-400 hover:text-white hover:bg-slate-800 transition-all duration-200 mt-auto"
                title={`Switch to ${theme === 'light' ? 'dark' : 'light'} mode`}
            >
                {theme === 'light' ? <Moon className="w-5 h-5" /> : <Sun className="w-5 h-5" />}
            </button>
        </aside>
    );
};
