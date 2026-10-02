import { listen } from "@tauri-apps/api/event";
import "./style.css";

const bars = Array.from(document.querySelectorAll<HTMLElement>(".equalizer i"));
const cpuOutput = document.querySelector<HTMLOutputElement>("#cpu");
const root = document.querySelector<HTMLElement>("#hal");

if (!cpuOutput || !root || bars.length !== 5) {
  throw new Error("La interfaz HAL no se pudo inicializar.");
}

let cpu = 0;
let opencodeActive = false;
let opencodeCpu = 0;
let wavePhase = 0;

function animateBars(): void {
  const overloaded = cpu >= 70;
  const busy = cpu >= 40 && !overloaded;
  const activity = overloaded ? 35 + cpu * 0.65 : busy ? 26 + cpu * 0.55 : 18 + cpu * 0.5;
  const globalTempo = overloaded ? 110 : busy ? 220 : 460;
  const waveTempo = opencodeCpu >= 10 ? 80 : opencodeCpu >= 2 ? 170 : 440;
  const tempo = opencodeActive ? waveTempo : globalTempo;

  root!.classList.toggle("overloaded", overloaded);
  root!.classList.toggle("busy", busy);
  root!.classList.toggle("opencode-active", opencodeActive);
  root!.style.setProperty("--tempo", `${tempo}ms`);

  for (const [index, bar] of bars.entries()) {
    const height = opencodeActive
      ? 52 + Math.sin(wavePhase + index * 1.15) * (28 + Math.min(opencodeCpu, 20) * 0.5)
      : 12 + Math.random() * activity;
    bar.style.height = `${Math.min(height, 100)}%`;
  }

  wavePhase += opencodeActive ? 0.72 : 0;
  window.setTimeout(animateBars, tempo);
}

void listen<number>("evento-cpu", (event) => {
  cpu = Math.max(0, Math.min(100, event.payload));
  cpuOutput!.value = `${cpu.toFixed(1)}%`;
  cpuOutput!.textContent = cpuOutput!.value;
});

void listen<boolean>("evento-opencode", (event) => {
  opencodeActive = event.payload;
});

void listen<number>("evento-opencode-cpu", (event) => {
  opencodeCpu = Math.max(0, event.payload);
});

animateBars();
