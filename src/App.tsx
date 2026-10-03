import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";

type AppMode = "Active" | "Focus" | "Break" | "Assessment" | "Paused";

type ChatMessage = {
  role: "user" | "assistant";
  content: string;
};

export default function App() {
  const [mode, setMode] = useState<AppMode>("Active");
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [inputText, setInputText] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  
  const messagesEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    invoke<AppMode>("get_app_mode").then(setMode).catch(console.error);
  }, []);

  useEffect(() => {
    // Scroll to bottom when messages change
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages, isLoading]);

  const changeMode = async (newMode: AppMode) => {
    try {
      await invoke("set_app_mode", { mode: newMode });
      setMode(newMode);
    } catch (e) {
      console.error("Failed to set app mode:", e);
    }
  };

  const handleSendMessage = async () => {
    if (!inputText.trim() || isLoading) return;

    const userMessage: ChatMessage = { role: "user", content: inputText.trim() };
    setMessages((prev) => [...prev, userMessage]);
    setInputText("");
    setIsLoading(true);
    setError(null);

    try {
      const response = await invoke<string>("send_message", { message: userMessage.content });
      setMessages((prev) => [...prev, { role: "assistant", content: response }]);
    } catch (err) {
      console.error("Chat Error:", err);
      // Determine user-friendly error message
      let errMsg = "An unexpected error occurred.";
      if (typeof err === "string") {
        if (err.includes("GEMINI_API_KEY")) {
          errMsg = "NOVA needs a Gemini API key to talk. Please configure it in the .env file.";
        } else if (err.includes("Network")) {
          errMsg = "Network error. Please check your connection.";
        } else {
          errMsg = "AI Provider Error. The service might be temporarily unavailable.";
        }
      }
      setError(errMsg);
      // Optional: remove user message or leave it
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="flex flex-col h-screen bg-zinc-900 text-zinc-100 font-sans antialiased overflow-hidden selection:bg-zinc-700">
      
      {/* Header */}
      <header className="flex items-center justify-between px-4 py-3 border-b border-zinc-800 bg-zinc-900/80 backdrop-blur-md z-10 shrink-0">
        <div className="flex items-center space-x-2">
          <div className="w-2.5 h-2.5 rounded-full bg-emerald-500 shadow-[0_0_8px_rgba(16,185,129,0.5)]"></div>
          <h1 className="font-semibold text-zinc-100 tracking-wide text-sm">NOVA</h1>
        </div>
        <div className="flex items-center space-x-2">
          <span className="text-xs font-medium px-2.5 py-1 rounded-md bg-zinc-800 text-zinc-300">
            {mode}
          </span>
        </div>
      </header>

      {/* Main Content Area */}
      <main className="flex-1 overflow-y-auto p-4 flex flex-col gap-6">
        
        {/* Only show greeting and statuses if no messages exist yet to keep it clean */}
        {messages.length === 0 && (
          <div className="space-y-6">
            <section className="space-y-1">
              <h2 className="text-xl font-medium text-zinc-100">Good morning.</h2>
              <p className="text-sm text-zinc-400">Your systems are running optimally.</p>
            </section>

            <section className="grid grid-cols-2 gap-3">
              <div className="bg-zinc-800/50 p-3 rounded-lg border border-zinc-700/50 flex flex-col justify-between h-20">
                <span className="text-xs font-semibold text-zinc-400">Focus Session</span>
                <span className="text-sm font-medium text-zinc-300">Inactive</span>
              </div>
              <div className="bg-zinc-800/50 p-3 rounded-lg border border-zinc-700/50 flex flex-col justify-between h-20">
                <span className="text-xs font-semibold text-zinc-400">Drowsiness Engine</span>
                <span className="text-sm font-medium text-zinc-500">Offline</span>
              </div>
            </section>

            <section className="pt-2">
              <button 
                onClick={() => changeMode("Assessment")}
                className="w-full py-2 px-4 rounded-lg bg-red-900/30 text-red-400 text-sm font-medium border border-red-900/50 hover:bg-red-900/50 transition-colors"
              >
                Enter Assessment Mode
              </button>
            </section>
          </div>
        )}

        {/* Chat Messages */}
        <div className="flex-1 space-y-4 flex flex-col justify-end">
          {messages.map((msg, idx) => (
            <div key={idx} className={`flex flex-col ${msg.role === "user" ? "items-end" : "items-start"}`}>
              <div className={`max-w-[85%] rounded-xl px-4 py-2.5 text-sm ${
                msg.role === "user" 
                  ? "bg-zinc-700 text-zinc-100 rounded-br-sm" 
                  : "bg-zinc-800/80 border border-zinc-700/50 text-zinc-300 rounded-bl-sm"
              }`}>
                {msg.content}
              </div>
            </div>
          ))}
          
          {isLoading && (
            <div className="flex flex-col items-start">
              <div className="bg-zinc-800/80 border border-zinc-700/50 rounded-xl rounded-bl-sm px-4 py-2.5 text-sm text-zinc-500 animate-pulse">
                Thinking...
              </div>
            </div>
          )}

          {error && (
            <div className="bg-red-900/20 border border-red-900/50 rounded-xl p-3 mt-2 text-sm text-red-400">
              {error}
            </div>
          )}
          
          <div ref={messagesEndRef} />
        </div>

      </main>

      {/* Footer / Chat Input */}
      <footer className="p-4 border-t border-zinc-800 bg-zinc-900 shrink-0">
        <div className="relative">
          <input 
            type="text" 
            value={inputText}
            onChange={(e) => setInputText(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && handleSendMessage()}
            placeholder="Message NOVA..." 
            disabled={isLoading}
            className="w-full bg-zinc-800 border border-zinc-700 text-sm text-zinc-200 rounded-lg py-3 pl-4 pr-12 focus:outline-none focus:border-zinc-500 focus:ring-1 focus:ring-zinc-500 disabled:opacity-50 transition-all placeholder:text-zinc-500"
          />
          <button 
            onClick={handleSendMessage}
            disabled={!inputText.trim() || isLoading}
            className="absolute right-2 top-1/2 -translate-y-1/2 p-2 text-zinc-400 hover:text-zinc-200 disabled:opacity-50 disabled:cursor-not-allowed transition-colors rounded-md hover:bg-zinc-700"
          >
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="m22 2-7 20-4-9-9-4Z"/><path d="M22 2 11 13"/></svg>
          </button>
        </div>
      </footer>

    </div>
  );
}
