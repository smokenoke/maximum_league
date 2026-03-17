//! LCU (League Client API) integration.
//! Phase 1.3: cert handling, port discovery, draft state.
//! Phase 1.5: champion masterdata (Community Dragon), user champion mastery.

mod client;
mod credentials;

use serde::{Deserialize, Serialize};

const CHAMPION_SUMMARY_URL: &str =
    "https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/v1/champion-summary.json";

#[derive(Debug, Serialize)]
pub struct LcuStatus {
    pub connected: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,
}

/// Returns connection status; if League client is running, discovers port and checks /current-summoner.
#[tauri::command]
pub async fn get_lcu_status() -> LcuStatus {
    let creds = match credentials::discover_credentials() {
        Some(c) => c,
        None => {
            return LcuStatus {
                connected: false,
                message: "League Client not found (no port/auth).".to_string(),
                port: None,
            };
        }
    };
    match client::lcu_get(&creds, "/lol-summoner/v1/current-summoner").await {
        Ok((status, _)) if status == 200 => LcuStatus {
            connected: true,
            message: "Connected to League Client.".to_string(),
            port: Some(creds.port),
        },
        Ok((status, _)) => LcuStatus {
            connected: false,
            message: format!("LCU returned status {}.", status),
            port: Some(creds.port),
        },
        Err(e) => LcuStatus {
            connected: false,
            message: format!("LCU request failed: {}", e),
            port: Some(creds.port),
        },
    }
}

// ---- Champ select session (simplified for recommender) ----

