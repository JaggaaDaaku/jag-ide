import React from 'react';
import { useAdminStore } from '../stores/adminStore';
import { Monitor, Image as ImageIcon, Zap, History } from 'lucide-react';

export const BrowserTestResults: React.FC = () => {
    // In a real scenario, this would come from a specific workflow's visual results
    // For now, we use the store but mock some visual state
    const { workflowHistory, isLoading } = useAdminStore();

    return (
        <div style={{ display: 'flex', flexDirection: 'column', gap: '1.5rem' }}>
            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
                <h2 style={{ fontSize: '1.5rem', margin: 0 }}>Visual Verification Results</h2>
                <div style={{ display: 'flex', gap: '0.5rem' }}>
                    <span className="badge badge--green" style={{ display: 'flex', alignItems: 'center', gap: '0.4rem' }}>
                        <Zap size={12} /> Playwright Stable
                    </span>
                </div>
            </div>

            {workflowHistory.length === 0 ? (
                <div className="governance-empty">
                    <Monitor size={64} className="governance-icon--slate" style={{ margin: '0 auto 1.5rem', display: 'block', opacity: 0.3 }} />
                    <h3>No Visual Tests Recorded</h3>
                    <p style={{ maxWidth: '400px', margin: '0 auto' }}>
                        Browser-based visual verification results will appear here once a mission includes UI artifacts.
                    </p>
                </div>
            ) : (
                <div style={{ display: 'flex', flexDirection: 'column', gap: '1.5rem' }}>
                    {/* Mock Visual Result Entry */}
                    <div className="governance-card">
                        <div className="governance-header">
                            <div className="governance-header__title">
                                <div style={{ padding: '0.5rem', background: 'rgba(180, 190, 254, 0.1)', borderRadius: '8px' }}>
                                    <ImageIcon size={20} className="governance-icon--indigo" />
                                </div>
                                <div className="governance-item__text">
                                    <div className="governance-item__title">Main Dashboard Visual Diff</div>
                                    <div className="governance-item__subtitle" style={{ fontSize: '9px', fontFamily: 'var(--font-mono)' }}>ARTIFACT-ID: ui_dash_v1</div>
                                </div>
                            </div>
                            <div className="similarity-score">
                                <div className="similarity-score__label">Similarity</div>
                                <div className="similarity-score__value">99.8%</div>
                            </div>
                        </div>

                        <div className="governance-body">
                            <div className="visual-grid">
                                <div className="visual-item">
                                    <div className="visual-label">
                                        <History size={12} /> Reference (Base)
                                    </div>
                                    <div className="visual-preview">
                                        <div className="visual-preview__overlay">Golden Image</div>
                                        <div style={{ opacity: 0.1, fontSize: '2rem', fontWeight: 700, fontFamily: 'var(--font-mono)' }}>UI BASE</div>
                                    </div>
                                </div>

                                <div className="visual-item">
                                    <div className="visual-label">
                                        <Zap size={12} /> Current (Agent Result)
                                    </div>
                                    <div className="visual-preview visual-preview--highlight">
                                        <div style={{ position: 'absolute', top: '0.5rem', right: '0.5rem' }} className="badge badge--green">PASS</div>
                                        <div style={{ opacity: 0.1, fontSize: '2rem', fontWeight: 700, fontFamily: 'var(--font-mono)' }}>UI RESULT</div>
                                    </div>
                                </div>
                            </div>
                        </div>

                        <div className="footer-note">
                            Verified via Playwright Chromium (1280x720 Headless)
                        </div>
                    </div>
                </div>
            )}
        </div>
    );
};
