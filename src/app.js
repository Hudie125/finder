// ==========================================================================
// Finder - Core Application Orchestrator
// Apple Liquid Glass UI, Global Shortcuts, Voice, AI, and Tauri IPC
// ==========================================================================

import { VoiceController } from './modules/voice.js';
import { AiAssistant } from './modules/ai.js';
import { SystemMonitorController } from './modules/monitor.js';

// Safe Tauri IPC Invoke Wrapper (gracefully falls back if in browser preview)
const invoke = window.__TAURI__ ? window.__TAURI__.core.invoke : async (cmd, args = {}) => {
  console.log(`[IPC Mock] ${cmd}`, args);
  return mockBackendResponse(cmd, args);
};

// ==========================================================================
// Application State
// ==========================================================================
const state = {
  activeTab: 'all',
  selectedIndex: 0,
  results: [],
  isVoiceActive: false,
  config: {
    hotkey: 'Right Alt',
    whisperEngine: 'groq',
    groqKey: '',
    aiProvider: 'pollinations',
    geminiKey: '',
    theme: 'liquid-dark',
    launchOnStartup: true
  },
  notes: []
};

// DOM References
const searchInput = document.getElementById('searchInput');
const clearBtn = document.getElementById('clearBtn');
const voiceBtn = document.getElementById('voiceBtn');
const voiceBanner = document.getElementById('voiceBanner');
const voiceStatusText = document.getElementById('voiceStatusText');
const voiceCancelBtn = document.getElementById('voiceCancelBtn');
const resultsList = document.getElementById('resultsList');
const categoryTabs = document.getElementById('categoryTabs');
const settingsBtn = document.getElementById('settingsBtn');
const settingsModal = document.getElementById('settingsModal');
const closeSettingsBtn = document.getElementById('closeSettingsBtn');
const saveSettingsBtn = document.getElementById('saveSettingsBtn');
const resetSettingsBtn = document.getElementById('resetSettingsBtn');
const weatherPill = document.getElementById('weatherPill');
const monitorPill = document.getElementById('monitorPill');
const aiView = document.getElementById('aiView');
const monitorView = document.getElementById('monitorView');
const notesView = document.getElementById('notesView');
const newNoteInput = document.getElementById('newNoteInput');
const addNoteBtn = document.getElementById('addNoteBtn');
const notesList = document.getElementById('notesList');
const liquidGlow = document.getElementById('liquidGlow');
const aiClearBtn = document.getElementById('aiClearBtn');

// Initialize Subsystems
const aiAssistant = new AiAssistant();
const systemMonitor = new SystemMonitorController({ invoke });
const voiceController = new VoiceController({
  onTranscription: (text, isFinal) => {
    searchInput.value = text;
    if (isFinal) {
      voiceStatusText.textContent = `Recognized: "${text}"`;
      setTimeout(() => {
        voiceController.stop();
        voiceBanner.classList.remove('active');
        voiceBtn.classList.remove('recording');
      }, 700);
    }
  },
  onCommand: (cmd) => executeVoiceCommand(cmd),
  onStatusChange: (status) => {
    voiceStatusText.textContent = status;
  }
});

// ==========================================================================
// Bootstrap
// ==========================================================================
async function init() {
  loadStoredSettings();
  systemMonitor.start();
  setupEventListeners();
  loadNotes();

  // Run initial default suggestions/frequent apps
  performSearch('');
}

