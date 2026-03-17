import { useLcuSync } from "../hooks/useLcuSync";
import { useChampionPool } from "../hooks/useChampionPool";
import { Wifi, WifiOff, Swords, Shield, Clock, Award } from "lucide-react";
import "./LiveDraftDashboard.css";

export function LiveDraftDashboard() {
  const { status, draft, error } = useLcuSync();
  const { topByMastery, masteryError, masterdataError } = useChampionPool(
    status?.connected ?? false
  );

  return (
    <div className="live-draft">
      <header className="live-draft__header">
        <h1 className="live-draft__title">Maximum League</h1>
        <div className="live-draft__connection" aria-live="polite">
          {error && (
            <span className="live-draft__error" title={error}>
              Error
            </span>
          )}
          {status?.connected ? (
            <span className="live-draft__badge live-draft__badge--connected">
              <Wifi size={16} aria-hidden />
              Connected
              {status.port != null && (
                <span className="live-draft__port"> :{status.port}</span>
              )}
            </span>
          ) : (
            <span className="live-draft__badge live-draft__badge--disconnected">
              <WifiOff size={16} aria-hidden />
              Not connected
            </span>
          )}
        </div>
      </header>

      {draft?.in_champ_select ? (
        <section className="live-draft__session">
          <div className="live-draft__phase">
            <Clock size={18} aria-hidden />
            <span>{draft.phase || "Champ select"}</span>
            {draft.time_left_sec != null && (
              <span className="live-draft__timer">
                {Math.floor(draft.time_left_sec / 60)}:
                {String(draft.time_left_sec % 60).padStart(2, "0")}
              </span>
            )}
          </div>

          <div className="live-draft__teams">
            <div className="live-draft__team">
              <h3 className="live-draft__team-title">
                <Shield size={18} aria-hidden /> My team
              </h3>
              <div className="live-draft__champs">
                <div className="live-draft__row">
                  <span className="live-draft__label">Bans</span>
                  <span className="live-draft__ids">
                    {draft.my_team_bans.length > 0
                      ? draft.my_team_bans.join(", ")
                      : "—"}
                  </span>
                </div>
                <div className="live-draft__row">
                  <span className="live-draft__label">Picks</span>
                  <span className="live-draft__ids">
                    {draft.my_team_picks.length > 0
                      ? draft.my_team_picks.join(", ")
                      : "—"}
                  </span>
                </div>
              </div>
            </div>
            <div className="live-draft__team">
              <h3 className="live-draft__team-title">
                <Swords size={18} aria-hidden /> Their team
              </h3>
              <div className="live-draft__champs">
                <div className="live-draft__row">
                  <span className="live-draft__label">Bans</span>
                  <span className="live-draft__ids">
                    {draft.their_team_bans.length > 0
                      ? draft.their_team_bans.join(", ")
                      : "—"}
                  </span>
                </div>
                <div className="live-draft__row">
                  <span className="live-draft__label">Picks</span>
                  <span className="live-draft__ids">
                    {draft.their_team_picks.length > 0
                      ? draft.their_team_picks.join(", ")
                      : "—"}
                  </span>
                </div>
              </div>
            </div>
          </div>
        </section>
      ) : (
        <section className="live-draft__idle" aria-live="polite">
          <p>
            {status?.connected
              ? "Not in champ select. Enter a queue to see live draft."
              : "Start the League client and log in to connect."}
          </p>
        </section>
      )}

      <section className="live-draft__pool">
        <h2 className="live-draft__pool-title">
          <Award size={18} aria-hidden />
          Your champion pool (by mastery)
        </h2>
        {masterdataError && (
          <p className="live-draft__error">Champion list: {masterdataError}</p>
        )}
        {masteryError && status?.connected && (
          <p className="live-draft__error">Mastery: {masteryError}</p>
        )}
        {topByMastery.length > 0 ? (
          <ul className="live-draft__pool-list">
            {topByMastery.map((c, i) => (
              <li key={i} className="live-draft__pool-item">
                <span className="live-draft__pool-name">{c.name}</span>
                <span className="live-draft__pool-meta">
                  M{c.level} · {c.points.toLocaleString()} pts
                </span>
              </li>
            ))}
          </ul>
        ) : (
          <p className="live-draft__pool-empty">
            {status?.connected
              ? "No mastery data yet."
              : "Connect to League client to see your pool."}
          </p>
        )}
      </section>
    </div>
  );
}
