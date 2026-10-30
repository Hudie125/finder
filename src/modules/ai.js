// ==========================================================================
// Finder - Multi-Provider AI Assistant Module
// Free Cloud AI (No Key Needed), Google Gemini, Groq, and Ollama
// ==========================================================================

export class AiAssistant {
  constructor(options = {}) {
    this.messagesContainer = document.getElementById('aiMessages');
    this.typingIndicator = document.getElementById('aiTyping');
    this.onStreamChunk = options.onStreamChunk || (() => {});
    this.history = [];
  }

  clear() {
    this.history = [];
    if (this.messagesContainer) {
      this.messagesContainer.innerHTML = `
        <div class="ai-msg assistant welcome-msg">
          <div class="msg-avatar">✦</div>
          <div class="msg-bubble">
            <p>Chat cleared. Ask me anything, generate code, or execute commands.</p>
          </div>
        </div>
      `;
    }
  }

  appendMessage(role, content) {
    if (!this.messagesContainer) return;
    const msgEl = document.createElement('div');
    msgEl.className = `ai-msg ${role}`;
    
    const avatar = document.createElement('div');
    avatar.className = 'msg-avatar';
    avatar.textContent = role === 'user' ? '👤' : '✦';

    const bubble = document.createElement('div');
    bubble.className = 'msg-bubble';
    bubble.innerHTML = this.renderMarkdown(content);

    msgEl.appendChild(avatar);
    msgEl.appendChild(bubble);
    this.messagesContainer.appendChild(msgEl);
    this.messagesContainer.scrollTop = this.messagesContainer.scrollHeight;

    return bubble;
  }

  showTyping(show = true) {
    if (this.typingIndicator) {
      this.typingIndicator.classList.toggle('active', show);
      if (show && this.messagesContainer) {
        this.messagesContainer.scrollTop = this.messagesContainer.scrollHeight;
      }
    }
  }

  async ask(prompt, config = {}) {
    const provider = config.provider || 'pollinations';
    const geminiKey = config.geminiKey || '';
    const groqKey = config.groqKey || '';

    this.appendMessage('user', prompt);
    this.history.push({ role: 'user', content: prompt });
    this.showTyping(true);

    const bubble = this.appendMessage('assistant', '');
    let fullResponse = '';

    try {
      if (provider === 'gemini' && geminiKey) {
        fullResponse = await this.queryGemini(prompt, geminiKey, (chunk) => {
          fullResponse += chunk;
          bubble.innerHTML = this.renderMarkdown(fullResponse);
          this.messagesContainer.scrollTop = this.messagesContainer.scrollHeight;
        });
      } else if (provider === 'groq' && groqKey) {
        fullResponse = await this.queryGroq(prompt, groqKey, (chunk) => {
          fullResponse += chunk;
          bubble.innerHTML = this.renderMarkdown(fullResponse);
          this.messagesContainer.scrollTop = this.messagesContainer.scrollHeight;
        });
      } else if (provider === 'ollama') {
        fullResponse = await this.queryOllama(prompt, (chunk) => {
          fullResponse += chunk;
          bubble.innerHTML = this.renderMarkdown(fullResponse);
          this.messagesContainer.scrollTop = this.messagesContainer.scrollHeight;
        });
      } else {
        // Free Cloud Intelligence (Pollinations AI text model - zero setup, zero cost)
        fullResponse = await this.queryPollinations(prompt, (chunk) => {
          fullResponse += chunk;
          bubble.innerHTML = this.renderMarkdown(fullResponse);
          this.messagesContainer.scrollTop = this.messagesContainer.scrollHeight;
        });
      }

      this.history.push({ role: 'assistant', content: fullResponse });
    } catch (err) {
      console.error('AI Query failed:', err);
      bubble.innerHTML = `<span style="color:#ff6b6b;">Error querying AI (${err.message}). Check connection or settings.</span>`;
    } finally {
      this.showTyping(false);
      this.attachCodeCopyButtons(bubble);
    }
  }

