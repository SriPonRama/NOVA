import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { listen } from "@tauri-apps/api/event";
import { enable, disable, isEnabled as checkAutostartEnabled } from "@tauri-apps/plugin-autostart";

type AppMode = "Active" | "Focus" | "Break" | "Assessment" | "Paused" | "Disabled";
type ViewMode = "Chat" | "Memory" | "Today" | "Settings";

type ChatMessage = {
  role: "user" | "assistant";
  content: string;
};

type MemoryCategory = "UserPreference" | "Goal" | "Study" | "Project" | "Important" | "Temporary";

type Memory = {
  id: string;
  content: string;
  category: MemoryCategory;
  created_at: number;
  updated_at: number;
};

type TaskStatus = "Pending" | "InProgress" | "Completed" | "Cancelled";
type TaskPriority = "Low" | "Medium" | "High";

type Task = {
  id: string;
  title: string;
  description: string | null;
  date: string;
  estimated_minutes: number | null;
  priority: TaskPriority;
  status: TaskStatus;
  position: number;
  created_at: number;
  updated_at: number;
};

type SessionType = "Focus" | "Break";
type SessionStatus = "Running" | "Paused" | "Completed" | "Cancelled";

type FocusSession = {
  id: string;
  task_id: string | null;
  session_type: SessionType;
  planned_seconds: number;
  started_at: number | null;
  paused_at: number | null;
  ended_at: number | null;
  status: SessionStatus;
  created_at: number;
  updated_at: number;
};

type TimerState = {
  active_session: FocusSession | null;
  remaining_seconds: number;
};

export interface RemainingWorkload {
  pending_tasks: number;
  in_progress_tasks: number;
  total_estimated_minutes: number;
  high_priority_count: number;
  medium_priority_count: number;
  low_priority_count: number;
}

export interface TaskRecommendation {
  task: Task | null;
  reasoning: string;
}

type DistractionState = 
  | "Idle"
  | "FocusActive"
  | "RelevantActivity"
  | "PotentialDistraction"
  | "DistractionConfirmed"
  | "InterventionShown"
  | "Cooldown";

type DesktopSettings = {
  enabled: boolean;
  grace_period_seconds: number;
  cooldown_minutes: number;
};

type CVSettings = { enabled: boolean };
type DrowsinessSignal = { face_detected: boolean, eye_measurement: number, confidence: number, timestamp: number, status: string };
type DrowsinessState = "Awake" | "PossiblyDrowsy" | "Drowsy" | "Intervention" | "Cooldown";

