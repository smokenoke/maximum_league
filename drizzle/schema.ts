/**
 * MVP schema — from Database_league.drawio + legacy ranked_matches.
 * Phase 1.2: matches, MD_champions, my_champ_mastery, match_bans, ranked_matches.
 */
import { sqliteTable, text, integer } from "drizzle-orm/sqlite-core";

// ---- Curated / draft ----

/** Match summary (from drawio CURATED.matches) */
export const matches = sqliteTable("matches", {
  matchId: text("match_id").primaryKey(),
  gameMode: text("game_mode"),
  queueId: integer("queue_id"),
  win: integer("win"), // 0/1
  timestamp: integer("timestamp"),
});

/** Bans per match (from drawio match_bans) */
export const matchBans = sqliteTable("match_bans", {
  matchId: text("match_id").notNull(),
  teamId: integer("team_id").notNull(),
  championId: integer("champion_id").notNull(),
  pickTurn: integer("pick_turn"),
});

/** Champion masterdata (from drawio MD_champions) */
export const mdChampions = sqliteTable("MD_champions", {
  championId: integer("champion_id").primaryKey(),
  name: text("name").notNull(),
  role: text("role"),
  tags: text("tags"), // e.g. comma-separated or JSON
  position: text("position"),
});

/** User champion mastery / pool (from drawio my_champ_mastery) */
export const myChampMastery = sqliteTable("my_champ_mastery", {
  championId: integer("champion_id").primaryKey(),
  championLevel: integer("champion_level"),
  championPoints: integer("champion_points"),
});

/**
 * Matchups: which champs were on each team per match (from drawio matchups).
 * Used by recommender for personal win rate vs specific enemy champions.
 * team_100 / team_200: e.g. comma-separated champion IDs or JSON; my_team: 100 or 200.
 */
export const matchups = sqliteTable("matchups", {
  matchId: text("match_id").primaryKey(),
  team100: text("team_100").notNull(), // champion IDs on blue (e.g. "1,2,3,4,5")
  team200: text("team_200").notNull(), // champion IDs on red
  myTeam: integer("my_team").notNull(), // 100 or 200
});

// ---- Legacy (ranked_matches from League_app) ----

export const rankedMatches = sqliteTable("ranked_matches", {
  matchId: text("match_id").primaryKey(),
  gameMode: text("game_mode"),
  queueId: integer("queue_id"),
  championName: text("champion_name"),
  kills: integer("kills"),
  deaths: integer("deaths"),
  assists: integer("assists"),
  win: integer("win"), // 0/1
  timestamp: integer("timestamp"),
});
