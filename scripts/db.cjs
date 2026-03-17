/**
 * DB connection for Node scripts only (seed, etc.).
 * Usage: const { db } = require('./scripts/db.cjs');
 * Do not import from frontend (better-sqlite3 is Node-only).
 */
const Database = require("better-sqlite3");
const { drizzle } = require("drizzle-orm/better-sqlite3");
const path = require("path");

const dbPath = path.join(__dirname, "..", "lol_ranked.db");
const sqlite = new Database(dbPath);
const db = drizzle(sqlite);

module.exports = { db, sqlite };
