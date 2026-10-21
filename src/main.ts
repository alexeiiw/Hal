import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import "./style.css";

const bars = Array.from(document.querySelectorAll<HTMLElement>(".equalizer i"));
const metricOutput = document.querySelector<HTMLOutputElement>("#metric");
const root = document.querySelector<HTMLElement>("#hal");
const wavePath = document.querySelector<SVGPathElement>("#wave-path");
const waveTail = document.querySelector<SVGPathElement>("#wave-tail");
const weatherIcon = document.querySelector<SVGSVGElement>("#weather-icon");
const clockHour = document.querySelector<SVGPathElement>("#clock-hour");
const clockMinute = document.querySelector<SVGPathElement>("#clock-minute");
const gaugeArc = document.querySelector<SVGCircleElement>("#gauge-arc");
const gaugeValue = document.querySelector<HTMLElement>("#gauge-value");
const photoImg = document.querySelector<HTMLImageElement>("#photo-img");
const netLine = document.querySelector<SVGPathElement>("#net-line");
const netUpLine = document.querySelector<SVGPathElement>("#net-up-line");
const netArea = document.querySelector<SVGPathElement>("#net-area");
const netDownEl = document.querySelector<HTMLElement>("#net-down");
const netUpEl = document.querySelector<HTMLElement>("#net-up-num");
const copArc = document.querySelector<SVGCircleElement>("#cop-arc");
const copUsed = document.querySelector<HTMLElement>("#cop-used");
const copTotal = document.querySelector<HTMLElement>("#cop-total");
const dsArc = document.querySelector<SVGCircleElement>("#ds-arc");
const dsBalance = document.querySelector<HTMLElement>("#ds-balance");
type MetricView = "cpu" | "memory" | "weather" | "clock" | "bitcoin" | "face" | "task" | "photo" | "red" | "copilot" | "deepseek";
type WeatherKind = "clear" | "cloud" | "fog" | "rain" | "snow" | "storm";
type BitcoinQuote = { price: number; previousPrice: number | null; updatedAt: number };
type NetworkSample = { down: number; up: number };
type CopilotStatus = {
  used: number;
  total: number;
  remaining: number;
  hasQuota: boolean;
  resetAt: string | null;
  fetchedAt: number;
  ok: boolean;
  error: string | null;
};
type DeepSeekStatus = {
  total: number;
  granted: number;
  toppedUp: number;
  currency: string;
  available: boolean;
  alertBelow: number;
  fetchedAt: number;
  ok: boolean;
  error: string | null;
};
type HalMode = "normal" | "developer" | "focus" | "relax" | "happy";
type ViewChange = { view: MetricView; enabled: boolean };

const bitcoinCacheKey = "hal-bitcoin-usd";
const bitcoinRefreshMs = 6 * 60 * 60 * 1000;
const bitcoinCacheMaxAgeMs = 24 * 60 * 60 * 1000;
const bitcoinBuyThreshold = 76_000;
const halSettingsKey = "hal-settings";
const halClockResetKey = "hal-clock-once";
const allViews: MetricView[] = ["cpu", "memory", "red", "weather", "clock", "bitcoin", "face", "task", "photo", "copilot", "deepseek"];

if (!metricOutput || !root || !wavePath || !waveTail || !weatherIcon || !clockHour || !clockMinute || !gaugeArc || !gaugeValue || !photoImg || !netLine || !netUpLine || !netArea || !netDownEl || !netUpEl || !copArc || !copUsed || !copTotal || !dsArc || !dsBalance || bars.length !== 5) {
  throw new Error("La interfaz HAL no se pudo inicializar.");
}

