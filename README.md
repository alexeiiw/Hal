# HAL 0.7.0

HAL es un indicador de escritorio minimalista para Windows. Muestra CPU, RAM, clima y hora en una ventana flotante, transparente y click-through.

## Comportamiento

- Se ejecuta manualmente cuando se necesita; no se inicia con Windows.
- Permanece sobre otras ventanas, pero los clics atraviesan HAL hacia la aplicacion de debajo.
- Se coloca abajo a la derecha del area util del monitor.
- Alterna CPU global, RAM principal, clima y reloj analogico cada cuatro segundos.
- Detecta la ejecucion de `OpenCode.exe` y sustituye el ecualizador por una onda sinusoidal SVG sin cambiar la metrica global.
- Consulta la temperatura y condicion meteorologica al iniciar y cada 30 minutos mediante Open-Meteo.
- Solicita la ubicacion del sistema para el clima. Si se deniega el permiso o falla la consulta, muestra `CLIMA NO DISP.`.

| Vista | Umbral medio | Umbral alto | Colores |
| --- | --- | --- | --- |
| CPU | 40% | 70% | Verde, azul, rojo |
| RAM | 60% | 80% | Verde, azul, rojo |
| Clima | No aplica | No aplica | Semantico segun la condicion |
| Reloj | No aplica | No aplica | Esfera tecnica verde de HAL |

Cada vista conserva su metrica durante cuatro segundos. Las barras usan `Math.random()` acotado por la metrica visible. La vista de clima muestra un icono SVG de sol verde, nubes y niebla celeste, lluvia azul, nieve blanco azulado o tormenta violeta. El reloj analogico muestra solo hora y minutos, con la lectura `HAL HH:MM`.

## Modo OpenCode

Durante la vista de CPU, si `OpenCode.exe` esta en ejecucion, HAL oculta el ecualizador y muestra una senal sinusoidal SVG continua. En la vista de RAM se muestra el ecualizador; en clima, el icono meteorologico; y en reloj, la esfera analogica. Los colores de CPU y RAM siguen la metrica visible.

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

- NSIS: `src-tauri\target\release\bundle\nsis\HAL_0.7.0_x64-setup.exe`
- MSI: `src-tauri\target\release\bundle\msi\HAL_0.7.0_x64_en-US.msi`

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
- `src/main.ts`: alterna CPU, RAM, clima y reloj; obtiene el clima con geolocalizacion y Open-Meteo.
- `src/style.css`: define el cristal, halo, estela, escaneo, iconos y reloj analogico SVG.

HAL mide la CPU global deliberadamente. La deteccion de OpenCode solo modifica el patron y frecuencia de animacion; no registra actividad, no analiza contenido y no cambia la metrica mostrada. Para el clima, el navegador solicita ubicacion y envia las coordenadas a Open-Meteo; HAL no las guarda. El reloj no realiza llamadas ni crea temporizadores propios: reutiliza el ciclo visual y cambia sus agujas solo al cambiar el minuto.
