import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";

type AppMode = "Active" | "Focus" | "Break" | "Assessment" | "Paused";
type ViewMode = "Chat" | "Memory" | "Today";

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

export default function App() {
  const [mode, setMode] = useState<AppMode>("Active");
  const [view, setView] = useState<ViewMode>("Today");
  
  // Chat state
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [inputText, setInputText] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const [chatError, setChatError] = useState<string | null>(null);
  const [pendingDeletionId, setPendingDeletionId] = useState<string | null>(null);
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

  useEffect(() => {
    invoke<AppMode>("get_app_mode").then(setMode).catch(console.error);
  }, []);

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
      const response = await invoke<{text: string, pending_deletion: string | null}>("send_message", { message: userMessage.content });
      setMessages((prev) => [...prev, { role: "assistant", content: response.text }]);
      
      if (response.pending_deletion) {
        setPendingDeletionId(response.pending_deletion);
      }
    } catch (err) {
      console.error("Chat Error:", err);
      let errMsg = "An unexpected error occurred.";
      if (typeof err === "string") {
        if (err.includes("GEMINI_API_KEY")) errMsg = "NOVA needs a Gemini API key to talk. Please configure it in the .env file.";
        else if (err.includes("Assessment Mode")) errMsg = "AI operations are blocked in Assessment Mode.";
        else if (err.includes("Network")) errMsg = "Network error. Please check your connection.";
        else errMsg = "AI Provider Error. Service might be unavailable.";
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
    setPendingDeletionId(null);
    if (confirm) {
      try {
        await invoke("delete_memory", { id });
        await sendMessageCore("I confirmed the deletion.");
      } catch (err) {
        console.error("Failed to delete memory via AI:", err);
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
      console.error(e);
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
      const res = await invoke<Task[]>("list_tasks", { date: getTodayDateStr() });
      setTasks(res);
    } catch (e) {
      console.error(e);
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

  const moveTask = async (index: number, direction: -1 | 1) => {
    if (index + direction < 0 || index + direction >= tasks.length) return;
    const newTasks = [...tasks];
    const temp = newTasks[index];
    newTasks[index] = newTasks[index + direction];
    newTasks[index + direction] = temp;
    
    // Optimistic update
    setTasks(newTasks);
    
    try {
      await invoke("reorder_tasks", { 
        date: getTodayDateStr(), 
        ordered_ids: newTasks.map(t => t.id) 
      });
    } catch (e: any) {
      setTaskError(e.toString());
      loadTasks(); // revert on fail
    }
  };

  // Progress Calculation
  const totalTasks = tasks.length;
  const completedTasks = tasks.filter(t => t.status === "Completed").length;
  const remainingTasks = totalTasks - completedTasks;
  const progressPercent = totalTasks === 0 ? 0 : Math.round((completedTasks / totalTasks) * 100);

  return (
    <div className="flex flex-col h-screen bg-zinc-900 text-zinc-100 font-sans antialiased overflow-hidden selection:bg-zinc-700">
      
      {/* Header */}
      <header className="flex items-center justify-between px-4 py-3 border-b border-zinc-800 bg-zinc-900/80 backdrop-blur-md z-10 shrink-0">
        <div className="flex items-center space-x-4">
          <div className="flex items-center space-x-2 mr-4">
            <div className="w-2.5 h-2.5 rounded-full bg-emerald-500 shadow-[0_0_8px_rgba(16,185,129,0.5)]"></div>
            <h1 className="font-semibold text-zinc-100 tracking-wide text-sm">NOVA</h1>
          </div>
          <button 
            onClick={() => setView("Today")}
            className={`text-sm font-medium transition-colors ${view === "Today" ? "text-emerald-400" : "text-zinc-500 hover:text-zinc-300"}`}
          >
            Today
          </button>
          <button 
            onClick={() => setView("Chat")}
            className={`text-sm font-medium transition-colors ${view === "Chat" ? "text-emerald-400" : "text-zinc-500 hover:text-zinc-300"}`}
          >
            Chat
          </button>
          <button 
            onClick={() => setView("Memory")}
            className={`text-sm font-medium transition-colors ${view === "Memory" ? "text-emerald-400" : "text-zinc-500 hover:text-zinc-300"}`}
          >
            Memory
          </button>
        </div>
        <div className="flex items-center space-x-2">
          <span className="text-xs font-medium px-2.5 py-1 rounded-md bg-zinc-800 text-zinc-300">
            {mode}
          </span>
        </div>
      </header>

      {/* Main Content Area */}
      <main className="flex-1 overflow-y-auto p-4 flex flex-col gap-6 relative">
        
        {view === "Chat" && (
          <>
            {messages.length === 0 && (
              <div className="space-y-6">
                <section className="space-y-1">
                  <h2 className="text-xl font-medium text-zinc-100">Good morning.</h2>
                  <p className="text-sm text-zinc-400">Your systems are running optimally.</p>
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
                  <p className="text-sm text-zinc-200 mb-3 text-center">NOVA is asking for permission to delete a memory. Are you sure?</p>
                  <div className="flex space-x-3 w-full justify-center">
                    <button 
                      onClick={() => handleConfirmDeleteAI(false)}
                      className="px-4 py-1.5 text-sm font-medium text-zinc-300 bg-zinc-700 hover:bg-zinc-600 rounded-lg transition-colors flex-1 max-w-[120px]"
                    >
                      Cancel
                    </button>
                    <button 
                      onClick={() => handleConfirmDeleteAI(true)}
                      className="px-4 py-1.5 text-sm font-medium text-white bg-red-600 hover:bg-red-500 rounded-lg shadow-sm shadow-red-900/50 transition-colors flex-1 max-w-[120px]"
                    >
                      Delete
                    </button>
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

            <div className="bg-zinc-800/50 rounded-xl p-4 mb-6 border border-zinc-700/50">
              <div className="flex justify-between text-sm mb-2">
                <span className="text-zinc-400">Progress</span>
                <span className="text-emerald-400 font-medium">{progressPercent}%</span>
              </div>
              <div className="w-full bg-zinc-700 rounded-full h-2">
                <div className="bg-emerald-500 h-2 rounded-full transition-all duration-500" style={{ width: `${progressPercent}%` }}></div>
              </div>
              <div className="flex justify-between mt-3 text-xs text-zinc-500">
                <span>{totalTasks} total</span>
                <span>{completedTasks} completed</span>
                <span>{remainingTasks} remaining</span>
              </div>
            </div>

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
                  {tasks.map((task, idx) => (
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
                          </div>
                        )}
                      </div>

                      <div className="flex flex-col opacity-0 group-hover:opacity-100 transition-opacity space-y-1">
                        <button onClick={() => moveTask(idx, -1)} disabled={idx === 0} className="text-zinc-500 hover:text-zinc-300 disabled:opacity-30">
                          <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="m18 15-6-6-6 6"/></svg>
                        </button>
                        <button onClick={() => moveTask(idx, 1)} disabled={idx === tasks.length - 1} className="text-zinc-500 hover:text-zinc-300 disabled:opacity-30">
                          <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="m6 9 6 6 6-6"/></svg>
                        </button>
                      </div>

                      <div className="opacity-0 group-hover:opacity-100 transition-opacity flex space-x-2 shrink-0">
                        <button onClick={() => deleteTask(task.id)} className="p-1 text-red-500/70 hover:text-red-400 rounded transition-colors hover:bg-red-900/20">
                          <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M3 6h18"/><path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6"/><path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2"/></svg>
                        </button>
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </div>

            {/* Modal for Create/Edit Task */}
            {isTaskFormOpen && (
              <div className="absolute inset-0 z-20 bg-zinc-900/90 backdrop-blur-sm flex items-center justify-center p-4">
                <div className="bg-zinc-800 border border-zinc-700 rounded-xl p-5 w-full max-w-md shadow-2xl flex flex-col">
                  <h3 className="text-lg font-medium text-zinc-100 mb-4">{taskEditingId ? "Edit Task" : "New Task"}</h3>
                  
                  {taskError && (
                    <div className="bg-red-900/20 border border-red-900/50 rounded-lg p-2 mb-4 text-xs text-red-400">
                      {taskError}
                    </div>
                  )}

                  <label className="text-xs text-zinc-400 mb-1">Title</label>
                  <input 
                    type="text"
                    value={taskFormTitle}
                    onChange={(e) => setTaskFormTitle(e.target.value)}
                    className="w-full bg-zinc-900 border border-zinc-700 text-sm text-zinc-200 rounded-lg py-2 px-3 mb-4 focus:outline-none focus:border-emerald-500"
                    placeholder="E.g., React Fundamentals"
                    autoFocus
                  />

                  <label className="text-xs text-zinc-400 mb-1">Description (Optional)</label>
                  <textarea 
                    value={taskFormDesc}
                    onChange={(e) => setTaskFormDesc(e.target.value)}
                    className="w-full bg-zinc-900 border border-zinc-700 text-sm text-zinc-200 rounded-lg py-2 px-3 mb-4 focus:outline-none focus:border-emerald-500 min-h-[60px] resize-none"
                    placeholder="Details..."
                  />

                  <div className="flex gap-4 mb-4">
                    <div className="flex-1">
                      <label className="text-xs text-zinc-400 mb-1">Date</label>
                      <input 
                        type="date"
                        value={taskFormDate}
                        onChange={(e) => setTaskFormDate(e.target.value)}
                        className="w-full bg-zinc-900 border border-zinc-700 text-sm text-zinc-200 rounded-lg py-1.5 px-3 focus:outline-none focus:border-emerald-500"
                      />
                    </div>
                    <div className="w-1/3">
                      <label className="text-xs text-zinc-400 mb-1">Est. Minutes</label>
                      <input 
                        type="number"
                        value={taskFormEst}
                        onChange={(e) => setTaskFormEst(e.target.value)}
                        placeholder="0"
                        min="0"
                        className="w-full bg-zinc-900 border border-zinc-700 text-sm text-zinc-200 rounded-lg py-1.5 px-3 focus:outline-none focus:border-emerald-500"
                      />
                    </div>
                  </div>

                  <label className="text-xs text-zinc-400 mb-1">Priority</label>
                  <select 
                    value={taskFormPriority}
                    onChange={(e) => setTaskFormPriority(e.target.value as TaskPriority)}
                    className="w-full bg-zinc-900 border border-zinc-700 text-sm text-zinc-200 rounded-lg py-2 px-3 mb-5 focus:outline-none focus:border-emerald-500"
                  >
                    <option value="Low">Low</option>
                    <option value="Medium">Medium</option>
                    <option value="High">High</option>
                  </select>

                  <div className="flex justify-end space-x-3">
                    <button onClick={() => setIsTaskFormOpen(false)} className="text-sm text-zinc-400 hover:text-zinc-200 transition-colors">
                      Cancel
                    </button>
                    <button onClick={saveTask} className="text-sm bg-emerald-600 hover:bg-emerald-500 text-white py-1.5 px-4 rounded-lg transition-colors">
                      Save
                    </button>
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

            <p className="text-xs text-zinc-500 mb-4">Your memories are stored locally on this device.</p>

            <input 
              type="text" 
              placeholder="Search memories..." 
              value={searchQuery}
              onChange={handleSearchChange}
              className="w-full bg-zinc-800/50 border border-zinc-700 text-sm text-zinc-200 rounded-lg py-2 pl-3 mb-4 focus:outline-none focus:border-emerald-500 focus:ring-1 focus:ring-emerald-500 transition-all"
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
                    <div className="mt-2 text-[10px] text-zinc-600">
                      Updated: {new Date(mem.updated_at * 1000).toLocaleString()}
                    </div>
                  </div>
                ))
              )}
            </div>

            {/* Modal for Create/Edit */}
            {isFormOpen && (
              <div className="absolute inset-0 z-20 bg-zinc-900/90 backdrop-blur-sm flex items-center justify-center p-4">
                <div className="bg-zinc-800 border border-zinc-700 rounded-xl p-5 w-full max-w-md shadow-2xl flex flex-col">
                  <h3 className="text-lg font-medium text-zinc-100 mb-4">{editingId ? "Edit Memory" : "New Memory"}</h3>
                  
                  <label className="text-xs text-zinc-400 mb-1">Category</label>
                  <select 
                    value={formCategory}
                    onChange={(e) => setFormCategory(e.target.value as MemoryCategory)}
                    className="w-full bg-zinc-900 border border-zinc-700 text-sm text-zinc-200 rounded-lg py-2 px-3 mb-4 focus:outline-none focus:border-emerald-500"
                  >
                    <option value="Temporary">Temporary</option>
                    <option value="UserPreference">User Preference</option>
                    <option value="Goal">Goal</option>
                    <option value="Study">Study</option>
                    <option value="Project">Project</option>
                    <option value="Important">Important</option>
                  </select>

                  <label className="text-xs text-zinc-400 mb-1">Content</label>
                  <textarea 
                    value={formContent}
                    onChange={(e) => setFormContent(e.target.value)}
                    className="w-full bg-zinc-900 border border-zinc-700 text-sm text-zinc-200 rounded-lg py-2 px-3 mb-4 focus:outline-none focus:border-emerald-500 min-h-[100px] resize-none"
                    placeholder="Enter memory content..."
                  />

                  <div className="flex justify-end space-x-3 mt-2">
                    <button onClick={() => setIsFormOpen(false)} className="text-sm text-zinc-400 hover:text-zinc-200 transition-colors">
                      Cancel
                    </button>
                    <button onClick={saveMemory} className="text-sm bg-emerald-600 hover:bg-emerald-500 text-white py-1.5 px-4 rounded-lg transition-colors">
                      Save
                    </button>
                  </div>
                </div>
              </div>
            )}
          </div>
        )}
      </main>

      {/* Footer / Chat Input */}
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
      )}

    </div>
  );
}