let cpu = 0;
let memory = 0;
let weather: { temperature: number; code: number } | null = null;
let bitcoin: BitcoinQuote | null = null;
let opencodeActive = false;
let opencodeCpu = 0;
let wavePhase = 0;
let metricView: MetricView = "cpu";
let clockDisplay = "HAL --:--";
let lastClockMinute = "";
let halMode: HalMode = "normal";
let phrase: string | null = null;
let task: string | null = null;
let notice: string | null = null;
let noticeTimeout = 0;
let netDownRate = 0;
let netUpRate = 0;
let copilot: CopilotStatus | null = null;
let deepseek: DeepSeekStatus | null = null;
let enabledViews: Record<MetricView, boolean> = {
  cpu: true,
  memory: true,
  red: true,
  weather: true,
  clock: true,
  bitcoin: true,
  face: true,
  task: true,
  photo: true,
  copilot: true,
  deepseek: true,
};

function saveHalSettings(): void {
  localStorage.setItem(halSettingsKey, JSON.stringify({ mode: halMode, phrase, task, enabledViews }));
  const disabledViews = allViews.filter((view) => !enabledViews[view]);
  void invoke("sync_hal_state", { mode: halMode, task, disabledViews }).catch(() => undefined);
}

function loadHalSettings(): void {
  try {
    const saved = JSON.parse(localStorage.getItem(halSettingsKey) ?? "{}") as {
      mode?: HalMode;
      phrase?: string | null;
      task?: string | null;
      enabledViews?: Partial<Record<MetricView, boolean>>;
    };

    if (["normal", "developer", "focus", "relax", "happy"].includes(saved.mode ?? "")) {
      halMode = saved.mode!;
    }
    phrase = typeof saved.phrase === "string" ? saved.phrase : null;
    task = typeof saved.task === "string" ? saved.task : null;
    enabledViews = { ...enabledViews, ...saved.enabledViews };
  } catch {
    localStorage.removeItem(halSettingsKey);
  }

  root!.dataset.mode = halMode;
  if (!localStorage.getItem(halClockResetKey)) {
    enabledViews.clock = true;
    localStorage.setItem(halClockResetKey, "1");
  }
  saveHalSettings();
}

function activeViews(): MetricView[] {
  const active = allViews.filter((view) => enabledViews[view] && (view !== "task" || task !== null));
  return active.length > 0 ? active : ["face"];
}

function weatherKind(code: number): WeatherKind {
  if (code === 0) return "clear";
  if (code <= 3) return "cloud";
  if (code <= 48) return "fog";
  if (code <= 67 || (code >= 80 && code <= 82)) return "rain";
  if (code <= 77 || code === 85 || code === 86) return "snow";
  return "storm";
}

function weatherLabel(code: number): string {
  const labels: Record<WeatherKind, string> = {
    clear: "DESPEJADO",
    cloud: "NUBES",
    fog: "NIEBLA",
    rain: "LLUVIA",
    snow: "NIEVE",
    storm: "TORMENTA",
  };
  return labels[weatherKind(code)];
}

function weatherIconMarkup(code: number): string {
  const sun = '<circle cx="25" cy="25" r="10"/><path d="M25 5v7M25 38v7M5 25h7M38 25h7M11 11l5 5M34 34l5 5M39 11l-5 5M16 34l-5 5"/>';
  const cloud = '<path d="M18 46h29a10 10 0 0 0 0-20 15 15 0 0 0-28-3A12 12 0 0 0 18 46Z"/>';
  const kind = weatherKind(code);

  if (kind === "clear") return sun;
  if (kind === "cloud") return `${sun}${cloud}`;
  if (kind === "fog") return `${cloud}<path d="M15 52h34M20 58h24"/>`;
  if (kind === "rain") return `${cloud}<path d="m23 51-3 7m12-7-3 7m12-7-3 7"/>`;
  if (kind === "snow") return `${cloud}<path d="m24 53 0 8m-4-4h8m11-4v8m-4-4h8"/>`;
  return `${cloud}<path d="m34 50-5 9h6l-3 7 8-11h-6l4-5Z"/>`;
}

