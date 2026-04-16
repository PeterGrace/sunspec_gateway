import React from 'react';
import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';
import Dashboard from './pages/Dashboard';
import { PointsBrowser } from './pages/PointsBrowser';
import { Devices } from './pages/Devices';
import { Controls } from './pages/Controls';
import { Settings } from './pages/Settings';
import { ThemeProvider } from './context/ThemeContext';
import { Navbar } from './components/Navbar';

import { StatusBar } from './components/StatusBar';

// Main content wrapper
const MainContent: React.FC<{ children: React.ReactNode }> = ({ children }) => (
  <main className="pt-16 pb-12">
    {children}
  </main>
);

function App() {
  return (
    <ThemeProvider>
      <BrowserRouter basename="/ui">
        <div className="min-h-screen bg-slate-50 dark:bg-slate-900 transition-colors duration-300">
          <Navbar />
          <MainContent>
            <Routes>
              <Route path="/" element={<Dashboard />} />
              <Route path="/devices" element={<Devices />} />
              <Route path="/controls" element={<Controls />} />
              <Route path="/points" element={<PointsBrowser />} />
              <Route path="/settings" element={<Settings />} />
              <Route path="*" element={<Navigate to="/" replace />} />
            </Routes>
          </MainContent>
          <StatusBar />
        </div>
      </BrowserRouter>
    </ThemeProvider>
  );
}

export default App;