  // 1. Free Cloud Provider: Pollinations.ai Text API (No API Key Required!)
  async queryPollinations(prompt, onChunk) {
    const systemInstruction = 'You are Finder AI, a fast, concise, helpful assistant built into a Spotlight-style desktop tool. Format responses with clean markdown.';
    const messages = [
      { role: 'system', content: systemInstruction },
      ...this.history.slice(-4),
      { role: 'user', content: prompt }
    ];

    const res = await fetch('https://text.pollinations.ai/', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        messages,
        model: 'openai',
        stream: true
      })
    });

    if (!res.ok) throw new Error(`Status ${res.status}`);
    const reader = res.body.getReader();
    const decoder = new TextDecoder();
    let complete = '';

    while (true) {
      const { done, value } = await reader.read();
      if (done) break;
      const chunk = decoder.decode(value, { stream: true });
      complete += chunk;
      onChunk(chunk);
    }
    return complete;
  }

  // 2. Google Gemini 2.0 / 1.5 Flash Streaming API
  async queryGemini(prompt, apiKey, onChunk) {
    const url = `https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash:streamGenerateContent?alt=sse&key=${apiKey}`;
    const contents = this.history.slice(-6).map(m => ({
      role: m.role === 'user' ? 'user' : 'model',
      parts: [{ text: m.content }]
    }));
    contents.push({ role: 'user', parts: [{ text: prompt }] });

    const res = await fetch(url, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ contents })
    });

    if (!res.ok) throw new Error(`Gemini status ${res.status}`);
    const reader = res.body.getReader();
    const decoder = new TextDecoder();
    let complete = '';

    while (true) {
      const { done, value } = await reader.read();
      if (done) break;
      const textChunk = decoder.decode(value, { stream: true });
      const lines = textChunk.split('\n');
      for (const line of lines) {
        if (line.startsWith('data: ')) {
          try {
            const data = JSON.parse(line.substring(6));
            const text = data.candidates?.[0]?.content?.parts?.[0]?.text || '';
            if (text) {
              complete += text;
              onChunk(text);
            }
          } catch (_) {}
        }
      }
    }
    return complete;
  }

  // 3. Groq Streaming API (Llama 3.3 70B)
  async queryGroq(prompt, apiKey, onChunk) {
    const res = await fetch('https://api.groq.com/openai/v1/chat/completions', {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${apiKey}`,
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({
        model: 'llama-3.3-70b-versatile',
        messages: [
          { role: 'system', content: 'You are Finder AI desktop assistant. Be ultra-fast and precise.' },
          ...this.history.slice(-4),
          { role: 'user', content: prompt }
        ],
        stream: true
      })
    });

    if (!res.ok) throw new Error(`Groq status ${res.status}`);
    const reader = res.body.getReader();
    const decoder = new TextDecoder();
    let complete = '';

    while (true) {
      const { done, value } = await reader.read();
      if (done) break;
      const textChunk = decoder.decode(value, { stream: true });
      const lines = textChunk.split('\n');
      for (const line of lines) {
        if (line.startsWith('data: ') && line !== 'data: [DONE]') {
          try {
            const data = JSON.parse(line.substring(6));
            const text = data.choices?.[0]?.delta?.content || '';
            if (text) {
              complete += text;
              onChunk(text);
            }
          } catch (_) {}
        }
      }
    }
    return complete;
  }

  // 4. Local Ollama API
  async queryOllama(prompt, onChunk) {
    const res = await fetch('http://localhost:11434/api/generate', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        model: 'llama3',
        prompt,
        stream: true
      })
    });

    if (!res.ok) throw new Error(`Ollama status ${res.status}`);
    const reader = res.body.getReader();
    const decoder = new TextDecoder();
    let complete = '';

    while (true) {
      const { done, value } = await reader.read();
      if (done) break;
      const textChunk = decoder.decode(value, { stream: true });
      const lines = textChunk.split('\n').filter(Boolean);
      for (const line of lines) {
        try {
          const data = JSON.parse(line);
          if (data.response) {
            complete += data.response;
            onChunk(data.response);
          }
        } catch (_) {}
      }
    }
    return complete;
  }

  // Lightweight Safe Markdown Renderer
  renderMarkdown(text) {
    if (!text) return '';
    let html = text
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;');

    // Code blocks ```code```
    html = html.replace(/```([a-z0-9_-]*)\n([\s\S]*?)```/gi, (match, lang, code) => {
      return `<div class="code-block-wrapper"><div class="code-header"><span>${lang || 'code'}</span><button class="copy-code-btn" data-code="${encodeURIComponent(code)}">Copy</button></div><pre><code>${code}</code></pre></div>`;
    });

    // Inline code `code`
    html = html.replace(/`([^`]+)`/g, '<code>$1</code>');

    // Bold **text**
    html = html.replace(/\*\*([^*]+)\*\*/g, '<b>$1</b>');

    // Italic *text*
    html = html.replace(/\*([^*]+)\*/g, '<i>$1</i>');

    // Line breaks
    html = html.replace(/\n/g, '<br/>');

    return html;
  }

  attachCodeCopyButtons(container) {
    if (!container) return;
    container.querySelectorAll('.copy-code-btn').forEach(btn => {
      btn.addEventListener('click', (e) => {
        e.stopPropagation();
        const code = decodeURIComponent(btn.getAttribute('data-code') || '');
        navigator.clipboard.writeText(code).then(() => {
          btn.textContent = 'Copied!';
          setTimeout(() => btn.textContent = 'Copy', 1800);
        });
      });
    });
  }
}
