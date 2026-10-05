import { listen } from "@tauri-apps/api/event";
import "./style.css";

const bars = Array.from(document.querySelectorAll<HTMLElement>(".equalizer i"));
const metricOutput = document.querySelector<HTMLOutputElement>("#metric");
const root = document.querySelector<HTMLElement>("#hal");
const wavePath = document.querySelector<SVGPathElement>("#wave-path");
const waveTail = document.querySelector<SVGPathElement>("#wave-tail");

if (!metricOutput || !root || !wavePath || !waveTail || bars.length !== 5) {
  throw new Error("La interfaz HAL no se pudo inicializar.");
}

let cpu = 0;
let memory = 0;
let opencodeActive = false;
let opencodeCpu = 0;
let wavePhase = 0;
let metricView: "cpu" | "memory" = "cpu";

function currentMetric(): { label: string; value: number; busyAt: number; criticalAt: number } {
  return metricView === "cpu"
    ? { label: "CPU", value: cpu, busyAt: 40, criticalAt: 70 }
    : { label: "RAM", value: memory, busyAt: 60, criticalAt: 80 };
}

function animateBars(): void {
  const metric = currentMetric();
  const overloaded = metric.value >= metric.criticalAt;
  const busy = metric.value >= metric.busyAt && !overloaded;
  const activity = overloaded
    ? 35 + metric.value * 0.65
    : busy
      ? 26 + metric.value * 0.55
      : 18 + metric.value * 0.5;
  const globalTempo = overloaded ? 110 : busy ? 220 : 460;
  const waveTempo = opencodeCpu >= 10 ? 80 : opencodeCpu >= 2 ? 170 : 440;
  const showOpenCodeWave = metricView === "cpu" && opencodeActive;
  const tempo = showOpenCodeWave ? waveTempo : globalTempo;

  root!.classList.toggle("overloaded", overloaded);
  root!.classList.toggle("busy", busy);
  root!.classList.toggle("opencode-active", showOpenCodeWave);
  root!.classList.toggle("memory-view", metricView === "memory");
  root!.style.setProperty("--tempo", `${tempo}ms`);
  metricOutput!.value = `${metric.label} ${metric.value.toFixed(1)}%`;
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
  } else {
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

window.setInterval(() => {
  metricView = metricView === "cpu" ? "memory" : "cpu";
}, 4000);

animateBars();
