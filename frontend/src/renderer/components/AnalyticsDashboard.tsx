import React, { useEffect } from 'react';
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer } from 'recharts';
import { useAdminStore } from '../stores/adminStore';
import { LoadingSpinner } from './LoadingSpinner';

export const AnalyticsDashboard: React.FC = () => {
    const { analytics, isLoading, fetchAnalytics, error } = useAdminStore();

    useEffect(() => {
        fetchAnalytics();
    }, []);

    if (isLoading && analytics.length === 0) return <LoadingSpinner message="Loading Analytics..." />;
    if (error) return <div className="error">Error loading analytics: {error}</div>;

    return (
        <div className="analytics-dashboard">
            <header className="analytics-header">
                <h3>📈 Enterprise Usage & Cost Trends</h3>
                <div className="period-selector">
                    {/* Future: Buttons for 7d, 30d, 90d */}
                    <span>Last 30 Days</span>
                </div>
            </header>

            <div className="chart-container" style={{ width: '100%', height: 400, marginTop: '20px' }}>
                <ResponsiveContainer>
                    <LineChart data={analytics}>
                        <CartesianGrid strokeDasharray="3 3" stroke="#f0f0f0" />
                        <XAxis dataKey="date" />
                        <YAxis yAxisId="left" orientation="left" stroke="#8884d8" />
                        <YAxis yAxisId="right" orientation="right" stroke="#82ca9d" />
                        <Tooltip />
                        <Legend />
                        <Line yAxisId="left" type="monotone" dataKey="total_tokens" stroke="#8884d8" name="Tokens" activeDot={{ r: 8 }} />
                        <Line yAxisId="right" type="monotone" dataKey="cost" stroke="#82ca9d" name="Cost ($)" />
                    </LineChart>
                </ResponsiveContainer>
            </div>

            <div className="stats-summary" style={{ display: 'grid', gridTemplateColumns: 'repeat(4, 1fr)', gap: '20px', marginTop: '30px' }}>
               <div className="stat-card">
                  <label>Total Calls</label>
                  <span>{analytics.reduce((acc, curr) => acc + curr.calls, 0)}</span>
               </div>
               <div className="stat-card">
                  <label>Total Tokens</label>
                  <span>{analytics.reduce((acc, curr) => acc + curr.total_tokens, 0).toLocaleString()}</span>
               </div>
               <div className="stat-card">
                  <label>Estimated Cost</label>
                  <span>${analytics.reduce((acc, curr) => acc + curr.cost, 0).toFixed(4)}</span>
               </div>
               <div className="stat-card">
                  <label>Avg. Cost/Call</label>
                  <span>${(analytics.reduce((acc, curr) => acc + curr.cost, 0) / (analytics.reduce((acc, curr) => acc + curr.calls, 0) || 1)).toFixed(6)}</span>
               </div>
            </div>
        </div>
    );
};