export default function App() {
  const [mode, setMode] = useState<AppMode>("Active");
  const [view, setView] = useState<ViewMode>("Today");
  
  // Chat state
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [inputText, setInputText] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const [chatError, setChatError] = useState<string | null>(null);
  const [pendingDeletionId, setPendingDeletionId] = useState<string | null>(null);
  const [pendingDeletionType, setPendingDeletionType] = useState<"memory" | "task" | null>(null);
  const messagesEndRef = useRef<HTMLDivElement>(null);

  // Memory state
  const [memories, setMemories] = useState<Memory[]>([]);
  const [searchQuery, setSearchQuery] = useState("");
  const [memoryError, setMemoryError] = useState<string | null>(null);
  
  const [isFormOpen, setIsFormOpen] = useState(false);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [formContent, setFormContent] = useState("");
  const [formCategory, setFormCategory] = useState<MemoryCategory>("Temporary");

  // Today state
  const [tasks, setTasks] = useState<Task[]>([]);
  const [taskError, setTaskError] = useState<string | null>(null);
  
  const [isTaskFormOpen, setIsTaskFormOpen] = useState(false);
  const [taskEditingId, setTaskEditingId] = useState<string | null>(null);
  const [taskFormTitle, setTaskFormTitle] = useState("");
  const [taskFormDesc, setTaskFormDesc] = useState("");
  const [taskFormEst, setTaskFormEst] = useState<string>("");
  const [taskFormPriority, setTaskFormPriority] = useState<TaskPriority>("Medium");
  const [taskFormDate, setTaskFormDate] = useState<string>("");

  const [workload, setWorkload] = useState<RemainingWorkload | null>(null);
  const [recommendation, setRecommendation] = useState<TaskRecommendation | null>(null);

  // Focus state
  const [timerState, setTimerState] = useState<TimerState>({ active_session: null, remaining_seconds: 0 });
  const [showSessionConfig, setShowSessionConfig] = useState<{taskId: string | null, type: SessionType} | null>(null);
  const [sessionDuration, setSessionDuration] = useState(25);

  // Desktop Awareness state
  const [distractionState, setDistractionState] = useState<DistractionState>("Idle");
  const [desktopSettings, setDesktopSettings] = useState<DesktopSettings>({ enabled: true, grace_period_seconds: 60, cooldown_minutes: 5 });

  const [startWithWindows, setStartWithWindows] = useState(false);
  const [isCompanion, setIsCompanion] = useState(false);

  const [cvSettings, setCvSettings] = useState<CVSettings>({ enabled: false });
  const [cvSignal, setCvSignal] = useState<DrowsinessSignal | null>(null);
  const [drowsinessState, setDrowsinessState] = useState<DrowsinessState>("Awake");
  const [showDrowsinessIntervention, setShowDrowsinessIntervention] = useState(false);

  useEffect(() => {
    invoke<AppMode>("get_app_mode").then(setMode).catch(console.error);
    invoke<DesktopSettings>("get_desktop_awareness_settings").then(setDesktopSettings).catch(console.error);
    invoke<CVSettings>("get_cv_settings").then(setCvSettings).catch(console.error);
    
    checkAutostartEnabled().then(setStartWithWindows).catch(console.error);
    
    try {
      const win = getCurrentWindow();
      if (win && win.label === "companion") {
        setIsCompanion(true);
      }
    } catch(e) {}

    const unlistenNavigate = listen("navigate", (event) => {
      if (event.payload === "Today") {
        setView("Today");
      }
    });

    const unlistenDrowsinessState = listen<DrowsinessState>("drowsiness_state_changed", (event) => {
      setDrowsinessState(event.payload);
    });

    const unlistenDrowsinessIntervention = listen("drowsiness_intervention", () => {
      setShowDrowsinessIntervention(true);
    });

    return () => {
      unlistenNavigate.then(f => f());
      unlistenDrowsinessState.then(f => f());
      unlistenDrowsinessIntervention.then(f => f());
    };
  }, []);

  useEffect(() => {
    const int = setInterval(() => {
      invoke<DrowsinessSignal | null>("get_latest_cv_signal").then(sig => {
        if (sig) setCvSignal(sig);
      }).catch(() => {});
    }, 1000);
    return () => clearInterval(int);
  }, []);

  const handleToggleAutostart = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const enabled = e.target.checked;
    setStartWithWindows(enabled);
    if (enabled) {
      await enable();
    } else {
      await disable();
    }
  };

  const handleToggleCV = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const enabled = e.target.checked;
    setCvSettings({ enabled });
    try {
      await invoke("toggle_cv_monitoring", { enabled });
    } catch (err) {
      console.error("Failed to toggle CV monitoring", err);
    }
  };

  const getTodayDateStr = () => {
    const d = new Date();
    return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`;
  };

  useEffect(() => {
    if (view === "Chat") {
      messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
    } else if (view === "Memory") {
      loadMemories();
    } else if (view === "Today") {
      loadTasks();
    }
  }, [view, messages, isLoading]);

  useEffect(() => {
    let interval: number;
    const fetchTimerState = async () => {
      try {
        const state = await invoke<TimerState>("get_timer_state");
        setTimerState(state);

        // If focus is running, poll distraction state
        if (state.active_session?.status === 'Running' && state.active_session.session_type === 'Focus') {
          const dState = await invoke<DistractionState>("get_distraction_state");
          setDistractionState(dState);
        } else {
          setDistractionState("Idle");
        }
      } catch (err) {
        console.error("Failed to fetch state", err);
      }
    };
    
    interval = window.setInterval(fetchTimerState, 1000);
    fetchTimerState();
    
    return () => clearInterval(interval);
  }, []);

  const changeMode = async (newMode: AppMode) => {
    try {
      await invoke("set_app_mode", { mode: newMode });
      setMode(newMode);
    } catch (e) {
      console.error("Failed to set app mode:", e);
    }
  };

  // Chat Handlers
  const sendMessageCore = async (messageText: string) => {
    if (!messageText.trim() || isLoading) return;

    const userMessage: ChatMessage = { role: "user", content: messageText.trim() };
    setMessages((prev) => [...prev, userMessage]);
    setIsLoading(true);
    setChatError(null);

    try {
      const response = await invoke<{text: string, pending_deletion: string | null, pending_deletion_type: string | null}>("send_message", { message: userMessage.content });
      setMessages((prev) => [...prev, { role: "assistant", content: response.text }]);
      
      if (response.pending_deletion) {
        setPendingDeletionId(response.pending_deletion);
        setPendingDeletionType(response.pending_deletion_type as "memory" | "task" | null);
      }
    } catch (err) {
      let errMsg = "An unexpected error occurred.";
      if (typeof err === "string") {
        if (err.includes("GEMINI_API_KEY")) errMsg = "NOVA needs a Gemini API key to talk.";
        else if (err.includes("Assessment Mode")) errMsg = "AI operations are blocked in Assessment Mode.";
        else errMsg = "AI Provider Error.";
      }
      setChatError(errMsg);
    } finally {
      setIsLoading(false);
    }
  };

  const handleSendMessage = async () => {
    if (!inputText.trim()) return;
    const text = inputText;
    setInputText("");
    await sendMessageCore(text);
  };

  const handleConfirmDeleteAI = async (confirm: boolean) => {
    if (!pendingDeletionId) return;
    const id = pendingDeletionId;
    const type = pendingDeletionType;
    setPendingDeletionId(null);
    setPendingDeletionType(null);
    if (confirm) {
      try {
        if (type === "task") {
            await invoke("delete_task", { id });
            await sendMessageCore("I confirmed the task deletion.");
            loadTasks();
        } else {
            await invoke("delete_memory", { id });
            await sendMessageCore("I confirmed the memory deletion.");
            loadMemories();
        }
      } catch (err) {
        console.error(`Failed to delete ${type} via AI:`, err);
      }
    } else {
      await sendMessageCore("I cancelled the deletion.");
    }
  };

  // Memory Handlers
  const loadMemories = async () => {
    try {
      setMemoryError(null);
      let res: Memory[];
      if (searchQuery.trim() !== "") {
        res = await invoke("search_memories", { query: searchQuery });
      } else {
        res = await invoke("list_memories");
      }
      setMemories(res);
    } catch (e) {
      setMemoryError("Failed to load memories");
    }
  };

  const handleSearchChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    setSearchQuery(e.target.value);
  };

  useEffect(() => {
    if (view === "Memory") {
      const timeout = setTimeout(() => {
        loadMemories();
      }, 300);
      return () => clearTimeout(timeout);
    }
  }, [searchQuery]);

  const openCreateForm = () => {
    setEditingId(null);
    setFormContent("");
    setFormCategory("Temporary");
    setIsFormOpen(true);
  };

  const openEditForm = (mem: Memory) => {
    setEditingId(mem.id);
    setFormContent(mem.content);
    setFormCategory(mem.category);
    setIsFormOpen(true);
  };

  const saveMemory = async () => {
    if (!formContent.trim()) {
      setMemoryError("Content cannot be empty.");
      return;
    }
    try {
      if (editingId) {
        await invoke("update_memory", { id: editingId, content: formContent, category: formCategory });
      } else {
        await invoke("create_memory", { content: formContent, category: formCategory });
      }
      setIsFormOpen(false);
      loadMemories();
    } catch (e: any) {
      setMemoryError(e.toString());
    }
  };

  const deleteMemory = async (id: string) => {
    if (!window.confirm("Are you sure you want to delete this memory?")) return;
    try {
      await invoke("delete_memory", { id });
      loadMemories();
    } catch (e: any) {
      setMemoryError(e.toString());
    }
  };

  const clearAllMemories = async () => {
    if (!window.confirm("CRITICAL: Are you sure you want to delete ALL memories? This cannot be undone.")) return;
    try {
      await invoke("clear_all_memories");
      loadMemories();
    } catch (e: any) {
      setMemoryError(e.toString());
    }
  };

  // Task Handlers
  const loadTasks = async () => {
    try {
      setTaskError(null);
      const dateStr = getTodayDateStr();
      const res = await invoke<Task[]>("list_tasks", { date: dateStr });
      setTasks(res);
      
      try {
        const wl = await invoke<RemainingWorkload>("get_remaining_workload", { date: dateStr });
        const rec = await invoke<TaskRecommendation>("get_next_recommended_task", { date: dateStr });
        setWorkload(wl);
        setRecommendation(rec);
      } catch (innerErr) {
        console.warn("Failed to load AI insights:", innerErr);
      }
    } catch (e) {
      setTaskError("Failed to load tasks");
    }
  };

  const openTaskCreateForm = () => {
    setTaskEditingId(null);
    setTaskFormTitle("");
    setTaskFormDesc("");
    setTaskFormEst("");
    setTaskFormPriority("Medium");
    setTaskFormDate(getTodayDateStr());
    setIsTaskFormOpen(true);
  };

  const openTaskEditForm = (t: Task) => {
    setTaskEditingId(t.id);
    setTaskFormTitle(t.title);
    setTaskFormDesc(t.description || "");
    setTaskFormEst(t.estimated_minutes ? t.estimated_minutes.toString() : "");
    setTaskFormPriority(t.priority);
    setTaskFormDate(t.date);
    setIsTaskFormOpen(true);
  };

  const saveTask = async () => {
    if (!taskFormTitle.trim()) {
      setTaskError("Title cannot be empty.");
      return;
    }
    const estimated_minutes = taskFormEst ? parseInt(taskFormEst, 10) : null;
    try {
      if (taskEditingId) {
        const existing = tasks.find(t => t.id === taskEditingId);
        await invoke("update_task", { 
          id: taskEditingId, 
          title: taskFormTitle, 
          description: taskFormDesc ? taskFormDesc : null, 
          date: taskFormDate, 
          estimated_minutes, 
          priority: taskFormPriority, 
          status: existing ? existing.status : "Pending"
        });
      } else {
        await invoke("create_task", { 
          title: taskFormTitle, 
          description: taskFormDesc ? taskFormDesc : null, 
          date: taskFormDate, 
          estimated_minutes, 
          priority: taskFormPriority
        });
      }
      setIsTaskFormOpen(false);
      loadTasks();
    } catch (e: any) {
      setTaskError(e.toString());
    }
  };

  const deleteTask = async (id: string) => {
    if (!window.confirm("Are you sure you want to delete this task?")) return;
    try {
      await invoke("delete_task", { id });
      loadTasks();
    } catch (e: any) {
      setTaskError(e.toString());
    }
  };

  const toggleTaskStatus = async (task: Task) => {
    const newStatus = task.status === "Completed" ? "Pending" : "Completed";
    try {
      await invoke("set_task_status", { id: task.id, status: newStatus });
      loadTasks();
    } catch (e: any) {
      setTaskError(e.toString());
    }
  };

  // Focus Handlers
  const startFocusSession = async () => {
    try {
      if (showSessionConfig?.type === "Focus" && showSessionConfig.taskId) {
        await invoke("start_focus_session", { taskId: showSessionConfig.taskId, durationSeconds: sessionDuration * 60 });
        loadTasks(); 
      } else if (showSessionConfig?.type === "Break") {
        await invoke("start_break", { durationSeconds: sessionDuration * 60 });
      }
      setShowSessionConfig(null);
    } catch (e: any) {
      setTaskError(e.toString());
    }
  };

  const pauseSession = async () => {
    try { await invoke("pause_focus_session"); } catch (e: any) { setTaskError(e.toString()); }
  };
  
  const resumeSession = async () => {
    try { await invoke("resume_focus_session"); } catch (e: any) { setTaskError(e.toString()); }
  };
  
  const finishSession = async () => {
    try { await invoke("finish_focus_session"); loadTasks(); } catch (e: any) { setTaskError(e.toString()); }
  };
  
  const cancelSession = async () => {
    if(!window.confirm("Cancel this session?")) return;
    try { await invoke("cancel_focus_session"); } catch (e: any) { setTaskError(e.toString()); }
  };

  const formatTime = (secs: number) => {
    const m = Math.floor(secs / 60);
    const s = secs % 60;
    return `${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`;
  };

  // Settings Handlers
  const handleToggleAwareness = async () => {
    try {
      const nv = !desktopSettings.enabled;
      await invoke("set_desktop_awareness_enabled", { enabled: nv });
      setDesktopSettings(prev => ({...prev, enabled: nv}));
    } catch(e) { console.error(e) }
  };

  const handleChangeGracePeriod = async (e: any) => {
    try {
      const v = parseInt(e.target.value, 10);
      await invoke("set_grace_period", { seconds: v });
      setDesktopSettings(prev => ({...prev, grace_period_seconds: v}));
    } catch(e) { console.error(e) }
  };

  const handleChangeCooldown = async (e: any) => {
    try {
      const v = parseInt(e.target.value, 10);
      await invoke("set_cooldown", { minutes: v });
      setDesktopSettings(prev => ({...prev, cooldown_minutes: v}));
    } catch(e) { console.error(e) }
  };

  // Intervention Handlers
  const handleReturnToFocus = async () => {
    await invoke("return_to_focus");
    setDistractionState("FocusActive");
  };

  const handleKeepWorkingHere = async () => {
    await invoke("keep_working_here");
    setDistractionState("Cooldown");
  };

  const handleTakeBreak = async () => {
    setShowDrowsinessIntervention(false);
    try {
      await invoke("take_drowsiness_break");
      setDrowsinessState("Cooldown");
    } catch (e) {
      console.error(e);
    }
  };

  const handleDismissIntervention = async () => {
    setShowDrowsinessIntervention(false);
    try {
      await invoke("dismiss_drowsiness_intervention");
      setDrowsinessState("Cooldown");
    } catch (e) {
      console.error(e);
    }
  };

  const activeTask = timerState.active_session?.task_id 
    ? tasks.find(t => t.id === timerState.active_session!.task_id) 
    : null;

  if (isCompanion) {
    return (
      <div className="flex flex-col h-screen bg-zinc-900 text-zinc-100 font-sans antialiased overflow-hidden p-4">
        <div className="flex justify-between items-center mb-6">
          <h1 className="font-semibold text-zinc-100 tracking-wide text-sm flex items-center gap-2">
            <div className={`w-2.5 h-2.5 rounded-full ${timerState.active_session?.status === 'Running' ? 'bg-red-500 shadow-[0_0_8px_rgba(239,68,68,0.5)] animate-pulse' : 'bg-emerald-500 shadow-[0_0_8px_rgba(16,185,129,0.5)]'}`}></div>
            NOVA
          </h1>
          <span className={`text-[10px] uppercase font-bold px-2 py-0.5 rounded ${mode === "Disabled" ? "bg-zinc-800 text-zinc-500" : "bg-emerald-900/30 text-emerald-500"}`}>{mode}</span>
        </div>
        
        {timerState.active_session ? (
          <div className="bg-zinc-800/50 rounded-xl p-4 border border-zinc-700/50 flex-1 flex flex-col justify-center items-center">
            <p className="text-xs text-zinc-400 mb-2 uppercase tracking-wider">{timerState.active_session.session_type}</p>
            <p className="text-4xl font-mono text-emerald-400 font-medium tracking-tight mb-4">{formatTime(timerState.remaining_seconds)}</p>
            {activeTask && (
              <p className="text-sm text-zinc-300 text-center mb-6">{activeTask.title}</p>
            )}
            <div className="flex gap-2">
              {timerState.active_session.status === 'Running' ? (
                <button onClick={pauseSession} className="px-3 py-1.5 bg-amber-600/20 text-amber-500 border border-amber-500/50 rounded hover:bg-amber-600/30 text-xs">Pause</button>
              ) : (
                <button onClick={resumeSession} className="px-3 py-1.5 bg-emerald-600/20 text-emerald-500 border border-emerald-500/50 rounded hover:bg-emerald-600/30 text-xs">Resume</button>
              )}
              <button onClick={finishSession} className="px-3 py-1.5 bg-zinc-700 hover:bg-zinc-600 text-zinc-300 rounded text-xs">Finish</button>
            </div>
          </div>
        ) : (
          <div className="bg-zinc-800/30 rounded-xl p-4 border border-zinc-700/30 flex-1 flex flex-col justify-center items-center space-y-3">
            <p className="text-sm text-zinc-400">Ready to focus.</p>
            <div className="flex gap-2">
              <button onClick={() => invoke("start_focus_session", { taskId: null, durationSeconds: 25 * 60 }).catch(console.error)} className="px-3 py-1.5 bg-emerald-600 hover:bg-emerald-500 text-white rounded text-xs">Start Focus</button>
              <button onClick={() => invoke("start_break", { durationSeconds: 5 * 60 }).catch(console.error)} className="px-3 py-1.5 bg-zinc-700 hover:bg-zinc-600 text-zinc-300 rounded text-xs">Break</button>
            </div>
          </div>
        )}
      </div>
    );
  }

  return (
    <div className="flex flex-col h-screen bg-zinc-900 text-zinc-100 font-sans antialiased overflow-hidden selection:bg-zinc-700">
      
      {/* Distraction Intervention Modal */}
      {(distractionState === "DistractionConfirmed" || distractionState === "InterventionShown") && (
        <div className="absolute inset-0 z-50 bg-zinc-900/90 backdrop-blur-md flex items-center justify-center p-4">
            <div className="bg-zinc-800 border border-zinc-700 rounded-2xl p-8 max-w-sm w-full shadow-[0_0_40px_rgba(0,0,0,0.5)] flex flex-col items-center text-center transform scale-100 animate-in fade-in zoom-in-95 duration-300">
                <div className="w-12 h-12 bg-amber-500/20 rounded-full flex items-center justify-center mb-4">
                  <span className="text-amber-500">
                    <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M12 9v2m0 4v.01"/><path d="M5.07 19H19.3c1.5 0 2.25-1.81 1.4-3.05l-7.11-12.2a1.72 1.72 0 0 0-2.98 0L3.5 15.95C2.65 17.19 3.4 19 4.9 19Z"/></svg>
                  </span>
                </div>
                <h2 className="text-xl font-medium text-zinc-100 mb-3">Hey there</h2>
                <p className="text-sm text-zinc-400 mb-8 leading-relaxed">You've been away from your current task for a while. Want to get back to it?</p>
                
                <div className="flex flex-col gap-3 w-full">
                    <button 
                      onClick={handleReturnToFocus} 
                      className="py-3 px-4 bg-emerald-600 hover:bg-emerald-500 text-white rounded-xl text-sm font-medium transition-colors shadow-lg shadow-emerald-900/20"
                    >
                      Return to Focus
                    </button>
                    <button 
                      onClick={handleKeepWorkingHere} 
                      className="py-3 px-4 bg-zinc-700 hover:bg-zinc-600 text-zinc-300 rounded-xl text-sm font-medium transition-colors"
                    >
                      Keep Working Here
                    </button>
                </div>
            </div>
        </div>
      )}

      {/* Drowsiness Intervention Modal */}
      {showDrowsinessIntervention && (
        <div className="absolute inset-0 z-50 bg-zinc-900/95 backdrop-blur-md flex items-center justify-center p-4">
            <div className="bg-zinc-800 border border-zinc-700 rounded-2xl p-8 max-w-sm w-full shadow-[0_0_40px_rgba(0,0,0,0.5)] flex flex-col items-center text-center transform scale-100 animate-in fade-in zoom-in-95 duration-300">
                <div className="w-12 h-12 bg-indigo-500/20 rounded-full flex items-center justify-center mb-4">
                  <span className="text-indigo-400">
                    <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M2 12h4l3-9 5 18 3-9h5"/></svg>
                  </span>
                </div>
                <h2 className="text-xl font-medium text-zinc-100 mb-3">Hey there</h2>
                <p className="text-sm text-zinc-400 mb-8 leading-relaxed">You seem a little tired. Would you like to take a break?</p>
                
                <div className="flex flex-col gap-3 w-full">
                    <button 
                      onClick={handleTakeBreak} 
                      className="py-3 px-4 bg-indigo-600 hover:bg-indigo-500 text-white rounded-xl text-sm font-medium transition-colors shadow-lg shadow-indigo-900/20"
                    >
                      Take a Break
                    </button>
                    <button 
                      onClick={handleDismissIntervention} 
                      className="py-3 px-4 bg-zinc-700 hover:bg-zinc-600 text-zinc-300 rounded-xl text-sm font-medium transition-colors"
                    >
                      I'm Fine
                    </button>
                </div>
            </div>
        </div>
      )}

      {/* Header */}
      <header className="flex items-center justify-between px-4 py-3 border-b border-zinc-800 bg-zinc-900/80 backdrop-blur-md z-10 shrink-0">
        <div className="flex items-center space-x-4">
          <div className="flex items-center space-x-2 mr-2">
            <div className={`w-2.5 h-2.5 rounded-full ${timerState.active_session?.status === 'Running' ? 'bg-red-500 shadow-[0_0_8px_rgba(239,68,68,0.5)] animate-pulse' : 'bg-emerald-500 shadow-[0_0_8px_rgba(16,185,129,0.5)]'}`}></div>
            <h1 className="font-semibold text-zinc-100 tracking-wide text-sm">NOVA</h1>
          </div>
          <button onClick={() => setView("Today")} className={`text-sm font-medium transition-colors ${view === "Today" ? "text-emerald-400" : "text-zinc-500 hover:text-zinc-300"}`}>Today</button>
          <button onClick={() => setView("Chat")} className={`text-sm font-medium transition-colors ${view === "Chat" ? "text-emerald-400" : "text-zinc-500 hover:text-zinc-300"}`}>Chat</button>
          <button onClick={() => setView("Memory")} className={`text-sm font-medium transition-colors ${view === "Memory" ? "text-emerald-400" : "text-zinc-500 hover:text-zinc-300"}`}>Memory</button>
          <button onClick={() => setView("Settings")} className={`text-sm font-medium transition-colors ${view === "Settings" ? "text-emerald-400" : "text-zinc-500 hover:text-zinc-300"}`}>Settings</button>
        </div>
        <div className="flex items-center space-x-2">
          {timerState.active_session && timerState.active_session.status !== "Completed" && timerState.active_session.status !== "Cancelled" && (
            <button 
              onClick={() => setView("Today")} 
              className="text-xs font-mono px-3 py-1 rounded bg-zinc-800 hover:bg-zinc-700 text-emerald-400 border border-emerald-900/50 transition-colors mr-2 flex items-center gap-2"
            >
              <span>{timerState.active_session.session_type.toUpperCase()}</span>
              <span>{formatTime(timerState.remaining_seconds)}</span>
            </button>
          )}
          <span className={`text-xs font-medium px-2.5 py-1 rounded-md ${mode === "Assessment" ? "bg-red-900/50 text-red-400 border border-red-900" : "bg-zinc-800 text-zinc-300"}`}>
            {mode}
          </span>
        </div>
      </header>

      {/* Main Content Area */}
      <main className="flex-1 overflow-y-auto p-4 flex flex-col gap-6 relative">
        
        {view === "Settings" && (
          <div className="max-w-xl mx-auto w-full pt-4">
            <h2 className="text-xl font-medium text-zinc-100 mb-6">Settings</h2>
            
            <section className="bg-zinc-800/50 border border-zinc-700/50 rounded-xl p-5 mb-6">
              <h3 className="text-sm font-medium text-zinc-300 mb-4 uppercase tracking-wider">Desktop Awareness</h3>
              
              <div className="flex items-center justify-between mb-5">
                <div>
                  <p className="text-zinc-100 font-medium text-sm">Enable Distraction Monitoring</p>
                  <p className="text-xs text-zinc-400 mt-1">NOVA will detect when you're distracted during active Focus Sessions.</p>
                </div>
                <label className="relative inline-flex items-center cursor-pointer">
                  <input type="checkbox" className="sr-only peer" checked={desktopSettings.enabled} onChange={handleToggleAwareness} />
                  <div className="w-11 h-6 bg-zinc-700 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-emerald-500"></div>
                </label>
              </div>

              <div className="mb-4">
                <label className="block text-zinc-100 font-medium text-sm mb-1">Grace Period</label>
                <p className="text-xs text-zinc-400 mb-2">Time allowed on distracting apps before NOVA asks you to return.</p>
                <select 
                  value={desktopSettings.grace_period_seconds} 
                  onChange={handleChangeGracePeriod}
                  disabled={!desktopSettings.enabled}
                  className="w-full bg-zinc-900 border border-zinc-700 text-sm text-zinc-200 rounded-lg py-2 px-3 focus:outline-none focus:border-emerald-500 disabled:opacity-50"
                >
                  <option value={30}>30 seconds</option>
                  <option value={60}>60 seconds</option>
                  <option value={90}>90 seconds</option>
                  <option value={120}>120 seconds</option>
                </select>
              </div>

              <div>
                <label className="block text-zinc-100 font-medium text-sm mb-1">Intervention Cooldown</label>
                <p className="text-xs text-zinc-400 mb-2">Time to wait after you choose "Keep Working Here" before asking again.</p>
                <select 
                  value={desktopSettings.cooldown_minutes} 
                  onChange={handleChangeCooldown}
                  disabled={!desktopSettings.enabled}
                  className="w-full bg-zinc-900 border border-zinc-700 text-sm text-zinc-200 rounded-lg py-2 px-3 focus:outline-none focus:border-emerald-500 disabled:opacity-50"
                >
                  <option value={3}>3 minutes</option>
                  <option value={5}>5 minutes</option>
                  <option value={10}>10 minutes</option>
                </select>
              </div>
            </section>
            <section className="bg-zinc-800/50 border border-zinc-700/50 rounded-xl p-5 mb-6 mt-6">
              <h3 className="text-sm font-medium text-zinc-300 mb-4 uppercase tracking-wider">Computer Vision</h3>
              
              <div className="flex items-center justify-between mb-5">
                <div>
                  <p className="text-zinc-100 font-medium text-sm">Enable Drowsiness Monitoring</p>
                  <p className="text-xs text-zinc-400 mt-1">NOVA will use your local webcam to detect signs of drowsiness during focus sessions.</p>
                </div>
                <label className="relative inline-flex items-center cursor-pointer">
                  <input type="checkbox" className="sr-only peer" checked={cvSettings.enabled} onChange={handleToggleCV} />
                  <div className="w-11 h-6 bg-zinc-700 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-emerald-500"></div>
                </label>
              </div>

              <div className="bg-zinc-900/50 border border-zinc-700/50 rounded-lg p-3 text-xs mb-3">
                <div className="flex justify-between items-center mb-1">
                  <span className="text-zinc-400">CV Status:</span>
                  <span className={`font-mono ${cvSettings.enabled ? 'text-emerald-400' : 'text-zinc-500'}`}>{cvSignal ? cvSignal.status : (cvSettings.enabled ? 'Starting/No signal' : 'Stopped')}</span>
                </div>
                {cvSignal && cvSettings.enabled && (
                  <>
                    <div className="flex justify-between items-center mb-1">
                      <span className="text-zinc-400">Face Detected:</span>
                      <span className="font-mono text-zinc-300">{cvSignal.face_detected ? 'Yes' : 'No'}</span>
                    </div>
                    <div className="flex justify-between items-center">
                      <span className="text-zinc-400">Eye Measurement (EAR):</span>
                      <span className="font-mono text-zinc-300">{cvSignal.eye_measurement.toFixed(3)}</span>
                    </div>
                  </>
                )}
              </div>

              <p className="text-xs text-zinc-500 italic mt-3">
                Privacy statement: Camera processing happens locally. NOVA does not save or upload webcam frames. No identity recognition is performed.
              </p>
            </section>
          </div>
        )}

        {view === "Chat" && (
          <>
            {messages.length === 0 && (
              <div className="space-y-6">
                <section className="space-y-1">
                  <h2 className="text-xl font-medium text-zinc-100">Good morning.</h2>
                  <p className="text-sm text-zinc-400">Your systems are running optimally.</p>
                </section>
                <section className="pt-2 flex gap-2">
                  <button 
                    onClick={() => changeMode(mode === "Disabled" ? "Active" : "Disabled")}
                    className={`flex-1 py-2 px-4 rounded-lg text-sm font-medium border transition-colors ${
                      mode === "Disabled" 
                        ? "bg-emerald-900/30 text-emerald-400 border-emerald-900/50 hover:bg-emerald-900/50" 
                        : "bg-zinc-800 text-zinc-300 border-zinc-700 hover:bg-zinc-700"
                    }`}
                  >
                    {mode === "Disabled" ? "Enable NOVA" : "Disable NOVA"}
                  </button>
                  <button 
                    onClick={() => changeMode("Assessment")}
                    className="flex-1 py-2 px-4 rounded-lg bg-red-900/30 text-red-400 text-sm font-medium border border-red-900/50 hover:bg-red-900/50 transition-colors"
                  >
                    Enter Assessment Mode
                  </button>
                </section>
              </div>
            )}

            <section className="bg-zinc-800/50 border border-zinc-700/50 rounded-xl p-5 mb-6 mt-6">
              <h3 className="text-sm font-medium text-zinc-300 mb-4 uppercase tracking-wider">Desktop Companion</h3>
              
              <div className="flex items-center justify-between mb-5">
                <div>
                  <p className="text-zinc-100 font-medium text-sm">Start NOVA with Windows</p>
                  <p className="text-xs text-zinc-400 mt-1">NOVA will launch silently in the background when you log in.</p>
                </div>
                <label className="relative inline-flex items-center cursor-pointer">
                  <input type="checkbox" className="sr-only peer" checked={startWithWindows} onChange={handleToggleAutostart} />
                  <div className="w-11 h-6 bg-zinc-700 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-emerald-500"></div>
                </label>
              </div>
            </section>

            <div className="flex-1 space-y-4 flex flex-col justify-end pb-2">
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

              {chatError && (
                <div className="bg-red-900/20 border border-red-900/50 rounded-xl p-3 mt-2 text-sm text-red-400">
                  {chatError}
                </div>
              )}

              {pendingDeletionId && (
                <div className="bg-zinc-800/80 border border-emerald-500/50 rounded-xl p-4 mt-2 shadow-lg shadow-black/20 flex flex-col items-center animate-in fade-in zoom-in-95 duration-200">
                  <p className="text-sm text-zinc-200 mb-3 text-center">NOVA is asking for permission to delete a {pendingDeletionType === 'task' ? 'task' : 'memory'}. Are you sure?</p>
                  <div className="flex space-x-3 w-full justify-center">
                    <button onClick={() => handleConfirmDeleteAI(false)} className="px-4 py-1.5 text-sm font-medium text-zinc-300 bg-zinc-700 hover:bg-zinc-600 rounded-lg transition-colors flex-1 max-w-[120px]">Cancel</button>
                    <button onClick={() => handleConfirmDeleteAI(true)} className="px-4 py-1.5 text-sm font-medium text-white bg-red-600 hover:bg-red-500 rounded-lg shadow-sm shadow-red-900/50 transition-colors flex-1 max-w-[120px]">Delete</button>
                  </div>
                </div>
              )}
              
              <div ref={messagesEndRef} />
            </div>
          </>
        )}

        {view === "Today" && (
          <div className="flex flex-col h-full">
            <div className="flex justify-between items-center mb-6">
              <div>
                <h2 className="text-lg font-medium text-zinc-100">Today</h2>
                <p className="text-sm text-zinc-400">{new Date().toLocaleDateString(undefined, { weekday: 'long', month: 'long', day: 'numeric' })}</p>
              </div>
              <button onClick={openTaskCreateForm} className="text-xs font-medium px-3 py-1.5 rounded bg-emerald-600 hover:bg-emerald-500 text-white transition-colors">
                + Add Task
              </button>
            </div>

            {/* Active Session Display */}
            {timerState.active_session && timerState.active_session.status !== 'Cancelled' && (
              <div className={`rounded-xl p-5 mb-6 border transition-all ${
                timerState.active_session.status === 'Completed' 
                  ? 'bg-emerald-900/20 border-emerald-500/50' 
                  : timerState.active_session.session_type === 'Break'
                    ? 'bg-blue-900/20 border-blue-500/30'
                    : 'bg-zinc-800/80 border-zinc-700'
              }`}>
                <div className="flex justify-between items-start mb-4">
                  <div>
                    <h3 className="text-sm font-medium text-zinc-400 uppercase tracking-wider flex items-center gap-2">
                      {timerState.active_session.session_type} {timerState.active_session.status === 'Paused' && "(PAUSED)"}
                    </h3>
                    <p className="text-lg font-medium text-zinc-100 mt-1">
                      {timerState.active_session.session_type === "Focus" 
                        ? (activeTask?.title || "Focus Session")
                        : "Take a break"}
                    </p>
                    
                    {/* Desktop Awareness Indicator */}
                    {timerState.active_session.session_type === 'Focus' && timerState.active_session.status === 'Running' && desktopSettings.enabled && (
                      <div className="flex items-center gap-1.5 mt-2 text-xs">
                        <span className="relative flex h-2 w-2">
                          <span className={`animate-ping absolute inline-flex h-full w-full rounded-full opacity-75 ${mode === "Assessment" ? "bg-red-400" : "bg-emerald-400"}`}></span>
                          <span className={`relative inline-flex rounded-full h-2 w-2 ${mode === "Assessment" ? "bg-red-500" : "bg-emerald-500"}`}></span>
                        </span>
                        <span className={`${mode === "Assessment" ? "text-red-400" : "text-emerald-400/80"} font-medium`}>
                          {mode === "Assessment" ? "Activity Monitoring Blocked" : "Focus activity: on"}
                        </span>
                      </div>
                    )}
                    {timerState.active_session.session_type === 'Focus' && timerState.active_session.status === 'Paused' && desktopSettings.enabled && (
                       <div className="flex items-center gap-1.5 mt-2 text-xs">
                         <span className="relative flex h-2 w-2">
                           <span className="relative inline-flex rounded-full h-2 w-2 bg-amber-500"></span>
                         </span>
                         <span className="text-amber-500/80 font-medium">Focus activity: paused</span>
                       </div>
                    )}
                    
                    {/* Drowsiness Indicator */}
                    {timerState.active_session.session_type === 'Focus' && timerState.active_session.status === 'Running' && cvSettings.enabled && (
                      <div className="flex items-center gap-1.5 mt-2 text-xs">
                        <span className="relative flex h-2 w-2">
                          <span className={`absolute inline-flex h-full w-full rounded-full opacity-75 ${drowsinessState === 'PossiblyDrowsy' ? 'bg-amber-400 animate-ping' : drowsinessState === 'Drowsy' || drowsinessState === 'Intervention' ? 'bg-red-400 animate-ping' : 'bg-emerald-400'}`}></span>
                          <span className={`relative inline-flex rounded-full h-2 w-2 ${drowsinessState === 'PossiblyDrowsy' ? 'bg-amber-500' : drowsinessState === 'Drowsy' || drowsinessState === 'Intervention' ? 'bg-red-500' : 'bg-emerald-500'}`}></span>
                        </span>
                        <span className={`${drowsinessState === 'PossiblyDrowsy' ? 'text-amber-400' : drowsinessState === 'Drowsy' || drowsinessState === 'Intervention' ? 'text-red-400' : 'text-emerald-400/80'} font-medium`}>
                          {drowsinessState === 'Cooldown' ? "Drowsiness monitoring: cooldown" : 
                           drowsinessState === 'PossiblyDrowsy' ? "Possibly tired" : 
                           drowsinessState === 'Drowsy' || drowsinessState === 'Intervention' ? "Drowsiness detected" : 
                           "Monitoring alertness"}
                        </span>
                      </div>
                    )}
                  </div>
                  <div className="text-4xl font-mono tracking-tight font-light text-emerald-400">
                    {formatTime(timerState.remaining_seconds)}
                  </div>
                </div>

                <div className="w-full bg-zinc-700/50 rounded-full h-1.5 mb-5 overflow-hidden">
                  <div className={`h-full transition-all duration-1000 ${timerState.active_session.status === 'Completed' ? 'bg-emerald-500' : 'bg-emerald-400'}`} 
                       style={{ width: `${Math.max(0, 100 - (timerState.remaining_seconds / timerState.active_session.planned_seconds) * 100)}%` }}></div>
                </div>

                <div className="flex space-x-3">
                  {timerState.active_session.status === 'Completed' ? (
                    timerState.active_session.session_type === 'Focus' ? (
                      <button onClick={() => { setShowSessionConfig({taskId: null, type: "Break"}); setSessionDuration(5); }} className="px-4 py-2 bg-blue-600 hover:bg-blue-500 text-white text-sm font-medium rounded-lg">
                        Start Break
                      </button>
                    ) : (
                      <button onClick={() => { cancelSession() }} className="px-4 py-2 bg-zinc-700 hover:bg-zinc-600 text-white text-sm font-medium rounded-lg">
                        Ready
                      </button>
                    )
                  ) : (
                    <>
                      {timerState.active_session.status === 'Running' ? (
                        <button onClick={pauseSession} className="px-4 py-2 bg-amber-600/20 text-amber-500 border border-amber-500/50 hover:bg-amber-600/30 text-sm font-medium rounded-lg">Pause</button>
                      ) : (
                        <button onClick={resumeSession} className="px-4 py-2 bg-emerald-600/20 text-emerald-500 border border-emerald-500/50 hover:bg-emerald-600/30 text-sm font-medium rounded-lg">Resume</button>
                      )}
                      <button onClick={finishSession} className="px-4 py-2 bg-zinc-700 hover:bg-zinc-600 text-zinc-300 text-sm font-medium rounded-lg">Finish</button>
                      <button onClick={cancelSession} className="px-4 py-2 text-zinc-500 hover:text-zinc-300 text-sm font-medium rounded-lg ml-auto">Cancel</button>
                    </>
                  )}
                </div>
              </div>
            )}

            {/* AI Insight Panel */}
            {workload && (workload.pending_tasks > 0 || workload.in_progress_tasks > 0) && (
              <div className="bg-indigo-900/10 border border-indigo-500/20 rounded-xl p-4 mb-6">
                <div className="flex items-center gap-2 mb-3">
                  <div className="w-5 h-5 rounded bg-indigo-500/20 flex items-center justify-center text-indigo-400">
                    <svg className="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 10V3L4 14h7v7l9-11h-7z" /></svg>
                  </div>
                  <h3 className="text-sm font-medium text-indigo-300">AI Productivity Insight</h3>
                </div>
                
                <div className="grid grid-cols-2 gap-4 mb-4">
                  <div className="bg-black/20 rounded-lg p-3">
                    <p className="text-xs text-zinc-500 mb-1">Remaining Tasks</p>
                    <p className="text-xl text-zinc-200">{workload.pending_tasks + workload.in_progress_tasks}</p>
                    <div className="flex gap-1 mt-1">
                      {workload.high_priority_count > 0 && <span className="w-2 h-2 rounded-full bg-red-500 mt-1" title="High Priority"></span>}
                      {workload.medium_priority_count > 0 && <span className="w-2 h-2 rounded-full bg-amber-500 mt-1" title="Medium Priority"></span>}
                      {workload.low_priority_count > 0 && <span className="w-2 h-2 rounded-full bg-emerald-500 mt-1" title="Low Priority"></span>}
                    </div>
                  </div>
                  <div className="bg-black/20 rounded-lg p-3">
                    <p className="text-xs text-zinc-500 mb-1">Est. Time Remaining</p>
                    <p className="text-xl text-zinc-200">{workload.total_estimated_minutes} min</p>
                    <p className="text-xs text-zinc-600 mt-1">Total planned work</p>
                  </div>
                </div>

                {recommendation && recommendation.task && (
                  <div className="bg-black/30 rounded-lg p-3 border border-indigo-500/10">
                    <p className="text-xs text-indigo-400 mb-1 font-medium">Recommended Next Task:</p>
                    <p className="text-sm text-zinc-200 font-medium mb-1">{recommendation.task.title}</p>
                    <p className="text-xs text-zinc-400 italic">"{recommendation.reasoning}"</p>
                  </div>
                )}
              </div>
            )}

            {taskError && (
              <div className="bg-red-900/20 border border-red-900/50 rounded-lg p-2 mb-4 text-xs text-red-400">
                {taskError}
              </div>
            )}

            <div className="flex-1 overflow-y-auto pb-4 pr-1">
              {tasks.length === 0 ? (
                <div className="text-center text-zinc-500 text-sm py-12">
                  <p>No tasks planned for today.</p>
                  <button onClick={openTaskCreateForm} className="mt-4 px-4 py-2 bg-zinc-800 hover:bg-zinc-700 rounded-lg transition-colors text-zinc-300">
                    Add your first task
                  </button>
                </div>
              ) : (
                <div className="space-y-2">
                  {tasks.map((task, _idx) => (
                    <div key={task.id} className={`group flex items-center gap-3 p-3 rounded-lg border transition-colors ${
                      task.status === "Completed" ? "bg-zinc-800/30 border-transparent opacity-60" : "bg-zinc-800/60 border-zinc-700/50 hover:bg-zinc-800"
                    }`}>
                      <button 
                        onClick={() => toggleTaskStatus(task)}
                        className={`w-5 h-5 rounded-full border-2 flex items-center justify-center shrink-0 transition-colors ${
                          task.status === "Completed" ? "border-emerald-500 bg-emerald-500" : "border-zinc-500 hover:border-zinc-400"
                        }`}
                      >
                        {task.status === "Completed" && (
                          <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="black" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round"><path d="M20 6 9 17l-5-5"/></svg>
                        )}
                      </button>
                      
                      <div className="flex-1 min-w-0" onClick={() => openTaskEditForm(task)}>
                        <h3 className={`text-sm truncate cursor-pointer ${task.status === "Completed" ? "line-through text-zinc-500" : "text-zinc-200"}`}>
                          {task.title}
                        </h3>
                        {(task.description || task.estimated_minutes || task.priority) && (
                          <div className="flex items-center gap-3 mt-1 text-xs text-zinc-500">
                            {task.priority && (
                              <span className={`px-1.5 py-0.5 rounded text-[10px] uppercase tracking-wider ${
                                task.priority === "High" ? "bg-red-900/30 text-red-400" :
                                task.priority === "Medium" ? "bg-amber-900/30 text-amber-400" :
                                "bg-zinc-800 text-zinc-400"
                              }`}>
                                {task.priority}
                              </span>
                            )}
                            {task.estimated_minutes && (
                              <span className="flex items-center gap-1">
                                <svg xmlns="http://www.w3.org/2000/svg" width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/></svg>
                                {task.estimated_minutes}m
                              </span>
                            )}
                            {task.status === "InProgress" && <span className="text-emerald-500">In Progress</span>}
                          </div>
                        )}
                      </div>

                      <div className="opacity-0 group-hover:opacity-100 transition-opacity flex items-center space-x-2 shrink-0">
                        {!timerState.active_session && task.status !== "Completed" && (
                          <button 
                            onClick={() => { setShowSessionConfig({taskId: task.id, type: "Focus"}); setSessionDuration(task.estimated_minutes || 25); }}
                            className="px-3 py-1.5 bg-emerald-600/20 text-emerald-400 hover:bg-emerald-600/40 rounded text-xs font-medium transition-all"
                          >
                            Start Focus
                          </button>
                        )}
                        <button onClick={() => deleteTask(task.id)} className="p-1 text-red-500/70 hover:text-red-400 rounded transition-colors hover:bg-red-900/20">
                          <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M3 6h18"/><path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6"/><path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2"/></svg>
                        </button>
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </div>
            
            {/* Session Config Modal */}
            {showSessionConfig && (
              <div className="absolute inset-0 z-30 bg-zinc-900/90 backdrop-blur-sm flex items-center justify-center p-4">
                <div className="bg-zinc-800 border border-zinc-700 rounded-xl p-6 w-full max-w-sm shadow-2xl flex flex-col">
                  <h3 className="text-lg font-medium text-zinc-100 mb-2">
                    Start {showSessionConfig.type === "Focus" ? "Focus Session" : "Break"}
                  </h3>
                  <p className="text-sm text-zinc-400 mb-6">Select duration in minutes:</p>
                  
                  <div className="grid grid-cols-3 gap-2 mb-6">
                    {[5, 10, 15, 25, 45, 60].map(mins => (
                      <button 
                        key={mins}
                        onClick={() => setSessionDuration(mins)}
                        className={`py-2 rounded-lg text-sm font-medium border transition-colors ${sessionDuration === mins ? 'bg-emerald-600/20 border-emerald-500 text-emerald-400' : 'bg-zinc-900 border-zinc-700 text-zinc-300 hover:border-zinc-500'}`}
                      >
                        {mins} m
                      </button>
                    ))}
                  </div>

                  <div className="flex justify-end space-x-3 mt-2">
                    <button onClick={() => setShowSessionConfig(null)} className="text-sm text-zinc-400 hover:text-zinc-200">Cancel</button>
                    <button onClick={startFocusSession} className="text-sm bg-emerald-600 hover:bg-emerald-500 text-white py-2 px-5 rounded-lg">Start</button>
                  </div>
                </div>
              </div>
            )}

            {/* Task Form Modal */}
            {isTaskFormOpen && (
              <div className="absolute inset-0 z-30 bg-zinc-900/90 backdrop-blur-sm flex items-center justify-center p-4">
                <div className="bg-zinc-800 border border-zinc-700 rounded-xl p-5 w-full max-w-md shadow-2xl flex flex-col">
                  <h3 className="text-lg font-medium text-zinc-100 mb-4">{taskEditingId ? "Edit Task" : "New Task"}</h3>
                  
                  <label className="text-xs text-zinc-400 mb-1">Title</label>
                  <input type="text" value={taskFormTitle} onChange={(e) => setTaskFormTitle(e.target.value)} className="w-full bg-zinc-900 border border-zinc-700 text-sm text-zinc-200 rounded-lg py-2 px-3 mb-3 focus:outline-none focus:border-emerald-500" />
                  
                  <label className="text-xs text-zinc-400 mb-1">Description (optional)</label>
                  <textarea value={taskFormDesc} onChange={(e) => setTaskFormDesc(e.target.value)} className="w-full bg-zinc-900 border border-zinc-700 text-sm text-zinc-200 rounded-lg py-2 px-3 mb-3 min-h-[60px] resize-none focus:outline-none focus:border-emerald-500" />
                  
                  <div className="flex gap-3 mb-4">
                    <div className="flex-1">
                      <label className="text-xs text-zinc-400 mb-1 block">Priority</label>
                      <select value={taskFormPriority} onChange={(e) => setTaskFormPriority(e.target.value as TaskPriority)} className="w-full bg-zinc-900 border border-zinc-700 text-sm text-zinc-200 rounded-lg py-2 px-3 focus:outline-none focus:border-emerald-500">
                        <option value="Low">Low</option>
                        <option value="Medium">Medium</option>
                        <option value="High">High</option>
                      </select>
                    </div>
                    <div className="flex-1">
                      <label className="text-xs text-zinc-400 mb-1 block">Est. Minutes (optional)</label>
                      <input type="number" value={taskFormEst} onChange={(e) => setTaskFormEst(e.target.value)} className="w-full bg-zinc-900 border border-zinc-700 text-sm text-zinc-200 rounded-lg py-2 px-3 focus:outline-none focus:border-emerald-500" />
                    </div>
                  </div>

                  <div className="flex justify-end space-x-3 mt-2">
                    <button onClick={() => setIsTaskFormOpen(false)} className="text-sm text-zinc-400 hover:text-zinc-200">Cancel</button>
                    <button onClick={saveTask} className="text-sm bg-emerald-600 hover:bg-emerald-500 text-white py-1.5 px-4 rounded-lg">Save Task</button>
                  </div>
                </div>
              </div>
            )}
          </div>
        )}

        {view === "Memory" && (
          <div className="flex flex-col h-full">
            <div className="flex justify-between items-center mb-4">
              <h2 className="text-lg font-medium text-zinc-100">Memory Manager</h2>
              <div className="space-x-2">
                <button onClick={openCreateForm} className="text-xs font-medium px-3 py-1.5 rounded bg-emerald-600 hover:bg-emerald-500 text-white transition-colors">
                  + Create
                </button>
                <button onClick={clearAllMemories} className="text-xs font-medium px-3 py-1.5 rounded bg-red-900/50 hover:bg-red-900 border border-red-800 text-red-200 transition-colors">
                  Clear All
                </button>
              </div>
            </div>

            <input 
              type="text" 
              placeholder="Search memories..." 
              value={searchQuery}
              onChange={handleSearchChange}
              className="w-full bg-zinc-800/50 border border-zinc-700 text-sm text-zinc-200 rounded-lg py-2 pl-3 mb-4 focus:outline-none focus:border-emerald-500"
            />

            {memoryError && (
               <div className="bg-red-900/20 border border-red-900/50 rounded-lg p-2 mb-4 text-xs text-red-400">
                 {memoryError}
               </div>
            )}

            <div className="flex-1 overflow-y-auto space-y-3 pb-4 pr-1">
              {memories.length === 0 ? (
                <div className="text-center text-zinc-500 text-sm py-8">No memories found.</div>
              ) : (
                memories.map((mem) => (
                  <div key={mem.id} className="bg-zinc-800/40 border border-zinc-700/50 rounded-lg p-3 group relative">
                    <div className="flex justify-between items-start mb-2">
                      <span className="text-[10px] uppercase font-bold text-emerald-500/80 tracking-wider">
                        {mem.category}
                      </span>
                      <div className="space-x-2 opacity-0 group-hover:opacity-100 transition-opacity flex">
                        <button onClick={() => openEditForm(mem)} className="text-zinc-400 hover:text-zinc-200 text-xs">Edit</button>
                        <button onClick={() => deleteMemory(mem.id)} className="text-red-400 hover:text-red-300 text-xs">Delete</button>
                      </div>
                    </div>
                    <p className="text-sm text-zinc-300 whitespace-pre-wrap">{mem.content}</p>
                  </div>
                ))
              )}
            </div>

            {/* Form Modal */}
            {isFormOpen && (
              <div className="absolute inset-0 z-20 bg-zinc-900/90 backdrop-blur-sm flex items-center justify-center p-4">
                <div className="bg-zinc-800 border border-zinc-700 rounded-xl p-5 w-full max-w-md shadow-2xl flex flex-col">
                  <h3 className="text-lg font-medium text-zinc-100 mb-4">{editingId ? "Edit Memory" : "New Memory"}</h3>
                  <label className="text-xs text-zinc-400 mb-1">Category</label>
                  <select value={formCategory} onChange={(e) => setFormCategory(e.target.value as MemoryCategory)} className="w-full bg-zinc-900 border border-zinc-700 text-sm text-zinc-200 rounded-lg py-2 px-3 mb-4">
                    <option value="Temporary">Temporary</option>
                    <option value="UserPreference">User Preference</option>
                    <option value="Goal">Goal</option>
                    <option value="Study">Study</option>
                    <option value="Project">Project</option>
                    <option value="Important">Important</option>
                  </select>
                  <label className="text-xs text-zinc-400 mb-1">Content</label>
                  <textarea value={formContent} onChange={(e) => setFormContent(e.target.value)} className="w-full bg-zinc-900 border border-zinc-700 text-sm text-zinc-200 rounded-lg py-2 px-3 mb-4 min-h-[100px] resize-none" />
                  <div className="flex justify-end space-x-3 mt-2">
                    <button onClick={() => setIsFormOpen(false)} className="text-sm text-zinc-400 hover:text-zinc-200">Cancel</button>
                    <button onClick={saveMemory} className="text-sm bg-emerald-600 hover:bg-emerald-500 text-white py-1.5 px-4 rounded-lg">Save</button>
                  </div>
                </div>
              </div>
            )}
          </div>
        )}

      </main>

      {view === "Chat" && (
        <footer className="p-4 border-t border-zinc-800 bg-zinc-900 shrink-0">
          <div className="relative">
            <input 
              type="text" 
              value={inputText}
              onChange={(e) => setInputText(e.target.value)}
              onKeyDown={(e) => e.key === "Enter" && handleSendMessage()}
              placeholder="Message NOVA..." 
              disabled={isLoading}
              className="w-full bg-zinc-800 border border-zinc-700 text-sm text-zinc-200 rounded-lg py-3 pl-4 pr-12 focus:outline-none focus:border-zinc-500"
            />
            <button 
              onClick={handleSendMessage}
              disabled={!inputText.trim() || isLoading}
              className="absolute right-2 top-1/2 -translate-y-1/2 p-2 text-zinc-400 hover:text-zinc-200 disabled:opacity-50 hover:bg-zinc-700 rounded-md"
            >
              <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="m22 2-7 20-4-9-9-4Z"/><path d="M22 2 11 13"/></svg>
            </button>
          </div>
        </footer>
      )}
    </div>
  );
}