function setWeatherPalette(code: number): void {
  const palettes: Record<WeatherKind, [string, string]> = {
    clear: ["#22c55e", "rgba(34, 197, 94, 0.75)"],
    cloud: ["#38bdf8", "rgba(56, 189, 248, 0.78)"],
    fog: ["#7dd3fc", "rgba(125, 211, 252, 0.72)"],
    rain: ["#3b82f6", "rgba(59, 130, 246, 0.8)"],
    snow: ["#e0f2fe", "rgba(224, 242, 254, 0.82)"],
    storm: ["#a78bfa", "rgba(167, 139, 250, 0.82)"],
  };
  const [color, glow] = palettes[weatherKind(code)];
  root!.style.setProperty("--weather-color", color);
  root!.style.setProperty("--weather-glow", glow);
}

function updateClock(): void {
  const now = new Date();
  const hours = now.getHours();
  const minutes = now.getMinutes();
  const minuteKey = `${hours}:${minutes}`;

  if (minuteKey === lastClockMinute) return;

  lastClockMinute = minuteKey;
  clockDisplay = `HAL ${hours.toString().padStart(2, "0")}:${minutes.toString().padStart(2, "0")}`;
  clockHour!.setAttribute("transform", `rotate(${(hours % 12) * 30 + minutes * 0.5} 32 32)`);
  clockMinute!.setAttribute("transform", `rotate(${minutes * 6} 32 32)`);
}

function bitcoinQuote(): BitcoinQuote | null {
  return bitcoin && Date.now() - bitcoin.updatedAt <= bitcoinCacheMaxAgeMs ? bitcoin : null;
}

function bitcoinDirection(quote: BitcoinQuote): "up" | "down" | "flat" {
  if (quote.previousPrice === null || quote.price === quote.previousPrice) return "flat";
  return quote.price > quote.previousPrice ? "up" : "down";
}

function formatBitcoinPrice(price: number): string {
  return `$${(price / 1_000).toFixed(1)}K`;
}

const gaugeCircumference = 2 * Math.PI * 42;

function updateGauge(): void {
  const ratio = Math.max(0, Math.min(1, memory / 100));
  gaugeArc!.style.strokeDashoffset = `${gaugeCircumference * (1 - ratio)}`;
  gaugeValue!.textContent = memory.toFixed(0);
}

const copCircumference = 2 * Math.PI * 42;

function updateCopilot(): void {
  const ok = copilot?.ok === true;
  const used = ok ? Math.round(copilot!.used) : 0;
  const total = ok ? Math.round(copilot!.total) : 0;
  const ratio = total > 0 ? Math.min(1, Math.max(0, used / total)) : 0;
  copArc!.style.strokeDashoffset = `${copCircumference * (1 - ratio)}`;
  copUsed!.textContent = ok ? String(used) : "--";
  copTotal!.textContent = total > 0 ? String(total) : "--";
}

const dsCircumference = 2 * Math.PI * 42;

function formatMoney(value: number, currency: string): string {
  const symbol = currency === "CNY" ? "¥" : "$";
  return `${symbol}${value.toFixed(2)}`;
}

function updateDeepSeek(): void {
  const ok = deepseek?.ok === true;
  const alert = ok ? Math.max(0.01, deepseek!.alertBelow) : 1;
  const balance = ok ? Math.max(0, deepseek!.total) : 0;
  const ratio = Math.min(1, balance / (alert * 2));
  dsArc!.style.strokeDashoffset = `${dsCircumference * (1 - ratio)}`;
  dsBalance!.textContent = ok ? formatMoney(balance, deepseek!.currency) : "--";
}

const netHistory: NetworkSample[] = [];
const netHistorySize = 58;
const netWidth = 116;
const netHeight = 70;

function formatRate(bytes: number): string {
  const mb = bytes / 1_048_576;
  if (mb >= 10) return `${mb.toFixed(0)}M`;
  if (mb >= 1) return `${mb.toFixed(1)}M`;
  return `${(bytes / 1024).toFixed(0)}K`;
}

