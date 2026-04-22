import React from 'react';
import { useAgentStore } from '../stores/agentStore';

interface ModelSelectorProps {
  agentRole: string;
}

export const ModelSelector: React.FC<ModelSelectorProps> = ({ agentRole }) => {
  const { availableModels } = useAgentStore();

  return (
    <div className="model-selector">
      <span className="model-selector__brain">🧠</span>
      <select className="model-selector__select">
        <option value="">Auto-detected</option>
        {availableModels.map(model => (
          <option key={model} value={model}>{model}</option>
        ))}
        {/* Fallbacks if none loaded */}
        {availableModels.length === 0 && (
          <>
            <option value="qwen2.5:7b">qwen2.5:7b</option>
            <option value="gemma:7b">gemma:7b</option>
            <option value="llama3:8b">llama3:8b</option>
          </>
        )}
      </select>
    </div>
  );
};
