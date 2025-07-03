import React from 'react';
import { X, Copy, Check } from 'lucide-react';

interface YamlModalProps {
  isOpen: boolean;
  onClose: () => void;
  modelId: number;
  pointId: string;
}

export const YamlModal: React.FC<YamlModalProps> = ({ 
  isOpen, 
  onClose, 
  modelId, 
  pointId 
}) => {
  const [copied, setCopied] = React.useState(false);

  const yamlContent = `models:
  "${modelId}":
    - catalog_ref: "${pointId}"
      interval: 15`;

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(yamlContent);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (err) {
      console.error('Failed to copy text: ', err);
    }
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 p-4">
      <div className="bg-white rounded-lg shadow-xl max-w-2xl w-full max-h-[80vh] overflow-hidden">
        {/* Header */}
        <div className="flex items-center justify-between p-6 border-b border-slate-200">
          <div>
            <h2 className="text-xl font-semibold text-slate-800">Point Configuration</h2>
            <p className="text-sm text-slate-500 mt-1">
              Model {modelId} • Point {pointId}
            </p>
          </div>
          <button
            onClick={onClose}
            className="p-2 hover:bg-slate-100 rounded-lg transition-colors duration-200"
          >
            <X className="w-5 h-5 text-slate-500" />
          </button>
        </div>

        {/* Content */}
        <div className="p-6">
          <div className="relative">
            <div className="flex items-center justify-between mb-3">
              <label className="text-sm font-medium text-slate-700">
                YAML Configuration
              </label>
              <button
                onClick={handleCopy}
                className="flex items-center space-x-2 px-3 py-1.5 text-sm bg-slate-100 hover:bg-slate-200 rounded-lg transition-colors duration-200"
              >
                {copied ? (
                  <>
                    <Check className="w-4 h-4 text-green-600" />
                    <span className="text-green-600">Copied!</span>
                  </>
                ) : (
                  <>
                    <Copy className="w-4 h-4 text-slate-600" />
                    <span className="text-slate-600">Copy</span>
                  </>
                )}
              </button>
            </div>
            
            <div className="bg-slate-900 rounded-lg p-4 overflow-x-auto">
              <pre className="text-sm text-slate-100 font-mono leading-relaxed">
                <code>{yamlContent}</code>
              </pre>
            </div>
          </div>

          <div className="mt-4 p-4 bg-blue-50 rounded-lg">
            <h3 className="text-sm font-medium text-blue-800 mb-2">Configuration Details</h3>
            <ul className="text-sm text-blue-700 space-y-1">
              <li><strong>Model ID:</strong> {modelId}</li>
              <li><strong>Point ID:</strong> {pointId}</li>
              <li><strong>Interval:</strong> 15 seconds (default polling interval)</li>
            </ul>
          </div>
        </div>

        {/* Footer */}
        <div className="flex justify-end space-x-3 p-6 border-t border-slate-200 bg-slate-50">
          <button
            onClick={onClose}
            className="px-4 py-2 text-slate-600 hover:text-slate-800 transition-colors duration-200"
          >
            Close
          </button>
          <button
            onClick={handleCopy}
            className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors duration-200"
          >
            Copy Configuration
          </button>
        </div>
      </div>
    </div>
  );
};