#[derive(Debug, Serialize, Clone)]
pub struct LiveDraftState {
    pub in_champ_select: bool,
    pub phase: String,
    pub time_left_sec: Option<u64>,
    pub my_team_bans: Vec<i64>,
    pub their_team_bans: Vec<i64>,
    pub my_team_picks: Vec<i64>,
    pub their_team_picks: Vec<i64>,
    pub local_player_cell_id: i64,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ChampSelectBans {
    #[serde(default)]
    my_team_bans: Vec<i64>,
    #[serde(default)]
    their_team_bans: Vec<i64>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ChampSelectTimer {
    phase: Option<String>,
    time_left_in_phase: Option<u64>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ChampSelectAction {
    #[serde(default)]
    champion_id: i64,
    #[serde(default)]
    actor_cell_id: i64,
    #[serde(default)]
    r#type: String,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ChampSelectSession {
    #[serde(default)]
    bans: ChampSelectBans,
    #[serde(default)]
    timer: ChampSelectTimer,
    #[serde(default)]
    my_team: Vec<ChampSelectCell>,
    #[serde(default)]
    their_team: Vec<ChampSelectCell>,
    #[serde(default)]
    actions: Vec<Vec<ChampSelectAction>>,
    local_player_cell_id: Option<i64>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ChampSelectCell {
    #[serde(default)]
    cell_id: i64,
    #[serde(default)]
    champion_id: i64,
}

/// Returns current champ select state (bans, picks, phase). Empty if not in champ select.
#[tauri::command]
pub async fn get_live_draft_state() -> Result<LiveDraftState, String> {
    let creds = credentials::discover_credentials()
        .ok_or_else(|| "League Client not found.".to_string())?;
    let (status, body) = client::lcu_get(&creds, "/lol-champ-select/v1/session")
        .await
        .map_err(|e| format!("LCU request failed: {}", e))?;
    if status == 404 || body.is_empty() || body == "null" {
        return Ok(LiveDraftState {
            in_champ_select: false,
            phase: String::new(),
            time_left_sec: None,
            my_team_bans: vec![],
            their_team_bans: vec![],
            my_team_picks: vec![],
            their_team_picks: vec![],
            local_player_cell_id: -1,
        });
    }
    if status != 200 {
        return Err(format!("Champ select endpoint returned {}", status));
    }
    let session: ChampSelectSession = serde_json::from_str(&body)
        .map_err(|e| format!("Invalid champ select JSON: {}", e))?;
    let my_team_cell_ids: std::collections::HashSet<i64> = session
        .my_team
        .iter()
        .map(|c| c.cell_id)
        .filter(|&id| id >= 0)
        .collect();
    let their_team_cell_ids: std::collections::HashSet<i64> = session
        .their_team
        .iter()
        .map(|c| c.cell_id)
        .filter(|&id| id >= 0)
        .collect();
    let (my_team_bans, their_team_bans) = if session.bans.my_team_bans.is_empty()
        && session.bans.their_team_bans.is_empty()
    {
        let mut my = Vec::new();
        let mut their = Vec::new();
        for turn in &session.actions {
            for act in turn {
                if act.r#type.eq_ignore_ascii_case("ban") && act.champion_id > 0 {
                    if my_team_cell_ids.contains(&act.actor_cell_id) {
                        my.push(act.champion_id);
                    } else if their_team_cell_ids.contains(&act.actor_cell_id) {
                        their.push(act.champion_id);
                    }
                }
            }
        }
        (my, their)
    } else {
        (
            session.bans.my_team_bans,
            session.bans.their_team_bans,
        )
    };
    let my_team_picks: Vec<i64> = session
        .my_team
        .iter()
        .filter(|c| c.champion_id > 0)
        .map(|c| c.champion_id)
        .collect();
    let their_team_picks: Vec<i64> = session
        .their_team
        .iter()
        .filter(|c| c.champion_id > 0)
        .map(|c| c.champion_id)
        .collect();
    let time_left_sec = session
        .timer
        .time_left_in_phase
        .map(|ms| ms / 1000);
    Ok(LiveDraftState {
        in_champ_select: true,
        phase: session.timer.phase.unwrap_or_default(),
        time_left_sec,
        my_team_bans,
        their_team_bans,
        my_team_picks,
        their_team_picks,
        local_player_cell_id: session.local_player_cell_id.unwrap_or(-1),
    })
}

// ---- Phase 1.5: Champion masterdata & user pool ----

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChampionMasterdata {
    pub id: i64,
    pub name: String,
    pub roles: Vec<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CommunityDragonChampion {
    #[serde(default)]
    id: i64,
    #[serde(default)]
    name: String,
    #[serde(default)]
    roles: Vec<String>,
}

/// Fetches champion list (id, name, roles) from Community Dragon.
#[tauri::command]
pub async fn get_champion_masterdata() -> Result<Vec<ChampionMasterdata>, String> {
    let body = client::fetch_public_url(CHAMPION_SUMMARY_URL).await?;
    let raw: Vec<CommunityDragonChampion> =
        serde_json::from_str(&body).map_err(|e| format!("Invalid champion-summary JSON: {}", e))?;
    Ok(raw
        .into_iter()
        .filter(|c| c.id > 0)
        .map(|c| ChampionMasterdata {
            id: c.id,
            name: c.name,
            roles: c.roles,
        })
        .collect())
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChampionMasteryEntry {
    pub champion_id: i64,
    pub champion_level: i64,
    pub champion_points: i64,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CurrentSummonerResponse {
    /// LCU may return "summonerId" or "id" as number or string.
    #[serde(default, alias = "id", deserialize_with = "deserialize_summoner_id")]
    summoner_id: i64,
    #[serde(default)]
    puuid: Option<String>,
}

fn deserialize_summoner_id<'de, D>(d: D) -> Result<i64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Visitor;
    struct V;
    impl<'de> Visitor<'de> for V {
        type Value = i64;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("number or string")
        }
        fn visit_i64<E: serde::de::Error>(self, v: i64) -> Result<i64, E> {
            Ok(v)
        }
        fn visit_u64<E: serde::de::Error>(self, v: u64) -> Result<i64, E> {
            Ok(v as i64)
        }
        fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<i64, E> {
            v.parse().map_err(serde::de::Error::custom)
        }
    }
    d.deserialize_any(V)
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LcuChampionMasteryEntry {
    #[serde(default)]
    champion_id: i64,
    #[serde(default)]
    champion_level: i64,
    #[serde(default)]
    champion_points: i64,
}

// ---- Debug: discover current-summoner shape and collections spec ----

/// Returns what the LCU current-summoner endpoint actually returns: keys and type/preview per field.
/// Run from devtools: invoke('get_lcu_current_summoner_debug').then(console.log)
#[tauri::command]
pub async fn get_lcu_current_summoner_debug() -> Result<serde_json::Value, String> {
    let creds = credentials::discover_credentials()
        .ok_or_else(|| "League Client not found.".to_string())?;
    let (status, body) = client::lcu_get(&creds, "/lol-summoner/v1/current-summoner")
        .await
        .map_err(|e| format!("LCU request failed: {}", e))?;
    if status != 200 {
        return Err(format!("current-summoner returned {}", status));
    }
    let v: serde_json::Value =
        serde_json::from_str(&body).map_err(|e| format!("Invalid JSON: {}", e))?;
    let obj = v.as_object().ok_or("Response is not a JSON object")?;
    let mut fields = serde_json::Map::new();
    for (k, val) in obj {
        let preview = match val {
            serde_json::Value::Null => serde_json::json!("null"),
            serde_json::Value::Bool(b) => serde_json::json!(*b),
            serde_json::Value::Number(n) => serde_json::json!(n.to_string()),
            serde_json::Value::String(s) => {
                let len = s.len();
                let tail = if len > 8 { format!("...{}", &s[s.len().saturating_sub(4)..]) } else { s.clone() };
                serde_json::json!({ "type": "string", "length": len, "preview": tail })
            }
            serde_json::Value::Array(a) => serde_json::json!({ "type": "array", "length": a.len() }),
            serde_json::Value::Object(_) => serde_json::json!({ "type": "object" }),
        };
        fields.insert(k.clone(), preview);
    }
    let keys: Vec<String> = obj.keys().cloned().collect();
    Ok(serde_json::json!({
        "keys": keys,
        "fields": fields,
    }))
}

/// Fetches LCU swagger and returns paths mentioning champion-mastery or collections.
/// Run from devtools: invoke('get_lcu_collections_spec').then(console.log)
#[tauri::command]
pub async fn get_lcu_collections_spec() -> Result<serde_json::Value, String> {
    let creds = credentials::discover_credentials()
        .ok_or_else(|| "League Client not found.".to_string())?;
    let mut swagger_status = Vec::new();
    for spec_path in ["/swagger/v2/swagger.json", "/swagger/v3/openapi.json"] {
        let (status, body) = match client::lcu_get(&creds, spec_path).await {
            Ok(r) => r,
            Err(e) => {
                swagger_status.push(serde_json::json!({
                    "url": spec_path,
                    "error": e,
                }));
                continue;
            }
        };
        swagger_status.push(serde_json::json!({
            "url": spec_path,
            "status": status,
            "body_length": body.len(),
        }));
        if status != 200 || body.is_empty() {
            continue;
        }
        let spec: serde_json::Value =
            serde_json::from_str(&body).map_err(|e| format!("Invalid swagger JSON: {}", e))?;
        let paths = match spec.get("paths").and_then(|p| p.as_object()) {
            Some(p) => p,
            None => continue,
        };
        let mut mastery_paths = vec![];
        let mut collections_paths = vec![];
        let mut any_mastery_paths = vec![];
        for (path_key, _) in paths {
            let key_lower = path_key.to_lowercase();
            let path_params: Vec<String> = path_key
                .split('/')
                .filter(|s| s.starts_with('{') && s.ends_with('}'))
                .map(|s| s.trim_matches(|c| c == '{' || c == '}').to_string())
                .collect();
            if key_lower.contains("champion-mastery") || key_lower.contains("championmastery") {
                mastery_paths.push(serde_json::json!({
                    "path": path_key,
                    "path_parameter_names": path_params,
                }));
            }
            if key_lower.contains("mastery") {
                any_mastery_paths.push(serde_json::json!({
                    "path": path_key,
                    "path_parameter_names": path_params,
                }));
            }
            if key_lower.contains("collections") {
                collections_paths.push(serde_json::json!({
                    "path": path_key,
                    "path_parameter_names": path_params,
                }));
            }
        }
        return Ok(serde_json::json!({
            "spec_url": spec_path,
            "swagger_fetch": swagger_status,
            "paths_containing_champion_mastery": mastery_paths,
            "paths_containing_mastery": any_mastery_paths,
            "paths_containing_collections": collections_paths,
        }));
    }
    Err(format!(
        "Could not load swagger (tried v2 and v3). Details: {}",
        serde_json::to_string(&swagger_status).unwrap_or_else(|_| "[]".to_string())
    ))
}

/// Debug: same flow as get_my_champion_mastery but returns every path we try and the LCU response.
/// Run in devtools: window.__TAURI__.core.invoke('get_lcu_mastery_debug').then(console.log).catch(console.error)
#[tauri::command]
pub async fn get_lcu_mastery_debug() -> Result<serde_json::Value, String> {
    let creds = credentials::discover_credentials()
        .ok_or_else(|| "League Client not found.".to_string())?;

    let (status, body) = client::lcu_get(&creds, "/lol-summoner/v1/current-summoner")
        .await
        .map_err(|e| format!("LCU request failed: {}", e))?;
    if status != 200 || body.is_empty() || body == "null" {
        return Ok(serde_json::json!({
            "error": "current-summoner not available",
            "status": status,
            "attempts": []
        }));
    }

    let summoner: CurrentSummonerResponse =
        serde_json::from_str(&body).map_err(|e| format!("Invalid current-summoner JSON: {}", e))?;

    let mut ids: Vec<String> = vec![];
    if summoner.summoner_id != 0 {
        ids.push(summoner.summoner_id.to_string());
    }
    if let Some(ref p) = summoner.puuid {
        let s = p.trim();
        if !s.is_empty() {
            ids.push(s.to_string());
        }
    }
    ids.push("current-summoner".to_string());

    // Probe: try other inventory endpoints to see if only champion-mastery fails or all inventory-by-id fail
    let mut probe: Vec<serde_json::Value> = vec![];
    let probe_paths: Vec<String> = vec![
        "/lol-collections/v1/inventories/chest-eligibility".to_string(),
        format!("/lol-collections/v1/inventories/{}/runes", summoner.summoner_id),
        format!("/lol-collections/v1/inventories/{}/backdrop", summoner.summoner_id),
    ];
    for probe_path in probe_paths {
        let res = client::lcu_get(&creds, &probe_path).await;
        let (status, body_preview) = match &res {
            Ok((s, b)) => (*s, if b.len() > 150 { format!("{}...", &b[..150]) } else { b.clone() }),
            Err(e) => (0, e.clone()),
        };
        probe.push(serde_json::json!({ "path": probe_path, "status": status, "body_preview": body_preview }));
    }

    const BASES: &[&str] = &[
        "/lol-collections/v1/inventories/{}/champion-mastery",
        "/lol-collections/v1/inventories/{}/champion-mastery/top",
        "/lol-champions/v1/inventories/{}/champion-mastery",
        "/lol-champions/v1/inventories/{}/champion-mastery/top",
    ];

    let mut attempts: Vec<serde_json::Value> = vec![];
    for id in &ids {
        for base in BASES {
            let path = base.replace("{}", id);
            let full_url = format!("https://127.0.0.1:{}{}", creds.port, path);
            let res = client::lcu_get(&creds, &path).await;
            let (status, body_preview) = match &res {
                Ok((s, b)) => (*s, {
                    if b.len() > 300 {
                        format!("{}...", &b[..300])
                    } else {
                        b.clone()
                    }
                }),
                Err(e) => (0, e.clone()),
            };
            attempts.push(serde_json::json!({
                "path": path,
                "url": full_url,
                "id_used": id,
                "status": status,
                "body_preview": body_preview,
            }));
        }
    }

    Ok(serde_json::json!({
        "current_summoner": {
            "summoner_id": summoner.summoner_id,
            "puuid": summoner.puuid,
        },
        "ids_tried": ids,
        "port": creds.port,
        "probe_other_inventory_endpoints": probe,
        "attempts": attempts,
    }))
}

/// Returns current user's champion mastery from the LCU.
/// Tries inventory paths (summonerId, PUUID, current-summoner), then end-of-game updates.
/// Returns empty vec if not connected or if this client build does not expose mastery (some
/// builds expose e.g. backdrop but not champion-mastery/runes).
#[tauri::command]
pub async fn get_my_champion_mastery() -> Result<Vec<ChampionMasteryEntry>, String> {
    let creds = credentials::discover_credentials()
        .ok_or_else(|| "League Client not found.".to_string())?;

    let (status, body) = client::lcu_get(&creds, "/lol-summoner/v1/current-summoner")
        .await
        .map_err(|e| format!("LCU request failed: {}", e))?;
    if status != 200 || body.is_empty() || body == "null" {
        return Ok(vec![]);
    }

    let summoner: CurrentSummonerResponse =
        serde_json::from_str(&body).map_err(|e| format!("Invalid current-summoner JSON: {}", e))?;

    // IDs to try: summonerId, PUUID, then literal "current-summoner".
    let mut ids: Vec<String> = vec![];
    if summoner.summoner_id != 0 {
        ids.push(summoner.summoner_id.to_string());
    }
    if let Some(ref p) = summoner.puuid {
        let s = p.trim();
        if !s.is_empty() {
            ids.push(s.to_string());
        }
    }
    ids.push("current-summoner".to_string());

    const BASES: &[&str] = &[
        "/lol-collections/v1/inventories/{}/champion-mastery",
        "/lol-collections/v1/inventories/{}/champion-mastery/top",
        "/lol-champions/v1/inventories/{}/champion-mastery",
        "/lol-champions/v1/inventories/{}/champion-mastery/top",
    ];

    for id in &ids {
        for base in BASES {
            let path = base.replace("{}", id);
            match client::lcu_get(&creds, &path).await {
                Ok((200, resp)) if !resp.is_empty() && resp != "null" => {
                    let raw: Vec<LcuChampionMasteryEntry> =
                        serde_json::from_str(&resp).map_err(|e| format!("Invalid mastery JSON: {}", e))?;
                    return Ok(raw
                        .into_iter()
                        .map(|e| ChampionMasteryEntry {
                            champion_id: e.champion_id,
                            champion_level: e.champion_level,
                            champion_points: e.champion_points,
                        })
                        .collect());
                }
                Ok((200, _)) => return Ok(vec![]),
                _ => {}
            }
        }
    }

    // Fallback: end-of-game mastery updates (may be empty or different shape; only populated after a game).
    if let Ok((200, resp)) = client::lcu_get(&creds, "/lol-end-of-game/v1/champion-mastery-updates").await {
        if !resp.is_empty() && resp != "null" {
            if let Ok(raw) = serde_json::from_str::<Vec<LcuChampionMasteryEntry>>(&resp) {
                if !raw.is_empty() {
                    return Ok(raw
                        .into_iter()
                        .map(|e| ChampionMasteryEntry {
                            champion_id: e.champion_id,
                            champion_level: e.champion_level,
                            champion_points: e.champion_points,
                        })
                        .collect());
                }
            }
        }
    }

    // This client build does not expose champion-mastery (same ID works for e.g. backdrop).
    // Return empty so the app can still run; UI can show "No mastery data" or use Riot API with API key.
    Ok(vec![])
}