// ==========================================================================
// Event Listeners & Interaction
// ==========================================================================
function setupEventListeners() {
  // Ambient Liquid Light Tracking
  window.addEventListener('mousemove', (e) => {
    if (!liquidGlow) return;
    const x = (e.clientX / window.innerWidth) * 60;
    const y = (e.clientY / window.innerHeight) * 30;
    liquidGlow.style.transform = `translate(${x}px, ${y}px)`;
  });

  // Search Input Handler (Debounced)
  let searchDebounce = null;
  searchInput.addEventListener('input', () => {
    const query = searchInput.value;
    clearBtn.style.opacity = query ? '1' : '0.4';

    if (query.startsWith('?') || query.startsWith('ai ') || state.activeTab === 'ai') {
      // Direct AI Mode
      return;
    }

    clearTimeout(searchDebounce);
    searchDebounce = setTimeout(() => {
      performSearch(query);
    }, 120);
  });

  // Global Keydown Handler
  window.addEventListener('keydown', (e) => {
    // Escape: Clear or Hide
    if (e.key === 'Escape') {
      if (settingsModal.style.display !== 'none') {
        settingsModal.style.display = 'none';
        return;
      }
      if (searchInput.value) {
        searchInput.value = '';
        clearBtn.style.opacity = '0.4';
        performSearch('');
        return;
      }
      invoke('hide_window');
      return;
    }

    // Ctrl+M: Voice / Whisper toggle
    if (e.ctrlKey && (e.key === 'm' || e.key === 'M' || e.key === 'ь')) {
      e.preventDefault();
      toggleVoice();
      return;
    }

    // Ctrl+,: Settings
    if (e.ctrlKey && e.key === ',') {
      e.preventDefault();
      openSettings();
      return;
    }

    // Navigation in Results
    if (state.activeTab !== 'ai' && state.activeTab !== 'monitor' && state.activeTab !== 'notes') {
      if (e.key === 'ArrowDown') {
        e.preventDefault();
        navigateResults(1);
        return;
      }
      if (e.key === 'ArrowUp') {
        e.preventDefault();
        navigateResults(-1);
        return;
      }
      if (e.key === 'Enter') {
        e.preventDefault();
        if (state.activeTab === 'ai' || searchInput.value.startsWith('?') || searchInput.value.startsWith('ai ')) {
          submitAiQuery();
        } else {
          activateSelectedItem(e.ctrlKey); // Ctrl+Enter = Launch as Admin
        }
        return;
      }
    } else if (state.activeTab === 'ai' && e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      submitAiQuery();
    }
  });

  // Clear Input Button
  clearBtn.addEventListener('click', () => {
    searchInput.value = '';
    clearBtn.style.opacity = '0.4';
    searchInput.focus();
    performSearch('');
  });

  // Voice Recording Button
  voiceBtn.addEventListener('click', toggleVoice);
  voiceCancelBtn.addEventListener('click', () => {
    voiceController.stop();
    voiceBanner.classList.remove('active');
    voiceBtn.classList.remove('recording');
  });

  // Category Tabs
  categoryTabs.querySelectorAll('.tab-btn').forEach(btn => {
    btn.addEventListener('click', () => {
      categoryTabs.querySelectorAll('.tab-btn').forEach(b => b.classList.remove('active'));
      btn.classList.add('active');
      const tab = btn.getAttribute('data-tab');
      switchTab(tab);
    });
  });

  // Header Widgets Clicks
  weatherPill.addEventListener('click', () => {
    searchInput.value = '>weather';
    performSearch('>weather');
    switchTab('commands');
  });

  monitorPill.addEventListener('click', () => {
    switchTab('monitor');
  });

  // AI Prompt Chips
  document.querySelectorAll('.prompt-chip').forEach(chip => {
    chip.addEventListener('click', () => {
      const prompt = chip.getAttribute('data-prompt');
      searchInput.value = prompt;
      switchTab('ai');
      aiAssistant.ask(prompt, state.config);
    });
  });

  if (aiClearBtn) {
    aiClearBtn.addEventListener('click', () => aiAssistant.clear());
  }

  // Notes View Handlers
  addNoteBtn.addEventListener('click', createNote);
  newNoteInput.addEventListener('keydown', (e) => {
    if (e.key === 'Enter') createNote();
  });

  // Settings Modal Handlers
  settingsBtn.addEventListener('click', openSettings);
  closeSettingsBtn.addEventListener('click', () => settingsModal.style.display = 'none');
  settingsModal.addEventListener('click', (e) => {
    if (e.target === settingsModal) settingsModal.style.display = 'none';
  });
  saveSettingsBtn.addEventListener('click', saveSettingsFromModal);
  resetSettingsBtn.addEventListener('click', resetSettingsToDefault);
  setupHotkeyRecorder();

  // Tauri Backend Events
  if (window.__TAURI__ && window.__TAURI__.event) {
    try {
      window.__TAURI__.event.listen('focus-search', () => {
        searchInput.focus();
        searchInput.select();
      });
      window.__TAURI__.event.listen('open-settings', () => {
        openSettings();
      });
    } catch (_) {}
  }
}

