# HAL 0.5.0

HAL es un indicador de escritorio minimalista para Windows. Muestra el uso global de CPU como un ecualizador flotante, transparente y click-through.

## Comportamiento

- Se ejecuta manualmente cuando se necesita; no se inicia con Windows.
- Permanece sobre otras ventanas, pero los clics atraviesan HAL hacia la aplicacion de debajo.
- Se coloca abajo a la derecha del area util del monitor.
- Alterna CPU global y RAM principal cada cuatro segundos, con un decimal.
- Detecta la ejecucion de `OpenCode.exe` y sustituye el ecualizador por una onda sinusoidal SVG sin cambiar la metrica global.

| Vista | Umbral medio | Umbral alto | Colores |
| --- | --- | --- | --- |
| CPU | 40% | 70% | Verde, azul, rojo |
| RAM | 60% | 80% | Verde, azul, rojo |

Cada vista conserva su metrica durante cuatro segundos. Las barras usan `Math.random()` acotado por la metrica visible. No hay IA ni llamadas de red en el ejecutable.

## Modo OpenCode

Durante la vista de CPU, si `OpenCode.exe` esta en ejecucion, HAL oculta el ecualizador y muestra una senal sinusoidal SVG continua. En la vista de RAM siempre se muestra el ecualizador. Los colores siguen la metrica visible.

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

- `src-tauri/src/main.rs`: mide CPU, RAM, procesos y OpenCode una vez por segundo.
- `src/main.ts`: alterna CPU/RAM y cambia entre el ecualizador y la senal SVG animada.
- `src/style.css`: define el cristal, halo, estela, escaneo y estados visuales.

HAL mide la CPU global deliberadamente. La deteccion de OpenCode solo modifica el patron y frecuencia de animacion; no registra actividad, no analiza contenido y no cambia la metrica mostrada.
