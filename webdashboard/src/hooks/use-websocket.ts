"use client";

import { useEffect, useRef, useState } from "react";

import { WS_BASE_URL } from "@/lib/api";
import { logError } from "@/lib/logger";

export type ReportStreamMessage =
  | { type: "CHUNK"; batch_index: number; total_batches: number; data: string }
  | { type: "DONE"; total_batches: number; total_records: number }
  | { type: "ERROR"; message: string; batch_index: number | null };

export type WsMessage =
  | { type: "handshake"; client_type: "dashboard" | "agent" }
  | ReportStreamMessage
  | { type: string; [key: string]: unknown }; // Fallback for other message types

export function useWebSocket() {
  const [isConnected, setIsConnected] = useState(false);
  const [lastMessage, setLastMessage] = useState<WsMessage | null>(null);
  const socketRef = useRef<WebSocket | null>(null);
  const pingIntervalRef = useRef<NodeJS.Timeout | null>(null);

  useEffect(() => {
    const socket = new WebSocket(WS_BASE_URL);
    socketRef.current = socket;

    socket.onopen = () => {
      setIsConnected(true);


      // Handshake cho Dashboard (nếu cần phân biệt với Agent)
      const handshake: WsMessage = {
        type: "handshake",
        client_type: "dashboard",
      };
      socket.send(JSON.stringify(handshake));

      // Bật heartbeat ping mỗi 30s để giữ kết nối qua proxy/Cloudflare
      pingIntervalRef.current = setInterval(() => {
        if (socket.readyState === WebSocket.OPEN) {
          socket.send(JSON.stringify({ type: "ping" }));
        }
      }, 30000);
    };

    socket.onmessage = (event: MessageEvent) => {
      try {
        const data = JSON.parse(event.data as string) as WsMessage;
        setLastMessage(data);
      } catch (e: unknown) {
        logError(e, { component: "WebSocket" });
        setLastMessage(null);
      }
    };

    socket.onclose = () => {
      setIsConnected(false);

    };

    return () => {
      if (pingIntervalRef.current) {
        clearInterval(pingIntervalRef.current);
        pingIntervalRef.current = null;
      }
      socket.close();
    };
  }, []);

  const sendMessage = (message: WsMessage) => {
    if (socketRef.current?.readyState === WebSocket.OPEN) {
      socketRef.current.send(JSON.stringify(message));
    }
  };

  return { isConnected, lastMessage, sendMessage };
}
