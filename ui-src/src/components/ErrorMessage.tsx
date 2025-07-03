import React from 'react';
import { AlertCircle } from 'lucide-react';

interface ErrorMessageProps {
  message: string;
  onRetry?: () => void;
}

export const ErrorMessage: React.FC<ErrorMessageProps> = ({ message, onRetry }) => {
  return (
    <div className="flex flex-col items-center justify-center min-h-64 p-8">
      <div className="flex items-center space-x-3 text-red-600 mb-4">
        <AlertCircle className="w-8 h-8" />
        <h3 className="text-lg font-semibold">Error</h3>
      </div>
      <p className="text-slate-600 text-center mb-6 max-w-md">{message}</p>
      {onRetry && (
        <button
          onClick={onRetry}
          className="px-6 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors duration-200 font-medium"
        >
          Try Again
        </button>
      )}
    </div>
  );
};