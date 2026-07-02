export const RELAY_WS_URL: string =
  (import.meta.env.VITE_RELAY_URL as string | undefined) ?? "ws://localhost:3001/ws";

export const RELAY_HTTP_URL: string =
  (import.meta.env.VITE_RELAY_HTTP_URL as string | undefined) ?? "http://localhost:3001";
