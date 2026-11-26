import { useCallback, useEffect, useRef, useState } from "react";
import { commands, subscribeFrames } from "../ipc";
import { formatError } from "../lib/format";
import { isFrameSnapshot, type FrameSnapshot } from "../types";

export type LoadState = "loading" | "ready" | "error";
export type NoticeKind = "ok" | "error";

export interface Notice {
  kind: NoticeKind;
  text: string;
}

export function useKinglight() {
  const [frame, setFrame] = useState<FrameSnapshot | null>(null);
  const [loadState, setLoadState] = useState<LoadState>("loading");
  const [loadError, setLoadError] = useState<string | null>(null);
  const [notice, setNotice] = useState<Notice | null>(null);
  const [busy, setBusy] = useState<string | null>(null);
  const [shows, setShows] = useState<string[]>([]);

  const mountedRef = useRef(true);
  const pendingRef = useRef(new Map<string, number>());

  useEffect(() => {
    mountedRef.current = true;
    return () => {
      mountedRef.current = false;
    };
  }, []);

  const adopt = useCallback((snapshot: FrameSnapshot) => {
    if (!mountedRef.current) return;
    setFrame(snapshot);
    setLoadState("ready");
    setLoadError(null);
  }, []);

  const notify = useCallback((kind: NoticeKind, text: string) => {
    if (!mountedRef.current) return;
    setNotice({ kind, text });
  }, []);

  const clearNotice = useCallback(() => {
    if (mountedRef.current) setNotice(null);
  }, []);

  const markBusy = useCallback((key: string) => {
    pendingRef.current.set(key, (pendingRef.current.get(key) ?? 0) + 1);
    setBusy(key);
  }, []);

  const unmarkBusy = useCallback((key: string) => {
    const pending = pendingRef.current;
    const count = pending.get(key) ?? 0;
    if (count <= 1) pending.delete(key);
    else pending.set(key, count - 1);
    const next = pending.keys().next();
    setBusy(next.done ? null : next.value);
  }, []);

  const refreshShows = useCallback(async () => {
    try {
      const list = await commands.listShows();
      if (mountedRef.current) setShows(list);
    } catch (error) {
      if (mountedRef.current) notify("error", `Show list failed: ${formatError(error)}`);
    }
  }, [notify]);

  useEffect(() => {
    let cancelled = false;
    let unsubscribe: (() => void) | undefined;

    const connect = async () => {
      try {
        unsubscribe = await subscribeFrames((snapshot) => {
          if (!cancelled) adopt(snapshot);
        });
        if (cancelled) {
          unsubscribe();
          return;
        }
        const snapshot = await commands.getSnapshot();
        if (cancelled) return;
        adopt(snapshot);
        await refreshShows();
      } catch (error) {
        if (!cancelled) {
          setLoadError(formatError(error));
          setLoadState("error");
        }
      }
    };

    void connect();

    return () => {
      cancelled = true;
      unsubscribe?.();
    };
  }, [adopt, refreshShows]);

  const run = useCallback(
    async <T,>(key: string, action: () => Promise<T>): Promise<T | undefined> => {
      markBusy(key);
      try {
        const result = await action();
        if (!mountedRef.current) return undefined;
        if (isFrameSnapshot(result)) adopt(result);
        return result;
      } catch (error) {
        if (mountedRef.current) notify("error", formatError(error));
        return undefined;
      } finally {
        if (mountedRef.current) unmarkBusy(key);
      }
    },
    [adopt, markBusy, notify, unmarkBusy],
  );

  const retryConnection = useCallback(() => {
    window.location.reload();
  }, []);

  return {
    frame,
    loadState,
    loadError,
    notice,
    busy,
    shows,
    run,
    notify,
    clearNotice,
    refreshShows,
    retryConnection,
  };
}
