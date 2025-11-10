-- ORE Round Winners Database Schema
-- This schema stores historical round winner data from ORE mining protocol

-- Main table for round winner data
CREATE TABLE IF NOT EXISTS round_winners (
    id INTEGER PRIMARY KEY,                    -- Round ID
    address TEXT NOT NULL,                     -- Round account address
    winning_square INTEGER NOT NULL,             -- Winning square (0-24)
    winning_row INTEGER NOT NULL,               -- Winning row (1-5)
    winning_col INTEGER NOT NULL,               -- Winning column (1-5)
    top_miner TEXT NOT NULL,                   -- Top miner pubkey
    top_miner_reward INTEGER NOT NULL,           -- ORE reward amount
    split_reward BOOLEAN NOT NULL,              -- Whether rewards are split
    motherlode_hit BOOLEAN NOT NULL,            -- Whether motherlode was hit
    motherlode_amount INTEGER NOT NULL,          -- Motherlode amount if hit
    total_deployed INTEGER NOT NULL,             -- Total SOL deployed in round
    total_vaulted INTEGER NOT NULL,             -- Total SOL put in ORE vault
    total_winnings INTEGER NOT NULL,             -- Total SOL won by miners
    winners_count INTEGER NOT NULL,              -- Number of winners
    expires_at INTEGER NOT NULL,                -- Expiration slot for claims
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP  -- When this record was created
);

-- Index for faster round lookups
CREATE INDEX IF NOT EXISTS idx_round_winners_id ON round_winners(id);

-- Index for queries by winning square
CREATE INDEX IF NOT EXISTS idx_round_winners_winning_square ON round_winners(winning_square);

-- Index for queries by top miner
CREATE INDEX IF NOT EXISTS idx_round_winners_top_miner ON round_winners(top_miner);

-- Index for time-based queries
CREATE INDEX IF NOT EXISTS idx_round_winners_created_at ON round_winners(created_at);

-- Index for expiration tracking
CREATE INDEX IF NOT EXISTS idx_round_winners_expires_at ON round_winners(expires_at);
