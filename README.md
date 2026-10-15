# HAL 1.4.0

HAL es un indicador de escritorio minimalista para Windows. Muestra CPU, RAM, red (ancho de banda), clima, hora, Bitcoin, tarea prioritaria, una mascota HAL y un marco de fotos en una ventana flotante, transparente y click-through. Se controla opcionalmente desde Telegram.

## Comportamiento

- Se ejecuta manualmente cuando se necesita; no se inicia con Windows.
- Permanece sobre otras ventanas, pero los clics atraviesan HAL hacia la aplicacion de debajo.
- Se coloca abajo a la derecha del area util del monitor.
- Se muestra como un HUD sin borde, fondo ni panel de cristal para dar prioridad a los iconos.
- Alterna CPU global, RAM principal, red, clima, reloj analogico, Bitcoin, mascota HAL y foto cada cuatro segundos; añade tarea prioritaria si esta definida.
- Muestra la RAM como un anillo de progreso con el porcentaje en el centro.
- Detecta la ejecucion de `OpenCode.exe` y, en la vista de CPU, sustituye el ecualizador por una onda sinusoidal SVG sin cambiar la metrica global.
- Consulta la temperatura y condicion meteorologica al iniciar y cada 30 minutos mediante Open-Meteo.
- Solicita la ubicacion del sistema para el clima. Si se deniega el permiso o falla la consulta, muestra `CLIMA NO DISP.`.

| Vista | Umbral medio | Umbral alto | Colores |
| --- | --- | --- | --- |
| CPU | 40% | 70% | Verde, azul, rojo |
| RAM | 60% | 80% | Anillo verde, azul, rojo |
| Red | No aplica | No aplica | Onda de bajada verde y subida celeste |
| Clima | No aplica | No aplica | Semantico segun la condicion |
| Reloj | No aplica | No aplica | Esfera tecnica verde de HAL |
| Bitcoin | No aplica | $76,000 USD | Naranja, verde/rojo y alerta roja |
| Mascota HAL | No aplica | No aplica | Cabeza con visor, ojos, brillo y antena en el color del modo |
| Tarea | No aplica | No aplica | Lente unico estilo HAL con insignia de prioridad |
| Foto | No aplica | No aplica | Foto local a pantalla completa |

Cada vista conserva su metrica durante cuatro segundos. Las barras de CPU usan `Math.random()` acotado por la metrica visible. La vista de clima muestra un icono SVG de sol verde, nubes y niebla celeste, lluvia azul, nieve blanco azulado o tormenta violeta. El reloj analogico muestra solo hora y minutos, con la lectura `HAL HH:MM`. La vista de red muestra la velocidad de bajada y subida en MB/s con dos ondas (bajada solida y subida punteada).

## Bitcoin

HAL consulta BTC/USD en CoinGecko al iniciar y cada seis horas. Guarda localmente el ultimo precio valido durante un maximo de 24 horas para mantener la lectura si falla la red o el servicio. La flecha verde o roja compara la ultima consulta con la anterior. Cuando BTC llega a `$76,000 USD` o menos, el icono y la lectura se vuelven rojos como alerta visual de compra.

## Red

La vista de red (`/on red`, tambien `net` o `banda`) muestra el ancho de banda en tiempo real: la velocidad de descarga y de subida en MB/s, con una onda solida para la bajada y una onda punteada celeste para la subida. Los datos se obtienen de forma nativa con `sysinfo` (`Networks`), sumando todas las interfaces y calculando el delta por segundo, sin dependencias nuevas. Se actualiza una vez por segundo.

## Mascota HAL

La vista de mascota muestra una cabeza SVG con casco redondeado, visor, dos ojos luminosos con brillo y una antena; su lectura es `HAL EN LINEA`. Los modos añaden capucha para developer, lentes para focus, audifonos con notas para relax y, en happy, ojos cerrados, sonrisa y rubor. Todos los trazos usan el color del modo (que a su vez depende del tema), de modo que la cara es estable y no cambia con la carga de CPU. La vista de tarea, en cambio, usa un lente unico estilo HAL con la insignia de prioridad.