// ==========================================================================
// Tabs & Views Management
// ==========================================================================
function switchTab(tab) {
  state.activeTab = tab;

  // Toggle dedicated views
  aiView.style.display = tab === 'ai' ? 'flex' : 'none';
  monitorView.style.display = tab === 'monitor' ? 'block' : 'none';
  notesView.style.display = tab === 'notes' ? 'block' : 'none';
  resultsList.style.display = (tab === 'ai' || tab === 'monitor' || tab === 'notes') ? 'none' : 'flex';

  if (tab === 'ai') {
    searchInput.placeholder = 'Ask AI anything (Markdown, code, answers)...';
  } else if (tab === 'notes') {
    searchInput.placeholder = 'Search notes or use !note <text>...';
    loadNotes();
  } else if (tab === 'monitor') {
    searchInput.placeholder = 'Live System Telemetry & Performance';
  } else {
    searchInput.placeholder = 'Search apps, files, commands, math, weather or ask AI (?)...';
    performSearch(searchInput.value);
  }
}

// ==========================================================================
// Search Core & Results Rendering
// ==========================================================================
async function performSearch(query) {
  const trimmed = query.trim();

  // Instant Check: Special Math Calculator
  const mathResult = tryEvaluateMath(trimmed);
  if (mathResult !== null) {
    state.results = [{
      title: `${trimmed} = ${mathResult}`,
      description: 'Calculator result - Press Enter to copy to clipboard',
      action: () => copyToClipboard(mathResult.toString()),
      category: 'Math',
      icon: '🧮'
    }];
    state.selectedIndex = 0;
    renderCalcCard(trimmed, mathResult);
    return;
  }

  // Instant Check: Currency Conversion e.g. "100 usd in rub"
  const currencyMatch = trimmed.match(/^(\d+(?:\.\d+)?)\s*([a-zA-Z]{3})\s+(?:in|to)\s+([a-zA-Z]{3})$/i);
  if (currencyMatch) {
    renderCurrencyCard(currencyMatch[1], currencyMatch[2].toUpperCase(), currencyMatch[3].toUpperCase());
    return;
  }

  // Instant Check: Quick Note Add "!note <text>"
  if (trimmed.startsWith('!note ')) {
    const noteText = trimmed.substring(6).trim();
    if (noteText) {
      renderActionInfoCard('Save Note', `Press Enter to save note: "${noteText}"`, () => {
        saveNoteDirect(noteText);
      });
      return;
    }
  }

  try {
    const data = await invoke('search', { query: trimmed, category: state.activeTab });
    state.results = data || [];
    state.selectedIndex = 0;
    renderResultsList(state.results);
  } catch (err) {
    console.warn('Search failed:', err);
  }
}

