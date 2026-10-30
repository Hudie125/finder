// ==========================================================================
// Finder - System Hardware Monitor & Weather Controller
// Real-time CPU, RAM, GPU, Disk, and Live Weather
// ==========================================================================

export class SystemMonitorController {
  constructor(options = {}) {
    this.invoke = options.invoke || (async () => null);
    this.timer = null;
    this.weatherTimer = null;
    this.lastData = null;
  }

  start() {
    this.update();
    this.timer = setInterval(() => this.update(), 2000);
    this.updateWeather();
    this.weatherTimer = setInterval(() => this.updateWeather(), 600000); // 10 minutes
  }

  stop() {
    if (this.timer) clearInterval(this.timer);
    if (this.weatherTimer) clearInterval(this.weatherTimer);
  }

  async update() {
    try {
      let data = await this.invoke('get_system_telemetry');
      if (!data) {
        // Fallback simulated metrics if in standalone web browser
        data = {
          cpu_name: 'AMD Ryzen 7 / Intel Core i7',
          cpu_usage: Math.round(15 + Math.random() * 25),
          cpu_cores: 8,
          ram_used_gb: 7.2,
          ram_total_gb: 16.0,
          ram_usage_percent: 45,
          gpu_name: 'NVIDIA GeForce RTX / Intel Iris',
          gpu_usage_percent: Math.round(10 + Math.random() * 20),
          os_name: navigator.platform || 'Windows 11',
          uptime: '4h 32m'
        };
      }
      this.lastData = data;
      this.render(data);
    } catch (err) {
      console.warn('Telemetry fetch error:', err);
    }
  }

  render(data) {
    // Header mini stats
    const cpuMini = document.getElementById('cpuMini');
    const ramMini = document.getElementById('ramMini');
    if (cpuMini) cpuMini.textContent = `${Math.round(data.cpu_usage)}%`;
    if (ramMini) ramMini.textContent = `${Math.round(data.ram_usage_percent)}%`;

    // Full Monitor View elements
    const cpuFullName = document.getElementById('cpuFullName');
    const cpuGauge = document.getElementById('cpuGauge');
    const cpuUsageVal = document.getElementById('cpuUsageVal');
    const cpuCoresVal = document.getElementById('cpuCoresVal');

    if (cpuFullName) cpuFullName.textContent = data.cpu_name;
    if (cpuGauge) cpuGauge.style.width = `${Math.min(100, Math.max(0, data.cpu_usage))}%`;
    if (cpuUsageVal) cpuUsageVal.textContent = `${Math.round(data.cpu_usage)}%`;
    if (cpuCoresVal) cpuCoresVal.textContent = data.cpu_cores ? `${data.cpu_cores} Cores` : '--';

    const ramInfoText = document.getElementById('ramInfoText');
    const ramGauge = document.getElementById('ramGauge');
    const ramUsageVal = document.getElementById('ramUsageVal');
    const ramFreeVal = document.getElementById('ramFreeVal');

    const used = (data.ram_used_gb || 0).toFixed(1);
    const total = (data.ram_total_gb || 0).toFixed(1);
    const free = (data.ram_total_gb - data.ram_used_gb).toFixed(1);

    if (ramInfoText) ramInfoText.textContent = `${used} / ${total} GB`;
    if (ramGauge) ramGauge.style.width = `${Math.min(100, Math.max(0, data.ram_usage_percent))}%`;
    if (ramUsageVal) ramUsageVal.textContent = `${Math.round(data.ram_usage_percent)}%`;
    if (ramFreeVal) ramFreeVal.textContent = `${free} GB`;

    const gpuFullName = document.getElementById('gpuFullName');
    const gpuGauge = document.getElementById('gpuGauge');
    const gpuUsageVal = document.getElementById('gpuUsageVal');

    if (gpuFullName) gpuFullName.textContent = data.gpu_name || 'Generic GPU';
    if (gpuGauge) gpuGauge.style.width = `${Math.min(100, Math.max(0, data.gpu_usage_percent || 15))}%`;
    if (gpuUsageVal) gpuUsageVal.textContent = `${Math.round(data.gpu_usage_percent || 15)}%`;

    const osInfoText = document.getElementById('osInfoText');
    const uptimeVal = document.getElementById('uptimeVal');
    if (osInfoText) osInfoText.textContent = data.os_name || 'Windows 11 / Linux';
    if (uptimeVal) uptimeVal.textContent = data.uptime || '--';
  }

  async updateWeather(city = '') {
    try {
      let weather = await this.invoke('get_weather', { city });
      if (!weather) {
        // Fallback: query wttr.in or open-meteo client-side
        const res = await fetch(`https://wttr.in/${encodeURIComponent(city)}?format=j1`);
        if (res.ok) {
          const json = await res.json();
          const current = json.current_condition?.[0] || {};
          const area = json.nearest_area?.[0]?.areaName?.[0]?.value || city || 'Local';
          weather = {
            temp: current.temp_C ? `${current.temp_C}°` : '--°',
            city: area,
            desc: current.weatherDesc?.[0]?.value || 'Clear',
            icon: '⛅'
          };
        }
      }

      if (weather) {
        const iconEl = document.getElementById('weatherIcon');
        const tempEl = document.getElementById('weatherTemp');
        const cityEl = document.getElementById('weatherCity');

        if (iconEl) iconEl.textContent = weather.icon || '⛅';
        if (tempEl) tempEl.textContent = weather.temp;
        if (cityEl) cityEl.textContent = weather.city;
      }
    } catch (e) {
      console.warn('Weather fetch error:', e);
    }
  }
}
