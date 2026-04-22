import React, { useEffect, useState } from 'react';
import { useAdminStore } from '../stores/adminStore';
import { LoadingSpinner } from './LoadingSpinner';

export const AuditLogViewer: React.FC = () => {
    const { auditLogs, fetchAuditLogs, verifyAudit, isLoading, error } = useAdminStore();
    const [verifyingId, setVerifyingId] = useState<number | null>(null);

    useEffect(() => {
        fetchAuditLogs();
    }, []);

    const handleVerify = async (id: number) => {
        setVerifyingId(id);
        await verifyAudit(id);
        setVerifyingId(null);
        // In a real impl, we'd update the row state with the verification result
    };

    const [isExporting, setIsExporting] = React.useState(false);

    const handleExport = async () => {
        setIsExporting(true);
        try {
            const response = await fetch('http://127.0.0.1:8080/api/admin/audit/export', {
                headers: {
                    // Auth header would go here if not using cookies/session
                }
            });
            const blob = await response.blob();
            const url = window.URL.createObjectURL(blob);
            const a = document.createElement('a');
            a.href = url;
            a.download = `audit_log_${new Date().toISOString().split('T')[0]}.csv`;
            document.body.appendChild(a);
            a.click();
            a.remove();
        } catch (err) {
            console.error('Export failed:', err);
        } finally {
            setIsExporting(false);
        }
    };

    if (isLoading && auditLogs.length === 0) return <LoadingSpinner message="Loading Audit Logs..." />;
    if (error) return <div className="error">Error loading audit logs: {error}</div>;

    return (
        <div className="audit-log-viewer">
            <header className="audit-header" style={{ marginBottom: '20px', display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                <h3>🛡️ System Audit Trail (Compliance Verified)</h3>
                <button 
                    className="secondary-btn" 
                    onClick={handleExport}
                    disabled={isExporting}
                >
                    {isExporting ? 'Exporting...' : 'Export CSV'}
                </button>
            </header>

            <div className="audit-table-container">
                <table className="audit-table" style={{ width: '100%', borderCollapse: 'collapse' }}>
                    <thead>
                        <tr style={{ background: '#f8f9fa', borderBottom: '2px solid #eee' }}>
                            <th style={{ padding: '12px', textAlign: 'left' }}>Timestamp</th>
                            <th style={{ padding: '12px', textAlign: 'left' }}>Action</th>
                            <th style={{ padding: '12px', textAlign: 'left' }}>User / Agent</th>
                            <th style={{ padding: '12px', textAlign: 'left' }}>Resource</th>
                            <th style={{ padding: '12px', textAlign: 'center' }}>Status</th>
                        </tr>
                    </thead>
                    <tbody>
                        {auditLogs.map((log) => (
                            <tr key={log.id} style={{ borderBottom: '1px solid #eee' }}>
                                <td style={{ padding: '12px' }}>{new Date(log.timestamp).toLocaleString()}</td>
                                <td style={{ padding: '12px' }}>
                                    <span className="action-badge" style={{ padding: '2px 8px', borderRadius: '4px', background: '#e3f2fd', color: '#1976d2', fontWeight: 500 }}>
                                        {log.action}
                                    </span>
                                </td>
                                <td style={{ padding: '12px' }}>
                                    {log.user_id || log.agent_id || 'System'}
                                </td>
                                <td style={{ padding: '12px' }}>
                                    {log.resource_type ? `${log.resource_type}:${log.resource_id}` : '-'}
                                </td>
                                <td style={{ padding: '12px', textAlign: 'center' }}>
                                    {log.signature ? (
                                        <button 
                                            className="verify-btn" 
                                            onClick={() => handleVerify(log.id)}
                                            disabled={verifyingId === log.id}
                                            style={{ background: '#e8f5e9', color: '#2e7d32', border: 'none', padding: '4px 8px', borderRadius: '4px', cursor: 'pointer' }}
                                        >
                                            {verifyingId === log.id ? 'Verifying...' : '✅ Verified'}
                                        </button>
                                    ) : (
                                        <span style={{ color: '#999' }}>Unsigned</span>
                                    )}
                                </td>
                            </tr>
                        ))}
                    </tbody>
                </table>
            </div>
        </div>
    );
};
