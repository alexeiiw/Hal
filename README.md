# HAL 0.4.1

HAL es un indicador de escritorio minimalista para Windows. Muestra el uso global de CPU como un ecualizador flotante, transparente y click-through.

## Comportamiento

- Se ejecuta manualmente cuando se necesita; no se inicia con Windows.
- Permanece sobre otras ventanas, pero los clics atraviesan HAL hacia la aplicacion de debajo.
- Se coloca abajo a la derecha del area util del monitor.
- Muestra CPU global cada segundo con un decimal.
- Detecta la ejecucion de `OpenCode.exe` y sustituye el ecualizador por una onda sinusoidal SVG sin cambiar la metrica global.

| CPU global | Estado | Color | Ritmo |
| --- | --- | --- | --- |
| Menor de 40% | Reposo | Verde | Lento |
| Desde 40% hasta menos de 70% | Trabajo | Azul | Medio |
| 70% o mas | Carga alta | Rojo | Rapido |

Las alturas de las cinco barras usan `Math.random()` acotado por la CPU medida. No hay IA ni llamadas de red en el ejecutable.

## Modo OpenCode

Cuando `OpenCode.exe` esta en ejecucion, HAL oculta el ecualizador y muestra una senal sinusoidal SVG continua. Los colores siguen reflejando la CPU global.

| CPU agregada de OpenCode | Frecuencia de la onda |
| --- | --- |
| Menor de 2% | Lenta |
| Desde 2% hasta menos de 10% | Media |
| 10% o mas | Rapida |

La CPU de OpenCode se calcula sumando sus procesos locales. HAL actualiza la lista de procesos cada segundo, por lo que al cerrar OpenCode vuelve automaticamente al ecualizador global. No inspecciona conversaciones, archivos ni argumentos de los procesos.

## Ejecutar

Para usar el binario ya compilado:

```cmd
src-tauri\target\release\hal.exe
```

Para cerrarlo desde CMD:

```cmd
taskkill /IM hal.exe /F
```

## Instaladores

- NSIS: `src-tauri\target\release\bundle\nsis\HAL_0.4.1_x64-setup.exe`
- MSI: `src-tauri\target\release\bundle\msi\HAL_0.4.1_x64_en-US.msi`

## Desarrollo

Requisitos: Node.js LTS, Rust estable con destino `x86_64-pc-windows-msvc` y Visual Studio Build Tools con C++.

```cmd
npm install
npm run tauri dev
```

Para crear los instaladores:

```cmd
npm run tauri build
```

## Arquitectura

- `src-tauri/src/main.rs`: mide la CPU global, refresca los procesos, detecta `OpenCode.exe`, suma su CPU y emite los estados una vez por segundo.
- `src/main.ts`: escucha los eventos de CPU y OpenCode para cambiar entre el ecualizador y una senal SVG animada.
- `src/style.css`: define el aspecto y los estados visuales.

HAL mide la CPU global deliberadamente. La deteccion de OpenCode solo modifica el patron y frecuencia de animacion; no registra actividad, no analiza contenido y no cambia la metrica mostrada.