function renderResultsList(items) {
  resultsList.innerHTML = '';
  if (!items || items.length === 0) {
    resultsList.innerHTML = `
      <div style="text-align: center; padding: 36px 0; color: var(--text-tertiary); font-size: 13.5px;">
        No results found. Type <b>?</b> to ask AI, <b>></b> for system commands, or <b>!note</b> to save notes.
      </div>
    `;
    return;
  }

  items.forEach((item, index) => {
    const el = document.createElement('div');
    el.className = `result-item ${index === state.selectedIndex ? 'selected' : ''}`;
    el.setAttribute('role', 'option');

    // Icon & Type mapping
    let icon = item.icon || '✦';
    if (!item.icon) {
      if (item.category === 'Apps') icon = '🚀';
      else if (item.category === 'Windows') icon = '🪟';
      else if (item.category === 'Files') icon = '📄';
      else if (item.category === 'Commands') icon = '⚡';
      else if (item.category === 'Web') icon = '🌐';
      else if (item.category === 'Notes') icon = '📝';
    }

    el.innerHTML = `
      <div class="item-left">
        <div class="item-icon-box">${icon}</div>
        <div class="item-info">
          <div class="item-title-row">
            <span class="item-title">${escapeHtml(item.title)}</span>
            <span class="item-badge">${escapeHtml(item.category || 'App')}</span>
          </div>
          <span class="item-desc">${escapeHtml(item.description || item.path || '')}</span>
          ${item.snippet ? `<div class="snippet-preview">${escapeHtml(item.snippet)}</div>` : ''}
        </div>
      </div>
      <div class="item-right">
        <span class="item-action-hint">${item.category === 'Apps' ? 'Ctrl+↵ Admin' : '↵ Execute'}</span>
        <span class="item-key-badge">↵</span>
      </div>
    `;

    el.addEventListener('click', (e) => {
      state.selectedIndex = index;
      updateSelection();
      activateSelectedItem(e.ctrlKey);
    });

    resultsList.appendChild(el);
  });
}

function navigateResults(delta) {
  if (state.results.length === 0) return;
  state.selectedIndex = (state.selectedIndex + delta + state.results.length) % state.results.length;
  updateSelection();
}

function updateSelection() {
  const items = resultsList.querySelectorAll('.result-item');
  items.forEach((el, idx) => {
    const isSelected = idx === state.selectedIndex;
    el.classList.toggle('selected', isSelected);
    if (isSelected) {
      el.scrollIntoView({ block: 'nearest', behavior: 'smooth' });
    }
  });
}

async function activateSelectedItem(asAdmin = false) {
  const item = state.results[state.selectedIndex];
  if (!item) return;

  if (item.action) {
    item.action();
    return;
  }

  if (item.category === 'Apps') {
    await invoke('launch_app', { path: item.path || item.cmd, asAdmin });
    invoke('hide_window');
  } else if (item.category === 'Windows') {
    await invoke('focus_window', { hwnd: item.hwnd });
    invoke('hide_window');
  } else if (item.category === 'Commands') {
    await invoke('execute_command', { command: item.cmd || item.title });
  } else if (item.category === 'Files') {
    await invoke('launch_app', { path: item.path, asAdmin: false });
    invoke('hide_window');
  } else if (item.category === 'Web' || item.url) {
    window.open(item.url || item.path, '_blank');
  }
}

// ==========================================================================
// Math Calculator & Currency Converter
// ==========================================================================
function tryEvaluateMath(expr) {
  if (!expr || expr.length < 2) return null;
  // Allow numbers, basic operators, Math functions
  if (!/^[0-9+\-*/^().,% \t]|sqrt|sin|cos|tan|pi|abs|pow|log/i.test(expr)) return null;
  if (!/[0-9]/.test(expr)) return null;

  try {
    let sanitized = expr
      .replace(/\^/g, '**')
      .replace(/pi/gi, 'Math.PI')
      .replace(/sqrt/gi, 'Math.sqrt')
      .replace(/sin/gi, 'Math.sin')
      .replace(/cos/gi, 'Math.cos')
      .replace(/tan/gi, 'Math.tan')
      .replace(/abs/gi, 'Math.abs')
      .replace(/log/gi, 'Math.log');

    // Safe mathematical evaluation using Function constructor with restricted scope
    const result = Function(`"use strict"; return (${sanitized});`)();
    if (typeof result === 'number' && !isNaN(result) && isFinite(result)) {
      return result;
    }
  } catch (_) {}
  return null;
}