function networkPath(values: number[], maxRate: number): string {
  const step = values.length > 1 ? netWidth / (values.length - 1) : netWidth;
  return values
    .map((value, index) => {
      const y = netHeight - 2 - Math.min(1, value / maxRate) * (netHeight - 6);
      return `${index === 0 ? "M" : "L"}${(index * step).toFixed(2)} ${y.toFixed(2)}`;
    })
    .join(" ");
}

function updateNetwork(): void {
  netDownEl!.textContent = formatRate(netDownRate);
  netUpEl!.textContent = formatRate(netUpRate);

  const maxRate = Math.max(1, ...netHistory.map((sample) => sample.down));
  const downPath = networkPath(netHistory.map((sample) => sample.down), maxRate);
  const upPath = networkPath(netHistory.map((sample) => sample.up), maxRate);

  netLine!.setAttribute("d", downPath);
  netUpLine!.setAttribute("d", upPath);
  netArea!.setAttribute("d", `${downPath} L${netWidth} ${netHeight} L0 ${netHeight} Z`);
}

const photoPickKey = "hal-photo-pick";
const photoRefreshMs = 6 * 60 * 60 * 1000;
const defaultPhoto =
  "data:image/svg+xml," +
  encodeURIComponent(
    '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 200 150">' +
      '<rect width="200" height="150" rx="10" fill="#0b120f"/>' +
      '<g fill="none" stroke="#22c55e" stroke-width="4" stroke-linecap="round" stroke-linejoin="round">' +
      '<rect x="62" y="52" width="76" height="54" rx="10"/>' +
      '<circle cx="100" cy="79" r="15"/>' +
      '<path d="M80 52l7-9h26l7 9"/>' +
      "</g>" +
      '<text x="100" y="128" fill="#6b8f80" font-family="Consolas, monospace" font-size="13" text-anchor="middle" letter-spacing="4">HAL FOTOS</text>' +
      "</svg>",
  );

type PhotoPick = { name: string; pickedAt: number };

function setDefaultPhoto(): void {
  photoImg!.src = defaultPhoto;
}

async function showPhoto(name: string): Promise<void> {
  try {
    const dataUrl = await invoke<string>("read_photo", { name });
    photoImg!.src = dataUrl;
  } catch {
    setDefaultPhoto();
  }
}

async function refreshPhoto(): Promise<void> {
  let names: string[] = [];
  try {
    names = await invoke<string[]>("list_photos");
  } catch {
    names = [];
  }

  if (names.length === 0) {
    setDefaultPhoto();
    return;
  }

  let pick: PhotoPick | null = null;
  try {
    pick = JSON.parse(localStorage.getItem(photoPickKey) ?? "null") as PhotoPick | null;
  } catch {
    pick = null;
  }

  const validPick =
    pick !== null &&
    typeof pick.name === "string" &&
    typeof pick.pickedAt === "number" &&
    Date.now() - pick.pickedAt < photoRefreshMs &&
    names.includes(pick.name);

  if (!validPick) {
    const name = names[Math.floor(Math.random() * names.length)];
    pick = { name, pickedAt: Date.now() };
    localStorage.setItem(photoPickKey, JSON.stringify(pick));
  }

  await showPhoto(pick!.name);
}

