import { useLocation, NavLink } from 'react-router-dom';
import { Sun, Database, LayoutDashboard, Moon, Settings, Server, Sliders } from 'lucide-react';
import { useTheme } from '../context/ThemeContext';

export const Navbar: React.FC = () => {
    const { theme, toggleTheme } = useTheme();
    const location = useLocation();

    // Map routes to titles
    const getPageTitle = (pathname: string) => {
        switch (pathname) {
            case '/': return 'SunSpec Gateway';
            case '/devices': return 'Device Explorer';
            case '/controls': return 'System Controls';
            case '/points': return 'Points Browser';
            case '/settings': return 'Settings';
            default: return 'SunSpec Gateway';
        }
    };

    const currentTitle = getPageTitle(location.pathname);

    return (
        <nav className="fixed top-0 left-0 w-full z-50 transition-all duration-300">
            {/* Glassmorphism Background Layer */}
            <div className="absolute inset-0 bg-white/70 dark:bg-slate-900/70 backdrop-blur-xl border-b border-white/20 dark:border-slate-800 shadow-sm transition-colors duration-300" />

            <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 relative">
                <div className="flex items-center justify-between h-16">
                    {/* Logo & Brand */}
                    <div className="flex items-center space-x-4">
                        <div className="relative group cursor-pointer">
                            <div className="absolute -inset-2 bg-gradient-to-r from-blue-600 via-indigo-500 to-purple-500 rounded-full blur opacity-25 group-hover:opacity-50 transition-opacity duration-500" />
                            <div className="relative w-10 h-10 bg-white dark:bg-slate-800 rounded-xl flex items-center justify-center border border-slate-200 dark:border-slate-700 shadow-sm group-hover:scale-105 transition-transform duration-300">
                                <Sun className="w-5 h-5 text-indigo-500 dark:text-indigo-400" />
                            </div>
                        </div>
                        <span className="text-slate-900 dark:text-white font-bold text-lg tracking-tight hidden sm:block transition-colors duration-300">
                            {currentTitle}
                        </span>
                    </div>

                    {/* Navigation Pills */}
                    <div className="flex items-center justify-center space-x-1 bg-slate-100/50 dark:bg-slate-800/50 p-1.5 rounded-2xl border border-slate-200/50 dark:border-slate-700/50 backdrop-blur-sm">
                        <NavLink
                            to="/"
                            className={({ isActive }) =>
                                `px-4 py-2 rounded-xl flex items-center space-x-2 transition-all duration-200 ${isActive
                                    ? 'bg-white dark:bg-slate-700 text-indigo-600 dark:text-white shadow-sm ring-1 ring-black/5 dark:ring-white/10'
                                    : 'text-slate-500 hover:text-slate-900 dark:text-slate-400 dark:hover:text-white hover:bg-slate-200/50 dark:hover:bg-slate-700/50'
                                }`
                            }
                            title="Dashboard"
                        >
                            <LayoutDashboard className="w-4 h-4" />
                            <span className="hidden md:inline text-sm font-medium">Dashboard</span>
                        </NavLink>

                        <NavLink
                            to="/devices"
                            className={({ isActive }) =>
                                `px-4 py-2 rounded-xl flex items-center space-x-2 transition-all duration-200 ${isActive
                                    ? 'bg-white dark:bg-slate-700 text-indigo-600 dark:text-white shadow-sm ring-1 ring-black/5 dark:ring-white/10'
                                    : 'text-slate-500 hover:text-slate-900 dark:text-slate-400 dark:hover:text-white hover:bg-slate-200/50 dark:hover:bg-slate-700/50'
                                }`
                            }
                            title="Devices"
                        >
                            <Server className="w-4 h-4" />
                            <span className="hidden md:inline text-sm font-medium">Devices</span>
                        </NavLink>

                        <NavLink
                            to="/controls"
                            className={({ isActive }) =>
                                `px-4 py-2 rounded-xl flex items-center space-x-2 transition-all duration-200 ${isActive
                                    ? 'bg-white dark:bg-slate-700 text-indigo-600 dark:text-white shadow-sm ring-1 ring-black/5 dark:ring-white/10'
                                    : 'text-slate-500 hover:text-slate-900 dark:text-slate-400 dark:hover:text-white hover:bg-slate-200/50 dark:hover:bg-slate-700/50'
                                }`
                            }
                            title="Controls"
                        >
                            <Sliders className="w-4 h-4" />
                            <span className="hidden md:inline text-sm font-medium">Controls</span>
                        </NavLink>

                        <NavLink
                            to="/points"
                            className={({ isActive }) =>
                                `px-4 py-2 rounded-xl flex items-center space-x-2 transition-all duration-200 ${isActive
                                    ? 'bg-white dark:bg-slate-700 text-indigo-600 dark:text-white shadow-sm ring-1 ring-black/5 dark:ring-white/10'
                                    : 'text-slate-500 hover:text-slate-900 dark:text-slate-400 dark:hover:text-white hover:bg-slate-200/50 dark:hover:bg-slate-700/50'
                                }`
                            }
                            title="Points Browser"
                        >
                            <Database className="w-4 h-4" />
                            <span className="hidden md:inline text-sm font-medium">Points</span>
                        </NavLink>

                        <NavLink
                            to="/settings"
                            className={({ isActive }) =>
                                `px-4 py-2 rounded-xl flex items-center space-x-2 transition-all duration-200 ${isActive
                                    ? 'bg-white dark:bg-slate-700 text-indigo-600 dark:text-white shadow-sm ring-1 ring-black/5 dark:ring-white/10'
                                    : 'text-slate-500 hover:text-slate-900 dark:text-slate-400 dark:hover:text-white hover:bg-slate-200/50 dark:hover:bg-slate-700/50'
                                }`
                            }
                            title="Settings"
                        >
                            <Settings className="w-4 h-4" />
                            <span className="hidden md:inline text-sm font-medium">Settings</span>
                        </NavLink>
                    </div>

                    {/* Theme Toggle */}
                    <div className="flex items-center space-x-4">
                        <div className="h-6 w-px bg-slate-200 dark:bg-slate-700" />
                        <button
                            onClick={toggleTheme}
                            className="p-2.5 rounded-xl bg-slate-100 dark:bg-slate-800 text-slate-500 dark:text-slate-400 hover:bg-white dark:hover:bg-slate-700 hover:text-indigo-500 dark:hover:text-indigo-400 shadow-sm border border-slate-200 dark:border-slate-700 transition-all duration-200"
                            title={theme === 'dark' ? 'Switch to Light Mode' : 'Switch to Dark Mode'}
                        >
                            {theme === 'dark' ? <Sun className="w-4 h-4" /> : <Moon className="w-4 h-4" />}
                        </button>
                    </div>
                </div>
            </div>
        </nav>
    );
};