function renderCalcCard(expr, result) {
  resultsList.innerHTML = `
    <div class="calc-card" id="calcCard">
      <div class="calc-expression">${escapeHtml(expr)} =</div>
      <div class="calc-result">${result.toLocaleString('en-US', { maximumFractionDigits: 8 })}</div>
      <div class="calc-footer">Press <b>Enter</b> or <b>Ctrl+C</b> to copy result to clipboard</div>
    </div>
  `;
  const card = document.getElementById('calcCard');
  if (card) {
    card.addEventListener('click', () => copyToClipboard(result.toString()));
  }
}

function renderCurrencyCard(amount, from, to) {
  const approxRates = {
    'USD_RUB': 96.5, 'EUR_RUB': 104.2, 'USD_EUR': 0.92, 'EUR_USD': 1.08,
    'USD_KZT': 490, 'USD_UAH': 41.5, 'EUR_GBP': 0.85, 'GBP_USD': 1.30
  };
  const key = `${from}_${to}`;
  const reverseKey = `${to}_${from}`;
  let rate = approxRates[key] || (approxRates[reverseKey] ? 1 / approxRates[reverseKey] : 1.0);
  const converted = (parseFloat(amount) * rate).toFixed(2);

  resultsList.innerHTML = `
    <div class="currency-card">
      <div class="calc-expression">${amount} ${from} in ${to} ≈</div>
      <div class="calc-result">${converted} ${to}</div>
      <div class="calc-footer">Estimated exchange rate: 1 ${from} = ${rate.toFixed(4)} ${to} (Press Enter to copy)</div>
    </div>
  `;
}

function renderActionInfoCard(title, subtitle, action) {
  resultsList.innerHTML = `
    <div class="result-item selected">
      <div class="item-left">
        <div class="item-icon-box">📌</div>
        <div class="item-info">
          <div class="item-title-row"><span class="item-title">${escapeHtml(title)}</span></div>
          <span class="item-desc">${escapeHtml(subtitle)}</span>
        </div>
      </div>
      <div class="item-right"><span class="item-key-badge">↵ Save</span></div>
    </div>
  `;
  state.results = [{ action }];
  state.selectedIndex = 0;
}

// ==========================================================================
// Voice & Whisper Execution
// ==========================================================================
function toggleVoice() {
  state.isVoiceActive = !state.isVoiceActive;
  if (state.isVoiceActive) {
    voiceBanner.classList.add('active');
    voiceBtn.classList.add('recording');
    voiceController.start(state.config.whisperEngine, state.config.groqKey);
  } else {
    voiceController.stop();
    voiceBanner.classList.remove('active');
    voiceBtn.classList.remove('recording');
  }
}

async function executeVoiceCommand(cmd) {
  console.log('[Voice Command Received]:', cmd);
  if (cmd.type === 'launch_app') {
    searchInput.value = cmd.target;
    await performSearch(cmd.target);
    if (state.results.length > 0) {
      activateSelectedItem(false);
    }
  } else if (cmd.type === 'system_command') {
    await invoke('execute_command', { command: cmd.command });
  } else if (cmd.type === 'weather') {
    switchTab('commands');
    searchInput.value = `>weather ${cmd.city}`;
    performSearch(`>weather ${cmd.city}`);
  } else if (cmd.type === 'timer') {
    await invoke('execute_command', { command: `timer ${cmd.duration}` });
  } else if (cmd.type === 'note') {
    saveNoteDirect(cmd.text);
  } else if (cmd.type === 'ai') {
    switchTab('ai');
    aiAssistant.ask(cmd.prompt, state.config);
  } else if (cmd.type === 'search') {
    searchInput.value = cmd.query;
    performSearch(cmd.query);
  }
}

