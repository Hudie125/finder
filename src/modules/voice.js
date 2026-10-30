// ==========================================================================
// Finder - Voice & Whisper Speech Recognition Module
// Supports Groq Whisper API, Local Whisper, and Web Speech API Fallback
// Includes Smart Voice Command Execution
// ==========================================================================

export class VoiceController {
  constructor(options = {}) {
    this.onTranscription = options.onTranscription || (() => {});
    this.onCommand = options.onCommand || (() => {});
    this.onStatusChange = options.onStatusChange || (() => {});
    
    this.isRecording = false;
    this.mediaRecorder = null;
    this.audioChunks = [];
    this.audioContext = null;
    this.analyser = null;
    this.dataArray = null;
    this.animFrameId = null;
    this.canvas = document.getElementById('voiceCanvas');
    this.canvasCtx = this.canvas ? this.canvas.getContext('2d') : null;
    
    this.recognition = null;
    this.initWebSpeech();
  }

  initWebSpeech() {
    const SpeechRecognition = window.SpeechRecognition || window.webkitSpeechRecognition;
    if (SpeechRecognition) {
      this.recognition = new SpeechRecognition();
      this.recognition.continuous = false;
      this.recognition.interimResults = true;
      this.recognition.lang = navigator.language || 'ru-RU';

      this.recognition.onresult = (event) => {
        let interimTranscript = '';
        let finalTranscript = '';

        for (let i = event.resultIndex; i < event.results.length; ++i) {
          if (event.results[i].isFinal) {
            finalTranscript += event.results[i][0].transcript;
          } else {
            interimTranscript += event.results[i][0].transcript;
          }
        }

        const text = finalTranscript || interimTranscript;
        if (text) {
          this.onTranscription(text, Boolean(finalTranscript));
          if (finalTranscript) {
            this.handleVoiceCommand(finalTranscript);
          }
        }
      };

      this.recognition.onerror = (e) => {
        console.warn('WebSpeech Error:', e.error);
        this.onStatusChange('Speech recognition error: ' + e.error);
      };

      this.recognition.onend = () => {
        if (this.isRecording) {
          this.stop();
        }
      };
    }
  }

  async start(engine = 'groq', apiKey = '') {
    if (this.isRecording) return;
    this.isRecording = true;
    this.onStatusChange('Listening...');

    try {
      const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
      this.setupVisualizer(stream);

      if (engine === 'webspeech' && this.recognition) {
        this.recognition.start();
      } else {
        // Record audio blob for Whisper API
        this.audioChunks = [];
        this.mediaRecorder = new MediaRecorder(stream);
        this.mediaRecorder.ondataavailable = (e) => {
          if (e.data.size > 0) this.audioChunks.push(e.data);
        };
        this.mediaRecorder.onstop = async () => {
          const audioBlob = new Blob(this.audioChunks, { type: 'audio/webm' });
          await this.transcribeWithWhisper(audioBlob, engine, apiKey);
        };
        this.mediaRecorder.start();
      }
    } catch (err) {
      console.error('Audio capture failed:', err);
      this.onStatusChange('Microphone access denied');
      this.stop();
    }
  }

  stop() {
    if (!this.isRecording) return;
    this.isRecording = false;
    this.onStatusChange('Processing audio with Whisper...');

    if (this.mediaRecorder && this.mediaRecorder.state !== 'inactive') {
      this.mediaRecorder.stop();
    }
    if (this.recognition) {
      try { this.recognition.stop(); } catch (_) {}
    }
    if (this.animFrameId) {
      cancelAnimationFrame(this.animFrameId);
      this.animFrameId = null;
    }
    if (this.audioContext && this.audioContext.state !== 'closed') {
      try { this.audioContext.close(); } catch (_) {}
    }
    if (this.canvasCtx && this.canvas) {
      this.canvasCtx.clearRect(0, 0, this.canvas.width, this.canvas.height);
    }
  }

  setupVisualizer(stream) {
    if (!this.canvasCtx) return;
    try {
      this.audioContext = new (window.AudioContext || window.webkitAudioContext)();
      const source = this.audioContext.createMediaStreamSource(stream);
      this.analyser = this.audioContext.createAnalyser();
      this.analyser.fftSize = 64;
      source.connect(this.analyser);

      const bufferLength = this.analyser.frequencyBinCount;
      this.dataArray = new Uint8Array(bufferLength);

      const drawWave = () => {
        if (!this.isRecording) return;
        this.animFrameId = requestAnimationFrame(drawWave);
        this.analyser.getByteFrequencyData(this.dataArray);

        const ctx = this.canvasCtx;
        const width = this.canvas.width;
        const height = this.canvas.height;
        ctx.clearRect(0, 0, width, height);

        const barWidth = (width / bufferLength) * 1.8;
        let x = 0;

        for (let i = 0; i < bufferLength; i++) {
          const barHeight = (this.dataArray[i] / 255) * height * 0.9 + 2;
          const y = (height - barHeight) / 2;

          ctx.fillStyle = `rgba(255, 255, 255, ${0.4 + (this.dataArray[i] / 255) * 0.6})`;
          ctx.beginPath();
          ctx.roundRect(x, y, barWidth - 2, barHeight, 3);
          ctx.fill();

          x += barWidth;
        }
      };

      drawWave();
    } catch (e) {
      console.warn('Visualizer setup error:', e);
    }
  }

