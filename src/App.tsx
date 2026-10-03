import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

// Match the Rust AppMode enum
type AppMode = "Active" | "Focus" | "Break" | "Assessment" | "Paused";

export default function App() {
  const [mode, setMode] = useState<AppMode>("Active");

  useEffect(() => {
    // Fetch initial state
    invoke<AppMode>("get_app_mode").then(setMode).catch(console.error);
  }, []);

  const changeMode = async (newMode: AppMode) => {
    try {
      await invoke("set_app_mode", { mode: newMode });
      setMode(newMode);
    } catch (e) {
      console.error("Failed to set app mode:", e);
    }
  };

  return (
    <div className="flex flex-col h-screen bg-zinc-900 text-zinc-100 font-sans antialiased overflow-hidden selection:bg-zinc-700">
      
      {/* Header / Identity */}
      <header className="flex items-center justify-between px-4 py-3 border-b border-zinc-800 bg-zinc-900/80 backdrop-blur-md">
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
      <main className="flex-1 overflow-y-auto p-4 space-y-6">
        
        {/* Greeting & Status */}
        <section className="space-y-1">
          <h2 className="text-xl font-medium text-zinc-100">Good morning.</h2>
          <p className="text-sm text-zinc-400">Your systems are running optimally.</p>
        </section>

        {/* Task Summary Placeholder */}
        <section className="bg-zinc-800/50 rounded-xl p-4 border border-zinc-700/50">
          <h3 className="text-xs font-semibold text-zinc-400 uppercase tracking-wider mb-3">Today's Focus</h3>
          <ul className="space-y-2">
            <li className="flex items-center space-x-3 text-sm">
              <div className="w-4 h-4 rounded border border-zinc-500"></div>
              <span className="text-zinc-300">Review system architecture</span>
            </li>
            <li className="flex items-center space-x-3 text-sm">
              <div className="w-4 h-4 rounded border border-zinc-500"></div>
              <span className="text-zinc-300">Complete desktop companion shell</span>
            </li>
          </ul>
        </section>

        {/* Status / Quick Access Placeholder */}
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

        {/* Development Controls (Temporary for Assessment Mode demonstration) */}
        <section className="pt-2">
           <button 
            onClick={() => changeMode("Assessment")}
            className="w-full py-2 px-4 rounded-lg bg-red-900/30 text-red-400 text-sm font-medium border border-red-900/50 hover:bg-red-900/50 transition-colors"
          >
            Enter Assessment Mode (Hides NOVA)
          </button>
        </section>

      </main>

      {/* Footer / Chat Placeholder */}
      <footer className="p-4 border-t border-zinc-800 bg-zinc-900">
        <div className="relative">
          <input 
            type="text" 
            disabled 
            placeholder="AI connection coming next..." 
            className="w-full bg-zinc-800 border border-zinc-700 text-sm text-zinc-300 rounded-lg py-2.5 pl-4 pr-10 focus:outline-none focus:border-zinc-500 cursor-not-allowed opacity-70"
          />
          <button disabled className="absolute right-2 top-1/2 -translate-y-1/2 p-1.5 text-zinc-500 cursor-not-allowed">
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="m22 2-7 20-4-9-9-4Z"/><path d="M22 2 11 13"/></svg>
          </button>
        </div>
      </footer>

    </div>
  );
}