// ==========================================================================
// AI Assistant Triggers
// ==========================================================================
function submitAiQuery() {
  const query = searchInput.value.replace(/^(\?|ai\s+)/i, '').trim();
  if (!query) return;
  switchTab('ai');
  aiAssistant.ask(query, state.config);
  searchInput.value = '';
}

// ==========================================================================
// Notes Persistence & Management
// ==========================================================================
async function loadNotes() {
  try {
    const notes = await invoke('get_notes');
    state.notes = notes || JSON.parse(localStorage.getItem('finder_notes') || '[]');
    renderNotesList();
  } catch (_) {
    state.notes = JSON.parse(localStorage.getItem('finder_notes') || '[]');
    renderNotesList();
  }
}

function renderNotesList() {
  if (!notesList) return;
  notesList.innerHTML = '';
  if (state.notes.length === 0) {
    notesList.innerHTML = `<div style="text-align:center; padding:20px; color:var(--text-tertiary);">No notes saved yet. Use !note <text> in search.</div>`;
    return;
  }
  state.notes.forEach((n, idx) => {
    const el = document.createElement('div');
    el.className = 'note-item';
    el.innerHTML = `
      <span class="note-text">${escapeHtml(n.text)}</span>
      <span class="note-time">${n.timestamp ? new Date(n.timestamp).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }) : ''}</span>
      <button class="note-delete-btn" title="Delete note">✕</button>
    `;
    el.querySelector('.note-delete-btn').addEventListener('click', () => deleteNote(idx));
    notesList.appendChild(el);
  });
}

async function createNote() {
  const text = newNoteInput.value.trim();
  if (!text) return;
  saveNoteDirect(text);
  newNoteInput.value = '';
}

async function saveNoteDirect(text) {
  const newNote = { text, timestamp: new Date().toISOString() };
  state.notes.unshift(newNote);
  localStorage.setItem('finder_notes', JSON.stringify(state.notes));
  try { await invoke('save_note', { text }); } catch (_) {}
  renderNotesList();
}

async function deleteNote(idx) {
  state.notes.splice(idx, 1);
  localStorage.setItem('finder_notes', JSON.stringify(state.notes));
  try { await invoke('delete_note', { id: idx }); } catch (_) {}
  renderNotesList();
}

// ==========================================================================
// Settings Modal
// ==========================================================================
function openSettings() {
  document.getElementById('settingHotkey').value = state.config.hotkey;
  document.getElementById('settingWhisperEngine').value = state.config.whisperEngine;
  document.getElementById('settingGroqKey').value = state.config.groqKey;
  document.getElementById('settingAiProvider').value = state.config.aiProvider;
  document.getElementById('settingGeminiKey').value = state.config.geminiKey;
  document.getElementById('settingTheme').value = state.config.theme;
  document.getElementById('settingLaunchOnStartup').checked = state.config.launchOnStartup;

  settingsModal.style.display = 'flex';
}

let isRecordingHotkey = false;

function setupHotkeyRecorder() {
  const recordBtn = document.getElementById('recordHotkeyBtn');
  const hotkeyInput = document.getElementById('settingHotkey');
  if (!recordBtn || !hotkeyInput) return;

  recordBtn.addEventListener('click', (e) => {
    e.preventDefault();
    isRecordingHotkey = true;
    recordBtn.textContent = 'Press keys...';
    recordBtn.classList.add('recording');
    hotkeyInput.value = 'Listening...';
  });

  window.addEventListener('keydown', (e) => {
    if (!isRecordingHotkey) return;
    e.preventDefault();
    e.stopPropagation();

    if (e.code === 'AltRight') {
      hotkeyInput.value = 'Right Alt';
      stopRecording();
      return;
    }
    if (e.code === 'AltLeft') {
      hotkeyInput.value = 'Left Alt';
      stopRecording();
      return;
    }
    if (e.key === 'Escape') {
      hotkeyInput.value = state.config.hotkey || 'Right Alt';
      stopRecording();
      return;
    }

    if (['Control', 'Shift', 'Alt', 'Meta'].includes(e.key)) {
      return;
    }

    let parts = [];
    if (e.ctrlKey) parts.push('Ctrl');
    if (e.altKey) parts.push('Alt');
    if (e.shiftKey) parts.push('Shift');
    if (e.metaKey) parts.push('Super');

    let keyName = e.code.replace('Key', '').replace('Digit', '');
    if (e.code === 'Space') keyName = 'Space';
    else if (e.code === 'Backquote') keyName = '~';
    else if (e.code.startsWith('F') && !isNaN(e.code.slice(1))) keyName = e.code;

    parts.push(keyName);
    hotkeyInput.value = parts.join('+');
    stopRecording();
  }, true);

  function stopRecording() {
    isRecordingHotkey = false;
    recordBtn.textContent = 'Record Shortcut';
    recordBtn.classList.remove('recording');
  }
}

