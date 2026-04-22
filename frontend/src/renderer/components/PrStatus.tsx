import React, { useEffect, useState } from 'react';
import { GitPullRequest, ExternalLink, CheckCircle2, Clock } from 'lucide-react';
import { api, PullRequestDto } from '../services/api';
import { LoadingSpinner } from './LoadingSpinner';

export const PrStatus: React.FC = () => {
  const [prs, setPrs] = useState<PullRequestDto[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    api.listPullRequests()
      .then((data: PullRequestDto[]) => {
        setPrs(data);
        setLoading(false);
      })
      .catch(() => setLoading(false));
  }, []);

  if (loading) return <LoadingSpinner message="Loading Pull Requests..." />;

  return (
    <div className="governance-card">
      <div className="governance-header">
        <div className="governance-header__title">
          <GitPullRequest size={20} className="governance-icon--indigo" />
          <h3>Autonomous Pull Requests</h3>
        </div>
        <span className="governance-header__badge">{prs.length} Active</span>
      </div>
      
      <div className="governance-body">
        {prs.length === 0 ? (
          <div className="governance-empty">
            <Clock size={40} className="governance-icon--slate" style={{ margin: '0 auto 1rem', display: 'block', opacity: 0.5 }} />
            <p>No active pull requests from missions yet.</p>
          </div>
        ) : (
          <div className="item-list">
            {prs.map(pr => (
              <div key={pr.number} className="governance-item">
                <div className="governance-item__info">
                  <CheckCircle2 size={16} className="governance-icon--emerald" />
                  <div className="governance-item__text">
                    <div className="governance-item__title">PR #{pr.number}</div>
                    <div className="governance-item__subtitle">{pr.state}</div>
                  </div>
                </div>
                <a 
                  href={pr.html_url} 
                  target="_blank" 
                  rel="noopener noreferrer"
                  className="governance-btn-icon"
                  title="View on GitHub"
                >
                  <ExternalLink size={16} />
                </a>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
};