function currentMetric(): {
  display: string;
  value: number;
  busyAt: number;
  criticalAt: number;
} {
  if (notice) {
    return { display: `AVISO ${notice}`, value: 0, busyAt: 101, criticalAt: 102 };
  }

  if (metricView === "cpu") {
    return { display: `CPU ${cpu.toFixed(1)}%`, value: cpu, busyAt: 40, criticalAt: 70 };
  }

  if (metricView === "memory") {
    return { display: `RAM ${memory.toFixed(1)}%`, value: memory, busyAt: 60, criticalAt: 80 };
  }

  if (metricView === "clock") {
    return { display: clockDisplay, value: 0, busyAt: 101, criticalAt: 102 };
  }

  if (metricView === "bitcoin") {
    const quote = bitcoinQuote();

    if (!quote) return { display: "BTC NO DISP.", value: 0, busyAt: 101, criticalAt: 102 };

    const arrow = { up: "↑", down: "↓", flat: "→" }[bitcoinDirection(quote)];
    const alert = quote.price <= bitcoinBuyThreshold;
    return {
      display: `BTC ${formatBitcoinPrice(quote.price)} ${arrow}`,
      value: alert ? 100 : 0,
      busyAt: 101,
      criticalAt: 90,
    };
  }

  if (metricView === "face") {
    return { display: phrase ?? "HAL EN LINEA", value: 0, busyAt: 101, criticalAt: 102 };
  }

  if (metricView === "task") {
    return { display: task ?? "SIN TAREA", value: 0, busyAt: 101, criticalAt: 102 };
  }

  if (metricView === "photo") {
    return { display: "HAL FOTOS", value: 0, busyAt: 101, criticalAt: 102 };
  }

  if (metricView === "red") {
    return {
      display: `RED ↓${formatRate(netDownRate)} ↑${formatRate(netUpRate)}`,
      value: 0,
      busyAt: 101,
      criticalAt: 102,
    };
  }

  if (metricView === "copilot") {
    if (!copilot || !copilot.ok) {
      return { display: copilot?.error ?? "COPILOT NO DISP.", value: 0, busyAt: 101, criticalAt: 102 };
    }

    const used = Math.round(copilot.used);
    const total = Math.round(copilot.total);
    const ratio = total > 0 ? Math.min(100, (used / total) * 100) : 0;
    return { display: `COPILOT ${used}/${total}`, value: ratio, busyAt: 70, criticalAt: 90 };
  }

  if (metricView === "deepseek") {
    if (!deepseek || !deepseek.ok) {
      return { display: deepseek?.error ?? "DEEPSEEK NO DISP.", value: 0, busyAt: 101, criticalAt: 102 };
    }

    return {
      display: `DEEPSEEK ${formatMoney(deepseek.total, deepseek.currency)}`,
      value: 0,
      busyAt: 101,
      criticalAt: 102,
    };
  }

  return weather
    ? {
        display: `${weather.temperature.toFixed(0)} C ${weatherLabel(weather.code)}`,
        value: 0,
        busyAt: 101,
        criticalAt: 102,
      }
    : { display: "CLIMA NO DISP.", value: 0, busyAt: 101, criticalAt: 102 };
}

