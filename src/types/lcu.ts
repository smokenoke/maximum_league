/** From Rust: get_lcu_status */
export interface LcuStatus {
  connected: boolean;
  message: string;
  port?: number;
}

/** From Rust: get_live_draft_state */
export interface LiveDraftState {
  in_champ_select: boolean;
  phase: string;
  time_left_sec: number | null;
  my_team_bans: number[];
  their_team_bans: number[];
  my_team_picks: number[];
  their_team_picks: number[];
  local_player_cell_id: number;
}

/** From Rust: get_champion_masterdata */
export interface ChampionMasterdata {
  id: number;
  name: string;
  roles: string[];
}

/** From Rust: get_my_champion_mastery */
export interface ChampionMasteryEntry {
  championId: number;
  championLevel: number;
  championPoints: number;
}
