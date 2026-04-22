-- Model Benchmarks Table
CREATE TABLE IF NOT EXISTS model_benchmarks (
    id TEXT PRIMARY KEY,
    model_name TEXT NOT NULL,
    task_type TEXT NOT NULL,
    latency_ms INTEGER NOT NULL,
    tokens_per_second REAL NOT NULL,
    total_tokens INTEGER NOT NULL,
    cost_usd REAL NOT NULL,
    timestamp TEXT NOT NULL,
    success BOOLEAN NOT NULL
);

-- Index for analytics
CREATE INDEX idx_bench_model_task ON model_benchmarks(model_name, task_type);
CREATE INDEX idx_bench_timestamp ON model_benchmarks(timestamp);