## Temas

HAL incluye seis temas de color: `clasico`, `menta`, `durazno`, `lavanda`, `cian` y `ambar`. Cada tema define el tono base y una variante por modo, de modo que developer, focus, relax y happy conservan su identidad dentro del tema elegido. El tema `clasico` mantiene los colores originales de cada modo. Los umbrales (azul en carga media y rojo en carga alta) y la alerta de Bitcoin siempre tienen prioridad sobre el color del tema.

## Fotos

La vista de foto muestra una sola imagen local durante 24 horas: HAL elige una al azar de la carpeta `%APPDATA%\HAL\fotos` y la conserva hasta que pasan 24 horas o la foto desaparece. La foto llena por completo la superficie de HAL, sin marco ni margen: la imagen se estira al area de la ventana para rellenarla entera. La carpeta se crea automaticamente al iniciar. Si esta vacia, si el archivo falla o si no hay fotos validas, HAL muestra una imagen predeterminada con el texto `HAL FOTOS`. Se aceptan imagenes `.png`, `.jpg`, `.jpeg`, `.gif`, `.webp` y `.bmp`.

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
| `/theme clasico` | Tema de color base (tambien `menta`, `durazno`, `lavanda`, `cian` o `ambar`) |
| `/on nombre` | Activa `cpu`, `ram`, `red`, `clima`, `reloj`, `btc`, `cara`, `tarea` o `foto` |
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

Los modos, el tema, las vistas activas, la frase, la tarea y los avisos se guardan localmente en HAL y se conservan al reiniciar. Al cumplirse un aviso, HAL muestra el ojo rojo de HAL con pulso y parpadeo durante 30 segundos y Telegram recibe una confirmacion. `/status` informa CPU, RAM, BTC, modo, tema, tarea y vistas activas. Telegram solo se consulta para recibir comandos; si no hay token o red, HAL sigue funcionando normalmente.

## Modo OpenCode

Durante la vista de CPU, si `OpenCode.exe` esta en ejecucion, HAL oculta el ecualizador y muestra una senal sinusoidal SVG continua. En la vista de RAM se muestra el anillo de progreso; en clima, el icono meteorologico; y en reloj, la esfera analogica. Los colores de CPU y RAM siguen la metrica visible.

La frecuencia de la onda aumenta de forma continua segun el consumo de procesador de OpenCode: a mayor carga, mayor frecuencia, y la amplitud de la onda tambien crece con la carga. La CPU de OpenCode se calcula sumando sus procesos locales. HAL actualiza la lista de procesos cada segundo, por lo que al cerrar OpenCode vuelve automaticamente al ecualizador global. No inspecciona conversaciones, archivos ni argumentos de los procesos.

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

- NSIS: `src-tauri\target\release\bundle\nsis\HAL_1.4.0_x64-setup.exe`
- MSI: `src-tauri\target\release\bundle\msi\HAL_1.4.0_x64_en-US.msi`

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

- `src-tauri/src/main.rs`: mide CPU, RAM, red, procesos y OpenCode una vez por segundo; recibe comandos, mantiene avisos, envia alertas de Telegram y expone las fotos locales.
- `src/main.ts`: alterna CPU, RAM, red, clima, reloj, Bitcoin, mascota, tarea y foto; conserva modos, tema, vistas, frase y tarea localmente.
- `src/style.css`: define el HUD, iconos, anillo de RAM, widget de red, marco de fotos, reloj analogico SVG, mascota por modo, tarea, avisos y estado visual de Bitcoin.

HAL mide la CPU global deliberadamente. La deteccion de OpenCode solo modifica el patron y frecuencia de animacion; no registra actividad, no analiza contenido y no cambia la metrica mostrada. Para el clima, el navegador solicita ubicacion y envia las coordenadas a Open-Meteo; HAL no las guarda. El reloj no realiza llamadas ni crea temporizadores propios: reutiliza el ciclo visual y cambia sus agujas solo al cambiar el minuto. Para Bitcoin, HAL guarda solo el precio, el precio anterior y la fecha de consulta en el almacenamiento local.
