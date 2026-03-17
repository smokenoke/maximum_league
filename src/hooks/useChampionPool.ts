import { useState, useEffect, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { ChampionMasterdata, ChampionMasteryEntry } from "../types/lcu";

const MASTERY_POLL_MS = 30_000;

export function useChampionPool(connected: boolean) {
  const [champions, setChampions] = useState<ChampionMasterdata[]>([]);
  const [mastery, setMastery] = useState<ChampionMasteryEntry[]>([]);
  const [masterdataError, setMasterdataError] = useState<string | null>(null);
  const [masteryError, setMasteryError] = useState<string | null>(null);

  // Load champion masterdata once
  useEffect(() => {
    let cancelled = false;
    setMasterdataError(null);
    invoke<ChampionMasterdata[]>("get_champion_masterdata")
      .then((data) => {
        if (!cancelled) setChampions(data);
      })
      .catch((e) => {
        if (!cancelled) setMasterdataError(String(e));
      });
    return () => {
      cancelled = true;
    };
  }, []);

  // Load mastery when connected, then poll
  useEffect(() => {
    if (!connected) {
      setMastery([]);
      setMasteryError(null);
      return;
    }
    let cancelled = false;
    const load = () => {
      invoke<ChampionMasteryEntry[]>("get_my_champion_mastery")
        .then((data) => {
          if (!cancelled) setMastery(data);
          if (!cancelled) setMasteryError(null);
        })
        .catch((e) => {
          if (!cancelled) setMasteryError(String(e));
        });
    };
    load();
    const t = setInterval(load, MASTERY_POLL_MS);
    return () => {
      cancelled = true;
      clearInterval(t);
    };
  }, [connected]);

  const championById = useMemo(() => {
    const m = new Map<number, ChampionMasterdata>();
    for (const c of champions) m.set(c.id, c);
    return m;
  }, [champions]);

  const topByMastery = useMemo(() => {
    const sorted = [...mastery].sort((a, b) => b.championPoints - a.championPoints);
    return sorted.slice(0, 15).map((e) => ({
      name: championById.get(e.championId)?.name ?? `#${e.championId}`,
      level: e.championLevel,
      points: e.championPoints,
    }));
  }, [mastery, championById]);

  return {
    champions,
    championById,
    mastery,
    topByMastery,
    masterdataError,
    masteryError,
  };
}
