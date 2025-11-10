-- ORE Round Winners Analysis Queries
-- This file contains example queries for analyzing ORE round winner data

-- 1. Most Recent Rounds
-- Get the latest rounds with complete winner information
SELECT
    id,
    winning_row,
    winning_col,
    top_miner,
    top_miner_reward,
    total_deployed,
    winners_count,
    created_at
FROM round_winners
ORDER BY id DESC
LIMIT 20;

-- 2. Motherlode Analysis
-- Find all rounds that hit the motherlode (rare events)
SELECT
    id,
    motherlode_amount,
    winners_count,
    total_deployed,
    winning_row,
    winning_col,
    created_at
FROM round_winners
WHERE motherlode_hit = TRUE
ORDER BY id DESC;

-- 3. Winning Square Distribution
-- Analyze which squares win most often (5x5 grid analysis)
SELECT
    winning_square,
    winning_row,
    winning_col,
    COUNT(*) as times_won,
    ROUND(COUNT(*) * 100.0 / (SELECT COUNT(*) FROM round_winners), 2) as win_percentage,
    AVG(total_winnings) as avg_winnings,
    AVG(total_deployed) as avg_deployed
FROM round_winners
GROUP BY winning_square, winning_row, winning_col
ORDER BY times_won DESC;

-- 4. Grid Heat Map Data
-- Format data for 5x5 grid visualization
SELECT
    winning_row,
    winning_col,
    COUNT(*) as frequency,
    ROUND(COUNT(*) * 100.0 / (SELECT COUNT(*) FROM round_winners), 1) as percentage
FROM round_winners
GROUP BY winning_row, winning_col
ORDER BY winning_row, winning_col;

-- 5. Top Miners Analysis
-- Find most successful miners
SELECT
    top_miner,
    COUNT(*) as rounds_won,
    SUM(top_miner_reward) as total_ore_rewards,
    AVG(total_deployed) as avg_deployed_per_round
FROM round_winners
WHERE top_miner_reward > 0
GROUP BY top_miner
ORDER BY rounds_won DESC
LIMIT 50;

-- 6. Reward Distribution Analysis
-- Analyze different types of rewards
SELECT
    COUNT(*) as total_rounds,
    SUM(CASE WHEN split_reward = TRUE THEN 1 ELSE 0 END) as split_reward_rounds,
    SUM(CASE WHEN motherlode_hit = TRUE THEN 1 ELSE 0 END) as motherlode_rounds,
    SUM(CASE WHEN split_reward = TRUE AND motherlode_hit = TRUE THEN 1 ELSE 0 END) as both_special_rounds,
    ROUND(AVG(top_miner_reward), 0) as avg_top_reward,
    SUM(motherlode_amount) as total_motherlode_distributed,
    ROUND(SUM(motherlode_amount) / SUM(CASE WHEN motherlode_hit = TRUE THEN 1 ELSE 0 END), 0) as avg_motherlode_amount
FROM round_winners;

-- 7. SOL Flow Analysis
-- Track SOL deployment and winnings over time
SELECT
    id,
    total_deployed,
    total_vaulted,
    total_winnings,
    ROUND(total_vaulted * 100.0 / total_deployed, 2) as vault_percentage,
    ROUND(total_winnings * 100.0 / total_deployed, 2) as winnings_percentage,
    created_at
FROM round_winners
ORDER BY id DESC
LIMIT 100;

-- 8. Weekly/Monthly Trends
-- Group rounds by time periods for trend analysis
SELECT
    strftime('%Y-%W', created_at) as week,
    COUNT(*) as rounds_completed,
    SUM(total_deployed) as weekly_deployed,
    AVG(total_winnings) as avg_winnings,
    COUNT(DISTINCT top_miner) as unique_winners
FROM round_winners
WHERE created_at >= datetime('now', '-90 days')
GROUP BY strftime('%Y-%W', created_at)
ORDER BY week DESC;

