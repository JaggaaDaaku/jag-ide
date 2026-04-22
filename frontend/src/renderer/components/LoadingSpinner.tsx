import React from 'react';

interface Props {
  message?: string;
}

export const LoadingSpinner: React.FC<Props> = ({ message }) => {
  return (
    <div className="loading-container">
      <div className="loading-spinner" />
      {message && <p className="loading-text">{message}</p>}
    </div>
  );
};
