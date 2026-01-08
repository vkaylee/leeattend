"use client";

import { useEffect, useRef, useState } from "react";
import { SOCKET_URL } from "@/lib/api";

export function useWebSocket() {
  const [isConnected, setIsConnected] = useState(false);
  const [lastMessage, setLastMessage] = useState<any>(null);
  const socketRef = useRef<WebSocket | null>(null);

  useEffect(() => {
    const socket = new WebSocket(SOCKET_URL);
    socketRef.current = socket;

    socket.onopen = () => {
      setIsConnected(true);
      console.log("Connected to API Server WebSocket");

      // Handshake cho Dashboard (nếu cần phân biệt với Agent)
      socket.send(
        JSON.stringify({
          type: "handshake",
          client_type: "dashboard",
        }),
      );
    };

    socket.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data);
        setLastMessage(data);
      } catch (e) {
        console.error("Failed to parse WS message", e);
      }
    };

    socket.onclose = () => {
      setIsConnected(false);
      console.log("Disconnected from API Server WebSocket");
      // Thử kết nối lại sau 5 giây
      setTimeout(() => {
        // logic reconnect có thể thêm ở đây
      }, 5000);
    };

    return () => {
      socket.close();
    };
  }, []);

  const sendMessage = (msg: any) => {
    if (socketRef.current?.readyState === WebSocket.OPEN) {
      socketRef.current.send(JSON.stringify(msg));
    }
  };

  return { isConnected, lastMessage, sendMessage };
}