-- 9. High-Stakes Rounds
-- Find rounds with most SOL deployed
SELECT
    id,
    total_deployed,
    total_winnings,
    winners_count,
    winning_row,
    winning_col,
    ROUND(total_winnings / winners_count, 2) as avg_winner_share,
    created_at
FROM round_winners
WHERE total_deployed > 10000000000  -- >100 SOL
ORDER BY total_deployed DESC
LIMIT 20;

-- 10. Competitive Rounds
-- Rounds with many winners (high competition)
SELECT
    id,
    winners_count,
    total_deployed,
    total_winnings,
    ROUND(total_deployed / winners_count, 2) as avg_per_winner,
    split_reward,
    created_at
FROM round_winners
WHERE winners_count > 100
ORDER BY winners_count DESC
LIMIT 20;

-- 11. Split Reward Patterns
-- Analyze when split rewards occur
SELECT
    winning_square,
    winning_row,
    winning_col,
    COUNT(*) as split_count,
    ROUND(COUNT(*) * 100.0 / (SELECT COUNT(*) FROM round_winners WHERE split_reward = TRUE), 2) as split_percentage,
    AVG(total_deployed) as avg_deployed_splits
FROM round_winners
WHERE split_reward = TRUE
GROUP BY winning_square, winning_row, winning_col
ORDER BY split_count DESC;

-- 12. Recent Performance
-- Last 50 rounds with key metrics
SELECT
    id,
    winning_row || ',' || winning_col as position,
    winners_count,
    total_deployed,
    total_winnings,
    split_reward,
    motherlode_hit,
    CASE
        WHEN motherlode_hit = TRUE THEN 'MOTHERLODE!'
        WHEN split_reward = TRUE THEN 'SPLIT'
        ELSE 'NORMAL'
    END as reward_type,
    created_at
FROM round_winners
ORDER BY id DESC
LIMIT 50;

-- 13. Statistics Summary
-- Quick overview of all collected data
SELECT
    MIN(id) as earliest_round,
    MAX(id) as latest_round,
    COUNT(*) as total_rounds_analyzed,
    SUM(total_deployed) as total_sol_deployed,
    SUM(total_winnings) as total_sol_won,
    SUM(total_vaulted) as total_sol_vaulted,
    COUNT(DISTINCT top_miner) as unique_winners,
    COUNT(CASE WHEN motherlode_hit = TRUE THEN 1 END) as motherlode_hits,
    ROUND(COUNT(CASE WHEN motherlode_hit = TRUE THEN 1 END) * 100.0 / COUNT(*), 2) as motherlode_hit_rate,
    COUNT(CASE WHEN split_reward = TRUE THEN 1 END) as split_rewards,
    ROUND(COUNT(CASE WHEN split_reward = TRUE THEN 1 END) * 100.0 / COUNT(*), 2) as split_reward_rate,
    ROUND(AVG(total_deployed), 0) as avg_deployed_per_round,
    MAX(total_deployed) as highest_deployed,
    ROUND(AVG(winners_count), 1) as avg_winners_per_round,
    MAX(winners_count) as most_winners_in_round
FROM round_winners;

-- 14. Export-Friendly Format
-- Get rounds ready for CSV export or external analysis
SELECT
    id,
    address,
    winning_row,
    winning_col,
    top_miner,
    top_miner_reward,
    split_reward,
    motherlode_hit,
    motherlode_amount,
    total_deployed,
    total_vaulted,
    total_winnings,
    winners_count,
    expires_at,
    created_at
FROM round_winners
ORDER BY id;

-- 15. Gaps Detection
-- Find missing round numbers (useful for debugging data collection)
WITH RECURSIVE numbers(id) AS (
    SELECT 1 as id
    UNION ALL
    SELECT id + 1 FROM numbers WHERE id < 100000
)
SELECT
    numbers.id as missing_round_id
FROM numbers
LEFT JOIN round_winners ON numbers.id = round_winners.id
WHERE round_winners.id IS NULL
AND numbers.id <= (SELECT MAX(id) FROM round_winners)
ORDER BY numbers.id;
