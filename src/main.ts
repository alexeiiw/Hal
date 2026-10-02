import { listen } from "@tauri-apps/api/event";
import "./style.css";

const bars = Array.from(document.querySelectorAll<HTMLElement>(".equalizer i"));
const cpuOutput = document.querySelector<HTMLOutputElement>("#cpu");
const root = document.querySelector<HTMLElement>("#hal");
const wavePath = document.querySelector<SVGPathElement>("#wave-path");

if (!cpuOutput || !root || !wavePath || bars.length !== 5) {
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

  if (opencodeActive) {
    const amplitude = 18 + Math.min(opencodeCpu, 20) * 0.45;
    const points: string[] = [];

    for (let x = 0; x <= 116; x += 2) {
      const y = 35 + Math.sin((x / 116) * Math.PI * 2 + wavePhase) * amplitude;
      points.push(`${x === 0 ? "M" : "L"}${x} ${y.toFixed(2)}`);
    }

    wavePath!.setAttribute("d", points.join(" "));
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