function animateBars(): void {
  if (metricView === "clock") updateClock();

  const metric = currentMetric();
  const overloaded = metric.value >= metric.criticalAt;
  const busy = metric.value >= metric.busyAt && !overloaded;
  const activity = overloaded
    ? 35 + metric.value * 0.65
    : busy
      ? 26 + metric.value * 0.55
      : 18 + metric.value * 0.5;
  const globalTempo = overloaded ? 110 : busy ? 220 : 460;
  const waveTempo = Math.max(60, 440 / (1 + opencodeCpu * 0.15));
  const showOpenCodeWave = metricView === "cpu" && opencodeActive;
  const staticView = notice !== null || metricView === "clock" || metricView === "bitcoin" || metricView === "face" || metricView === "task" || metricView === "memory" || metricView === "photo" || metricView === "red" || metricView === "copilot" || metricView === "deepseek" || (metricView === "weather" && weather !== null);
  const tempo = staticView ? 1000 : showOpenCodeWave ? waveTempo : globalTempo;

  root!.classList.toggle("overloaded", overloaded);
  root!.classList.toggle("busy", busy);
  root!.classList.toggle("opencode-active", showOpenCodeWave);
  root!.classList.toggle("memory-view", metricView === "memory");
  root!.classList.toggle("weather-view", metricView === "weather" && weather !== null);
  root!.classList.toggle("clock-view", metricView === "clock");
  root!.classList.toggle("btc-view", metricView === "bitcoin");
  root!.classList.toggle("face-view", metricView === "face");
  root!.classList.toggle("task-view", metricView === "task");
  root!.classList.toggle("photo-view", metricView === "photo");
  root!.classList.toggle("red-view", metricView === "red");
  root!.classList.toggle("copilot-view", metricView === "copilot");
  root!.classList.toggle("cop-error", metricView === "copilot" && (!copilot || !copilot.ok));
  root!.classList.toggle("cop-empty", metricView === "copilot" && !!copilot && copilot.ok && (!copilot.hasQuota || copilot.used >= copilot.total));
  root!.classList.toggle("deepseek-view", metricView === "deepseek");
  root!.classList.toggle("ds-error", metricView === "deepseek" && (!deepseek || !deepseek.ok));
  root!.classList.toggle("ds-low", metricView === "deepseek" && !!deepseek && deepseek.ok && deepseek.total > deepseek.alertBelow && deepseek.total <= deepseek.alertBelow * 2);
  root!.classList.toggle("ds-empty", metricView === "deepseek" && !!deepseek && deepseek.ok && (!deepseek.available || deepseek.total <= deepseek.alertBelow));
  root!.classList.toggle("notice-active", notice !== null);
  const quote = bitcoinQuote();
  root!.classList.toggle("btc-alert", metricView === "bitcoin" && !!quote && quote.price <= bitcoinBuyThreshold);
  root!.classList.toggle("btc-up", metricView === "bitcoin" && !!quote && bitcoinDirection(quote) === "up");
  root!.classList.toggle("btc-down", metricView === "bitcoin" && !!quote && bitcoinDirection(quote) === "down");
  updateGauge();
  updateCopilot();
  updateDeepSeek();
  root!.style.setProperty("--tempo", `${tempo}ms`);
  metricOutput!.value = metric.display;
  metricOutput!.textContent = metricOutput!.value;

  if (showOpenCodeWave) {
    const amplitude = 18 + Math.min(opencodeCpu, 20) * 0.45;
    const points: string[] = [];

    for (let x = 0; x <= 116; x += 2) {
      const y = 35 + Math.sin((x / 116) * Math.PI * 2 + wavePhase) * amplitude;
      points.push(`${x === 0 ? "M" : "L"}${x} ${y.toFixed(2)}`);
    }

    const nextPath = points.join(" ");
    waveTail!.setAttribute("d", wavePath!.getAttribute("d") ?? nextPath);
    wavePath!.setAttribute("d", nextPath);
    wavePhase += 0.45;
  } else if (!staticView) {
    wavePhase = 0;

    for (const bar of bars) {
      const height = 12 + Math.random() * activity;
      bar.style.height = `${Math.min(height, 100)}%`;
    }
  }

  window.setTimeout(animateBars, tempo);
}

void listen<number>("evento-cpu", (event) => {
  cpu = Math.max(0, Math.min(100, event.payload));
});

void listen<number>("evento-memoria", (event) => {
  memory = Math.max(0, Math.min(100, event.payload));
});

void listen<boolean>("evento-opencode", (event) => {
  opencodeActive = event.payload;
});

void listen<number>("evento-opencode-cpu", (event) => {
  opencodeCpu = Math.max(0, event.payload);
});

void listen<NetworkSample>("evento-red", (event) => {
  netDownRate = Math.max(0, event.payload.down);
  netUpRate = Math.max(0, event.payload.up);
  netHistory.push({ down: netDownRate, up: netUpRate });
  if (netHistory.length > netHistorySize) netHistory.shift();
  updateNetwork();
});

void listen<CopilotStatus>("evento-copilot", (event) => {
  copilot = event.payload;
});

