import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { LcuStatus, LiveDraftState } from "../types/lcu";

const POLL_MS = 1500;

export function useLcuSync() {
  const [status, setStatus] = useState<LcuStatus | null>(null);
  const [draft, setDraft] = useState<LiveDraftState | null>(null);
  const [error, setError] = useState<string | null>(null);

  const fetchStatus = useCallback(async () => {
    try {
      const result = (await invoke("get_lcu_status")) as LcuStatus;
      setStatus(result);
      setError(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
      setStatus(null);
    }
  }, []);

  const fetchDraft = useCallback(async () => {
    try {
      const result = (await invoke("get_live_draft_state")) as LiveDraftState;
      setDraft(result);
    } catch {
      setDraft(null);
    }
  }, []);

  useEffect(() => {
    void fetchStatus();
    void fetchDraft();
    const t = setInterval(() => {
      void fetchStatus();
      void fetchDraft();
    }, POLL_MS);
    return () => clearInterval(t);
  }, [fetchStatus, fetchDraft]);

  return { status, draft, error };
}
