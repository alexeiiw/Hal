# HAL 1.2.0

HAL es un indicador de escritorio minimalista para Windows. Muestra CPU, RAM, clima, hora, Bitcoin, tarea prioritaria y una mascota HAL en una ventana flotante, transparente y click-through. Se controla opcionalmente desde Telegram.

## Comportamiento

- Se ejecuta manualmente cuando se necesita; no se inicia con Windows.
- Permanece sobre otras ventanas, pero los clics atraviesan HAL hacia la aplicacion de debajo.
- Se coloca abajo a la derecha del area util del monitor.
- Se muestra como un HUD sin borde, fondo ni panel de cristal para dar prioridad a los iconos.
- Alterna CPU global, RAM principal, clima, reloj analogico, Bitcoin y mascota HAL cada cuatro segundos; añade tarea prioritaria si esta definida.
- Detecta la ejecucion de `OpenCode.exe` y sustituye el ecualizador por una onda sinusoidal SVG sin cambiar la metrica global.
- Consulta la temperatura y condicion meteorologica al iniciar y cada 30 minutos mediante Open-Meteo.
- Solicita la ubicacion del sistema para el clima. Si se deniega el permiso o falla la consulta, muestra `CLIMA NO DISP.`.

| Vista | Umbral medio | Umbral alto | Colores |
| --- | --- | --- | --- |
| CPU | 40% | 70% | Verde, azul, rojo |
| RAM | 60% | 80% | Verde, azul, rojo |
| Clima | No aplica | No aplica | Semantico segun la condicion |
| Reloj | No aplica | No aplica | Esfera tecnica verde de HAL |
| Bitcoin | No aplica | $76,000 USD | Naranja, verde/rojo y alerta roja |
| Mascota HAL | No aplica | No aplica | Carcasa, ojos y sensores verdes |
| Tarea | No aplica | No aplica | Mascota con insignia de prioridad |

Cada vista conserva su metrica durante cuatro segundos. Las barras usan `Math.random()` acotado por la metrica visible. La vista de clima muestra un icono SVG de sol verde, nubes y niebla celeste, lluvia azul, nieve blanco azulado o tormenta violeta. El reloj analogico muestra solo hora y minutos, con la lectura `HAL HH:MM`.

## Bitcoin

HAL consulta BTC/USD en CoinGecko al iniciar y cada seis horas. Guarda localmente el ultimo precio valido durante un maximo de 24 horas para mantener la lectura si falla la red o el servicio. La flecha verde o roja compara la ultima consulta con la anterior. Cuando BTC llega a `$76,000 USD` o menos, el icono y la lectura se vuelven rojos como alerta visual de compra.

## Mascota HAL

La ultima vista del ciclo muestra una mascota SVG estatica con carcasa ovalada y ojos. No descarga imagenes ni ejecuta animaciones adicionales; su lectura es `HAL EN LINEA`. Los modos usan capucha para developer, lentes para focus, audifonos completos para relax y sonrisa amarilla para happy.

## Telegram

HAL admite control remoto mediante long polling de Telegram. Al iniciarse crea el archivo local `%APPDATA%\HAL\telegram.json` si no existe. Coloca el token de `@BotFather` en el campo `token` y reinicia HAL:

```json
{
  "token": "PEGA_AQUI_EL_TOKEN_DE_BOTFATHER",
  "chat_id": null,
  "offset": null
}
```

El archivo esta fuera del proyecto y no se publica en Git. Despues, envia `/pair` al bot para vincular el primer chat. HAL solo aceptara comandos de ese chat hasta que se borre `chat_id` del archivo con HAL cerrado.

| Comando | Funcion |
| --- | --- |
| `/pair` | Vincula el primer chat autorizado o confirma la vinculacion actual |
| `/inventory` | Muestra las vistas activas y desactivadas, junto a los modos disponibles |
| `/mode normal` | Paleta y cara original de HAL |
| `/mode developer` | Capucha y acentos cian |
| `/mode focus` | Lentes y acentos azules |
| `/mode relax` | Auriculares y acentos violetas |
| `/on nombre` | Activa `cpu`, `ram`, `weather`, `clock`, `bitcoin` o `face` |
| `/off nombre` | Desactiva una vista del ciclo |
| `/off hal` | Cierra HAL de forma remota |
| `/phrase texto` | Define una frase de hasta 30 caracteres para la mascota |
| `/phrase clear` | Restaura `HAL EN LINEA` |
| `/task texto` | Define la tarea prioritaria de hasta 30 caracteres |
| `/task clear` | Elimina la tarea y su vista del ciclo |
| `/aviso 10` | Programa un aviso dentro de 10 minutos |
| `/aviso 08:00` | Programa un aviso para la proxima hora local 08:00 |
| `/aviso list` | Lista los avisos activos |
| `/aviso cancel` | Cancela todos los avisos |
| `/status` | Confirma que el bot esta vinculado |

Los modos, las vistas activas, la frase, la tarea y los avisos se guardan localmente en HAL y se conservan al reiniciar. Al cumplirse un aviso, HAL muestra una alerta visual durante 30 segundos y Telegram recibe una confirmacion. `/status` informa CPU, RAM, BTC, modo, tarea y vistas activas. Telegram solo se consulta para recibir comandos; si no hay token o red, HAL sigue funcionando normalmente.

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

- NSIS: `src-tauri\target\release\bundle\nsis\HAL_1.2.0_x64-setup.exe`
- MSI: `src-tauri\target\release\bundle\msi\HAL_1.2.0_x64_en-US.msi`

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

- `src-tauri/src/main.rs`: mide CPU, RAM, procesos y OpenCode una vez por segundo; recibe comandos, mantiene avisos y envia alertas de Telegram.
- `src/main.ts`: alterna CPU, RAM, clima, reloj, Bitcoin, mascota y tarea; conserva modos, vistas, frase y tarea localmente.
- `src/style.css`: define el HUD, iconos, reloj analogico SVG, mascota por modo, tarea, avisos y estado visual de Bitcoin.

HAL mide la CPU global deliberadamente. La deteccion de OpenCode solo modifica el patron y frecuencia de animacion; no registra actividad, no analiza contenido y no cambia la metrica mostrada. Para el clima, el navegador solicita ubicacion y envia las coordenadas a Open-Meteo; HAL no las guarda. El reloj no realiza llamadas ni crea temporizadores propios: reutiliza el ciclo visual y cambia sus agujas solo al cambiar el minuto. Para Bitcoin, HAL guarda solo el precio, el precio anterior y la fecha de consulta en el almacenamiento local.