void listen<DeepSeekStatus>("evento-deepseek", (event) => {
  deepseek = event.payload;
});

void listen<HalMode>("evento-modo", (event) => {
  if (["normal", "developer", "focus", "relax", "happy"].includes(event.payload)) {
    halMode = event.payload;
    root!.dataset.mode = halMode;
    saveHalSettings();
  }
});

void listen<ViewChange>("evento-vista", (event) => {
  if (!allViews.includes(event.payload.view)) return;

  const nextViews = { ...enabledViews, [event.payload.view]: event.payload.enabled };
  if (!Object.values(nextViews).some(Boolean)) return;

  enabledViews = nextViews;
  if (!enabledViews[metricView]) metricView = activeViews()[0];
  saveHalSettings();
});

void listen<string | null>("evento-frase", (event) => {
  phrase = event.payload;
  saveHalSettings();
});

void listen<string | null>("evento-tarea", (event) => {
  task = event.payload;
  if (metricView === "task" && !task) metricView = activeViews()[0];
  saveHalSettings();
});

void listen<string>("evento-aviso", (event) => {
  notice = event.payload;
  window.clearTimeout(noticeTimeout);
  noticeTimeout = window.setTimeout(() => {
    notice = null;
  }, 30_000);
});

async function loadWeather(position: GeolocationPosition): Promise<void> {
  const { latitude, longitude } = position.coords;
  const response = await fetch(
    `https://api.open-meteo.com/v1/forecast?latitude=${latitude}&longitude=${longitude}&current=temperature_2m,weather_code`,
  );
  const data = (await response.json()) as {
    current?: { temperature_2m: number; weather_code: number };
  };

  if (data.current) {
    weather = { temperature: data.current.temperature_2m, code: data.current.weather_code };
    weatherIcon!.innerHTML = weatherIconMarkup(weather.code);
    setWeatherPalette(weather.code);
  }
}

function refreshWeather(): void {
  navigator.geolocation?.getCurrentPosition(
    (position) => void loadWeather(position).catch(() => undefined),
    () => undefined,
    { maximumAge: 30 * 60 * 1000, timeout: 10_000 },
  );
}

function loadBitcoinCache(): void {
  try {
    const cached = JSON.parse(localStorage.getItem(bitcoinCacheKey) ?? "null") as BitcoinQuote | null;

    if (cached && typeof cached.price === "number" && typeof cached.updatedAt === "number") {
      bitcoin = { price: cached.price, previousPrice: cached.previousPrice ?? null, updatedAt: cached.updatedAt };
    }
  } catch {
    localStorage.removeItem(bitcoinCacheKey);
  }
}

async function refreshBitcoin(): Promise<void> {
  const response = await fetch(
    "https://api.coingecko.com/api/v3/simple/price?ids=bitcoin&vs_currencies=usd",
  );

  if (!response.ok) throw new Error(`CoinGecko respondio ${response.status}`);

  const data = (await response.json()) as { bitcoin?: { usd?: number } };
  const price = data.bitcoin?.usd;

  if (typeof price !== "number") throw new Error("CoinGecko no devolvio BTC/USD");

  bitcoin = { price, previousPrice: bitcoin?.price ?? null, updatedAt: Date.now() };
  localStorage.setItem(bitcoinCacheKey, JSON.stringify(bitcoin));
}

window.setInterval(() => {
  const views = activeViews();
  metricView = views[(views.indexOf(metricView) + 1) % views.length];
}, 4000);

loadHalSettings();
refreshWeather();
window.setInterval(refreshWeather, 30 * 60 * 1000);
loadBitcoinCache();
void refreshBitcoin().catch(() => undefined);
window.setInterval(() => void refreshBitcoin().catch(() => undefined), bitcoinRefreshMs);
void refreshPhoto();
window.setInterval(() => void refreshPhoto(), 60 * 60 * 1000);

animateBars();
