/* eslint-disable */
import React, { useState, useRef } from "react";

import Button from "@/components/ui/button";

interface ESignatureModalProps {
  documentTitle: string;
  isOpen: boolean;
  onClose: () => void;
  onSign: (payload: { signature_image_base64: string; ip_address?: string; user_agent?: string }) => void;
}

export function ESignatureModal({ isOpen, onClose, onSign, documentTitle }: ESignatureModalProps) {
  const [activeTab, setActiveTab] = useState<"draw" | "type">("draw");
  const [typedName, setTypedName] = useState("");
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [isDrawing, setIsDrawing] = useState(false);

  if (!isOpen) return null;

  // Xử lý sự kiện vẽ trên Canvas
  const startDrawing = (e: React.MouseEvent<HTMLCanvasElement> | React.TouchEvent<HTMLCanvasElement>) => {
    setIsDrawing(true);
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const rect = canvas.getBoundingClientRect();
    const clientX = "touches" in e ? e.touches[0].clientX : e.clientX;
    const clientY = "touches" in e ? e.touches[0].clientY : e.clientY;

    ctx.beginPath();
    ctx.moveTo(clientX - rect.left, clientY - rect.top);
  };

  const draw = (e: React.MouseEvent<HTMLCanvasElement> | React.TouchEvent<HTMLCanvasElement>) => {
    if (!isDrawing) return;
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const rect = canvas.getBoundingClientRect();
    const clientX = "touches" in e ? e.touches[0].clientX : e.clientX;
    const clientY = "touches" in e ? e.touches[0].clientY : e.clientY;

    ctx.lineTo(clientX - rect.left, clientY - rect.top);
    ctx.stroke();
  };

  const stopDrawing = () => {
    setIsDrawing(false);
  };

  const clearCanvas = () => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;
    ctx.clearRect(0, 0, canvas.width, canvas.height);
  };

  const handleSign = () => {
    let signatureBase64 = "";

    if (activeTab === "draw") {
      const canvas = canvasRef.current;
      if (canvas) {
        signatureBase64 = canvas.toDataURL("image/png");
      }
    } else {
      // Simulate typing signature as base64 (In a real app, we'd generate an image from text)
      // For this implementation, we just encode the text as a placeholder.
      signatureBase64 = `text:${btoa(typedName)}`;
    }

    onSign({
      signature_image_base64: signatureBase64,
      user_agent: navigator.userAgent,
      // IP is usually grabbed by the backend, but we send what we can
    });
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm transition-opacity">
      <div className="bg-white dark:bg-slate-900 w-full max-w-lg rounded-2xl shadow-2xl overflow-hidden animate-in fade-in zoom-in-95 duration-200">
        
        {/* Header */}
        <div className="px-6 py-4 border-b border-slate-200 dark:border-slate-800 flex justify-between items-center bg-slate-50/50 dark:bg-slate-800/50">
          <div>
            <h2 className="text-xl font-semibold text-slate-900 dark:text-white">Sign Document</h2>
            <p className="text-sm text-slate-500 dark:text-slate-400 mt-1 truncate max-w-[300px]">
              {documentTitle}
            </p>
          </div>
          <Button 
            className="text-slate-400 hover:text-slate-600 dark:hover:text-slate-200 transition-colors p-2 rounded-full hover:bg-slate-100 dark:hover:bg-slate-800 h-9 w-9"
            onClick={onClose}
            size="icon"
            variant="ghost"
          >
            <svg fill="none" height="20" stroke="currentColor" strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" viewBox="0 0 24 24" width="20" xmlns="http://www.w3.org/2000/svg"><path d="M18 6 6 18"/><path d="m6 6 12 12"/></svg>
          </Button>
        </div>

        {/* Body */}
        <div className="p-6">
          {/* Tabs */}
          <div className="flex space-x-1 bg-slate-100 dark:bg-slate-800/50 p-1 rounded-xl mb-6">
            <Button
              className={`flex-1 py-2 text-sm font-medium rounded-lg transition-all duration-200 h-auto font-medium hover:bg-transparent ${
                activeTab === "draw" 
                  ? "bg-white dark:bg-slate-700 text-blue-600 dark:text-blue-400 shadow-sm hover:bg-white dark:hover:bg-slate-700" 
                  : "text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-white"
              }`}
              onClick={() => { setActiveTab("draw"); }}
              variant="ghost"
            >
              Draw Signature
            </Button>
            <Button
              className={`flex-1 py-2 text-sm font-medium rounded-lg transition-all duration-200 h-auto font-medium hover:bg-transparent ${
                activeTab === "type" 
                  ? "bg-white dark:bg-slate-700 text-blue-600 dark:text-blue-400 shadow-sm hover:bg-white dark:hover:bg-slate-700" 
                  : "text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-white"
              }`}
              onClick={() => { setActiveTab("type"); }}
              variant="ghost"
            >
              Type Signature
            </Button>
          </div>

          {/* Draw Tab */}
          {activeTab === "draw" && (
            <div className="space-y-3">
              <div className="relative border-2 border-dashed border-slate-300 dark:border-slate-700 rounded-xl bg-slate-50 dark:bg-slate-900/50 overflow-hidden group">
                <canvas
                  className="w-full h-[200px] cursor-crosshair touch-none"
                  height={200}
                  onMouseDown={startDrawing}
                  onMouseLeave={stopDrawing}
                  onMouseMove={draw}
                  onMouseUp={stopDrawing}
                  onTouchEnd={stopDrawing}
                  onTouchMove={draw}
                  onTouchStart={startDrawing}
                  ref={canvasRef}
                  width={400}
                />
                <div className="absolute bottom-3 right-3 opacity-0 group-hover:opacity-100 transition-opacity">
                  <Button 
                    className="text-xs px-3 py-1.5 rounded-md shadow-sm text-slate-600 dark:text-slate-300 hover:bg-slate-50 dark:hover:bg-slate-700 transition-colors h-auto font-normal border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-800"
                    onClick={clearCanvas}
                    variant="outline"
                  >
                    Clear Canvas
                  </Button>
                </div>
              </div>
              <p className="text-xs text-center text-slate-500">
                Use your mouse or finger to draw your signature inside the box.
              </p>
            </div>
          )}

          {/* Type Tab */}
          {activeTab === "type" && (
            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-2">
                  Your Full Name
                </label>
                <input
                  className="w-full px-4 py-3 bg-slate-50 dark:bg-slate-900/50 border border-slate-200 dark:border-slate-700 rounded-xl focus:ring-2 focus:ring-blue-500/20 focus:border-blue-500 transition-all outline-none text-slate-900 dark:text-white"
                  onChange={(e) => { setTypedName(e.target.value); }}
                  placeholder="e.g. John Doe"
                  type="text"
                  value={typedName}
                />
              </div>
              
              <div className="h-[120px] flex items-center justify-center border border-slate-200 dark:border-slate-800 rounded-xl bg-slate-50/50 dark:bg-slate-900/30">
                {typedName ? (
                  <span className="text-4xl font-serif text-slate-800 dark:text-slate-200 italic opacity-80" style={{ fontFamily: "'Caveat', 'Dancing Script', cursive" }}>
                    {typedName}
                  </span>
                ) : (
                  <span className="text-sm text-slate-400">Preview will appear here</span>
                )}
              </div>
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="px-6 py-4 border-t border-slate-100 dark:border-slate-800 bg-slate-50/30 dark:bg-slate-900/30 flex justify-end gap-3">
          <Button
            className="px-5 py-2.5 text-sm font-medium text-slate-600 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-slate-800 rounded-lg transition-colors h-auto font-normal"
            onClick={onClose}
            variant="ghost"
          >
            Cancel
          </Button>
          <Button
            className="px-6 py-2.5 text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 rounded-lg shadow-sm shadow-blue-500/20 transition-all disabled:opacity-50 disabled:cursor-not-allowed active:scale-[0.98] h-auto font-normal"
            disabled={activeTab === "type" && !typedName.trim()}
            onClick={handleSign}
          >
            Sign & Accept
          </Button>
        </div>

      </div>
    </div>
  );
}