  async transcribeWithWhisper(audioBlob, engine, apiKey) {
    try {
      let transcriptionText = '';

      if (engine === 'groq' && apiKey) {
        // Groq Whisper API (whisper-large-v3, ultra fast)
        const formData = new FormData();
        formData.append('file', audioBlob, 'audio.webm');
        formData.append('model', 'whisper-large-v3');
        formData.append('temperature', '0');

        const res = await fetch('https://api.groq.com/openai/v1/audio/transcriptions', {
          method: 'POST',
          headers: {
            'Authorization': `Bearer ${apiKey}`
          },
          body: formData
        });

        if (!res.ok) throw new Error(`Groq Whisper error: ${res.statusText}`);
        const data = await res.json();
        transcriptionText = data.text;
      } else if (engine === 'local') {
        // Local Whisper API (e.g. whisper.cpp / Faster-Whisper server)
        const formData = new FormData();
        formData.append('file', audioBlob, 'audio.webm');

        const res = await fetch('http://localhost:8000/v1/audio/transcriptions', {
          method: 'POST',
          body: formData
        });
        if (!res.ok) throw new Error(`Local Whisper error: ${res.statusText}`);
        const data = await res.json();
        transcriptionText = data.text;
      } else {
        // Fallback: zero-key free Whisper endpoint via HuggingFace Inference API
        const res = await fetch('https://api-inference.huggingface.co/models/openai/whisper-large-v3-turbo', {
          method: 'POST',
          body: audioBlob
        });
        if (res.ok) {
          const data = await res.json();
          transcriptionText = data.text || '';
        }
      }

      if (transcriptionText) {
        this.onTranscription(transcriptionText.trim(), true);
        this.handleVoiceCommand(transcriptionText.trim());
      } else {
        this.onStatusChange('No speech recognized');
      }
    } catch (err) {
      console.warn('Whisper API failed, falling back:', err);
      this.onStatusChange('Whisper failed. Check API key in Settings.');
    }
  }

  // ==========================================================================
  // Smart Voice Command Parser
  // ==========================================================================
  handleVoiceCommand(rawText) {
    const text = rawText.toLowerCase().trim();

    // 1. App Launching: "открой <app>", "запусти <app>", "open <app>", "launch <app>"
    const launchMatch = text.match(/^(?:открой|запусти|включи|open|launch|run)\s+(.+)$/i);
    if (launchMatch) {
      const appName = launchMatch[1].replace(/[.!?,]+$/, '').trim();
      this.onCommand({ type: 'launch_app', target: appName });
      return;
    }

    // 2. Lock Workstation: "заблокируй", "лок", "lock pc", "lock screen"
    if (text.includes('заблокируй') || text.includes('заблокировать') || text.includes('lock pc') || text.includes('lock screen')) {
      this.onCommand({ type: 'system_command', command: 'lock' });
      return;
    }

    // 3. Sleep / Shutdown / Reboot
    if (text.includes('выключи пк') || text.includes('выключи компьютер') || text.includes('shutdown')) {
      this.onCommand({ type: 'system_command', command: 'shutdown' });
      return;
    }
    if (text.includes('перезагрузи') || text.includes('reboot')) {
      this.onCommand({ type: 'system_command', command: 'reboot' });
      return;
    }
    if (text.includes('спящий режим') || text.includes('уснуть') || text.includes('sleep')) {
      this.onCommand({ type: 'system_command', command: 'sleep' });
      return;
    }

    // 4. Weather: "какая погода", "погода в <город>", "weather in <city>"
    const weatherMatch = text.match(/(?:погода\s+в|погода|weather\s+in|weather)\s*(.*)/i);
    if (weatherMatch && (text.includes('погода') || text.includes('weather'))) {
      const city = weatherMatch[1].trim() || '';
      this.onCommand({ type: 'weather', city });
      return;
    }

    // 5. Timer: "таймер на 5 минут", "timer 10 minutes", "поставь таймер на..."
    const timerMatch = text.match(/(?:таймер\s+на|timer\s+for|timer)\s*(\d+)\s*(минут|мин|секунд|сек|часов|час|m|s|h)?/i);
    if (timerMatch) {
      const num = timerMatch[1];
      const unit = (timerMatch[2] || 'm').toLowerCase();
      let formatted = `${num}m`;
      if (unit.startsWith('с') || unit === 's') formatted = `${num}s`;
      if (unit.startsWith('ч') || unit === 'h') formatted = `${num}h`;
      this.onCommand({ type: 'timer', duration: formatted });
      return;
    }

    // 6. Notes: "заметка <текст>", "сохрани заметку <текст>", "note <text>"
    const noteMatch = text.match(/^(?:заметка|сохрани заметку|напомни|note)\s+(.+)$/i);
    if (noteMatch) {
      this.onCommand({ type: 'note', text: noteMatch[1].trim() });
      return;
    }

    // 7. AI Assistant: "спроси у нейросети...", "спроси у ии...", "ask ai..."
    const aiMatch = text.match(/^(?:спроси у нейросети|спроси у ии|спроси|ask ai|ask)\s+(.+)$/i);
    if (aiMatch) {
      this.onCommand({ type: 'ai', prompt: aiMatch[1].trim() });
      return;
    }

    // 8. Default: Enter as search query
    this.onCommand({ type: 'search', query: rawText });
  }
}
