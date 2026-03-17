CREATE TABLE `match_bans` (
	`match_id` text NOT NULL,
	`team_id` integer NOT NULL,
	`champion_id` integer NOT NULL,
	`pick_turn` integer
);
--> statement-breakpoint
CREATE TABLE `matches` (
	`match_id` text PRIMARY KEY NOT NULL,
	`game_mode` text,
	`queue_id` integer,
	`win` integer,
	`timestamp` integer
);
--> statement-breakpoint
CREATE TABLE `MD_champions` (
	`champion_id` integer PRIMARY KEY NOT NULL,
	`name` text NOT NULL,
	`role` text,
	`tags` text,
	`position` text
);
--> statement-breakpoint
CREATE TABLE `my_champ_mastery` (
	`champion_id` integer PRIMARY KEY NOT NULL,
	`champion_level` integer,
	`champion_points` integer
);
--> statement-breakpoint
CREATE TABLE `ranked_matches` (
	`match_id` text PRIMARY KEY NOT NULL,
	`game_mode` text,
	`queue_id` integer,
	`champion_name` text,
	`kills` integer,
	`deaths` integer,
	`assists` integer,
	`win` integer,
	`timestamp` integer
);