function saveSettingsFromModal() {
  state.config = {
    hotkey: document.getElementById('settingHotkey').value,
    whisperEngine: document.getElementById('settingWhisperEngine').value,
    groqKey: document.getElementById('settingGroqKey').value,
    aiProvider: document.getElementById('settingAiProvider').value,
    geminiKey: document.getElementById('settingGeminiKey').value,
    theme: document.getElementById('settingTheme').value,
    launchOnStartup: document.getElementById('settingLaunchOnStartup').checked
  };

  localStorage.setItem('finder_settings', JSON.stringify(state.config));
  document.documentElement.setAttribute('data-theme', state.config.theme);
  invoke('save_settings', { settings: state.config });
  settingsModal.style.display = 'none';
}

function resetSettingsToDefault() {
  localStorage.removeItem('finder_settings');
  loadStoredSettings();
  openSettings();
}

function loadStoredSettings() {
  const saved = localStorage.getItem('finder_settings');
  if (saved) {
    try {
      state.config = { ...state.config, ...JSON.parse(saved) };
    } catch (_) {}
  }
  document.documentElement.setAttribute('data-theme', state.config.theme);
}

// ==========================================================================
// Utilities & Mock Fallbacks (for web browser dev preview)
// ==========================================================================
function copyToClipboard(text) {
  navigator.clipboard.writeText(text);
  searchInput.value = 'Copied to clipboard!';
  setTimeout(() => {
    searchInput.value = '';
    performSearch('');
  }, 1200);
}

function escapeHtml(str) {
  if (!str) return '';
  return str.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
}

function mockBackendResponse(cmd, args) {
  if (cmd === 'search') {
    const q = (args.query || '').toLowerCase();
    const mockApps = [
      { title: 'Google Chrome', category: 'Apps', description: 'Web Browser', cmd: 'chrome.exe', icon: '🌐' },
      { title: 'Visual Studio Code', category: 'Apps', description: 'Code Editor', cmd: 'code.exe', icon: '💻' },
      { title: 'Windows Terminal', category: 'Apps', description: 'Command Line', cmd: 'wt.exe', icon: '⚡' },
      { title: 'Task Manager', category: 'Commands', description: 'System Performance & Processes', cmd: 'taskmgr.exe', icon: '📊' },
      { title: 'Lock Workstation', category: 'Commands', description: 'Lock Windows session', cmd: 'lock', icon: '🔒' },
      { title: 'Snipping Tool', category: 'Commands', description: 'Screen capture', cmd: 'ms-screenclip:', icon: '✂️' },
      { title: 'Generate Password 24', category: 'Commands', description: 'Secure password', cmd: 'pw 24', icon: '🔑' },
      { title: 'Timer 25m (Pomodoro)', category: 'Commands', description: 'Focus timer', cmd: 'timer 25m', icon: '⏱️' }
    ];
    if (!q) return mockApps;
    return mockApps.filter(a => a.title.toLowerCase().includes(q) || a.description.toLowerCase().includes(q));
  }
  return null;
}

// Start app
window.addEventListener('DOMContentLoaded', init);
