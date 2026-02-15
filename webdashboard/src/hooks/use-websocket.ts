"use client";

import { useEffect, useRef, useState } from "react";
import { WS_BASE_URL } from "@/lib/api";

export type ReportStreamMessage = 
  | { type: "CHUNK"; batch_index: number; total_batches: number; data: string }
  | { type: "DONE"; total_batches: number; total_records: number }
  | { type: "ERROR"; message: string; batch_index: number | null };

export type WsMessage = 
  | { type: "handshake"; client_type: "dashboard" | "agent" }
  | ReportStreamMessage
  | { type: string; [key: string]: any }; // Fallback for other message types

export function useWebSocket() {
  const [isConnected, setIsConnected] = useState(false);
  const [lastMessage, setLastMessage] = useState<WsMessage | null>(null);
  const socketRef = useRef<WebSocket | null>(null);

  useEffect(() => {
    const socket = new WebSocket(WS_BASE_URL);
    socketRef.current = socket;

    socket.onopen = () => {
      setIsConnected(true);
      console.log("Connected to API Server WebSocket");

      // Handshake cho Dashboard (nếu cần phân biệt với Agent)
      const handshake: WsMessage = {
        type: "handshake",
        client_type: "dashboard",
      };
      socket.send(JSON.stringify(handshake));
    };

    socket.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data) as WsMessage;
        setLastMessage(data);
      } catch (e) {
        console.error("Failed to parse WS message", e);
      }
    };

    socket.onclose = () => {
      setIsConnected(false);
      console.log("Disconnected from API Server WebSocket");
    };

    return () => {
      socket.close();
    };
  }, []);

  const sendMessage = (msg: WsMessage) => {
    if (socketRef.current?.readyState === WebSocket.OPEN) {
      socketRef.current.send(JSON.stringify(msg));
    }
  };

  return { isConnected, lastMessage, sendMessage };
}
