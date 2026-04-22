import React, { useEffect } from 'react';
import { useAdminStore } from '../stores/adminStore';
import { CheckCircle, XCircle, AlertTriangle, MessageSquare } from 'lucide-react';
import { LoadingSpinner } from './LoadingSpinner';

export const ApprovalQueue: React.FC = () => {
    const { approvalQueue, fetchApprovals, decideApproval, isLoading } = useAdminStore();

    useEffect(() => {
        fetchApprovals();
    }, []);

    if (isLoading && approvalQueue.length === 0) {
        return <LoadingSpinner message="Loading pending approvals..." />;
    }

    return (
        <div style={{ display: 'flex', flexDirection: 'column', gap: '1rem' }}>
            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '1rem' }}>
                <h2 style={{ fontSize: '1.5rem', margin: 0 }}>Manual Approval Queue</h2>
                <span className="badge badge--amber">
                    {approvalQueue.length} Pending Review
                </span>
            </div>

            {approvalQueue.length === 0 ? (
                <div className="governance-empty">
                    <CheckCircle size={48} className="governance-icon--emerald" style={{ margin: '0 auto 1.5rem', display: 'block', opacity: 0.2 }} />
                    <p>No artifacts currently requiring manual approval.</p>
                </div>
            ) : (
                <div className="item-list">
                    {approvalQueue.map((approval) => (
                        <div key={approval.id} className="governance-item" style={{ flexDirection: 'column', alignItems: 'stretch' }}>
                            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start' }}>
                                <div style={{ display: 'flex', flexDirection: 'column', gap: '0.4rem' }}>
                                    <div style={{ display: 'flex', alignItems: 'center', gap: '0.8rem' }}>
                                        <span className="governance-item__title">{approval.id}</span>
                                        <span className="badge badge--amber" style={{ fontSize: '9px' }}>
                                            {approval.decision.type || 'RequiresApproval'}
                                        </span>
                                    </div>
                                    <p className="governance-item__subtitle" style={{ textTransform: 'none', fontStyle: 'italic' }}>
                                        Reasoning: {approval.decision.reasoning}
                                    </p>
                                </div>
                                <div className="similarity-score">
                                    <div className="similarity-score__value" style={{ color: 'var(--text)' }}>
                                        {(approval.decision.confidence * 100).toFixed(0)}%
                                    </div>
                                    <div className="similarity-score__label">Confidence</div>
                                </div>
                            </div>

                            {approval.decision.suggested_fixes && approval.decision.suggested_fixes.length > 0 && (
                                <div className="suggested-fixes">
                                    <h4 className="suggested-fixes__title">
                                        <AlertTriangle size={12} /> Suggested Fixes
                                    </h4>
                                    <ul className="suggested-fixes__list">
                                        {approval.decision.suggested_fixes.map((fix: string, idx: number) => (
                                            <li key={idx}>{fix}</li>
                                        ))}
                                    </ul>
                                </div>
                            )}

                            <div className="btn-group" style={{ justifyContent: 'flex-end' }}>
                                <button 
                                    onClick={() => decideApproval(approval.id, 'Rejected')}
                                    className="btn-action btn-action--reject"
                                >
                                    <XCircle size={14} /> Reject
                                </button>
                                <button 
                                    onClick={() => decideApproval(approval.id, 'Approved')}
                                    className="btn-action btn-action--approve"
                                >
                                    <CheckCircle size={14} /> Approve
                                </button>
                            </div>
                        </div>
                    ))}
                </div>
            )}
        </div>
    );
};
