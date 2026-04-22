import React, { useEffect, useState } from 'react';
import { ShieldCheck, ShieldAlert, BarChart3, Activity } from 'lucide-react';
import { LoadingSpinner } from './LoadingSpinner';

interface CoverageData {
  rust: number;
  ts: number;
  passed: boolean;
  is_mock: boolean;
}

export const CoverageReport: React.FC = () => {
  const [data, setData] = useState<CoverageData | null>(null);

  useEffect(() => {
    fetch('/api/validation/coverage')
      .then(res => res.json())
      .then(setData)
      .catch(console.error);
  }, []);

  if (!data) return <LoadingSpinner message="Loading coverage data..." />;

  const getStatusClass = (value: number) => {
    if (value >= 0.8) return 'governance-icon--emerald';
    if (value >= 0.6) return 'governance-icon--amber';
    return 'governance-icon--rose';
  };

  const getFillClass = (value: number) => {
    if (value >= 0.8) return 'progress-bar__fill--green';
    if (value >= 0.6) return 'progress-bar__fill--peach';
    return 'progress-bar__fill--red';
  };

  return (
    <div className="governance-card">
      <div className="governance-header">
        <div className="governance-header__title">
          <ShieldCheck size={20} className="governance-icon--emerald" />
          <h3>Quality Gates</h3>
        </div>
        {data.passed ? (
          <span className="badge badge--green">MERIT PASS</span>
        ) : (
          <span className="badge badge--red">GATED</span>
        )}
      </div>

      <div className="governance-body" style={{ display: 'flex', flexDirection: 'column', gap: '2rem' }}>
        {/* Rust Coverage */}
        <div className="progress-container">
          <div className="progress-label-row">
            <div className="governance-item__info">
              <Activity size={16} className="governance-icon--slate" />
              <span className="governance-item__title">Backend (Rust)</span>
            </div>
            <span className={`governance-item__title ${getStatusClass(data.rust)}`} style={{ fontFamily: 'var(--font-mono)' }}>
              {(data.rust * 100).toFixed(1)}%
            </span>
          </div>
          <div className="progress-bar">
            <div 
              className={`progress-bar__fill ${getFillClass(data.rust)}`}
              style={{ width: `${data.rust * 100}%` }}
            />
          </div>
        </div>

        {/* TS Coverage */}
        <div className="progress-container">
          <div className="progress-label-row">
            <div className="governance-item__info">
              <BarChart3 size={16} className="governance-icon--slate" />
              <span className="governance-item__title">Frontend (TSX)</span>
            </div>
            <span className={`governance-item__title ${getStatusClass(data.ts)}`} style={{ fontFamily: 'var(--font-mono)' }}>
              {(data.ts * 100).toFixed(1)}%
            </span>
          </div>
          <div className="progress-bar">
            <div 
              className={`progress-bar__fill ${getFillClass(data.ts)}`}
              style={{ width: `${data.ts * 100}%` }}
            />
          </div>
        </div>

        {data.is_mock && (
          <div className="governance-item" style={{ background: 'rgba(250, 179, 135, 0.05)', borderColor: 'rgba(250, 179, 135, 0.1)' }}>
            <div className="governance-item__info">
              <ShieldAlert size={16} className="governance-icon--amber" />
              <p style={{ fontSize: '11px', color: 'var(--overlay1)', lineHeight: '1.4' }}>
                Running in <b style={{ color: 'var(--peach)' }}>Mock Mode</b>. Actual coverage metrics are bypassed for development.
              </p>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};
