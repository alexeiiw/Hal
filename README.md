# HAL 1.6.0

HAL es un indicador de escritorio minimalista para Windows. Muestra CPU, RAM, red (ancho de banda), clima, hora, Bitcoin, creditos de GitHub Copilot, saldo de DeepSeek, tarea prioritaria, una mascota HAL y un marco de fotos en una ventana flotante, transparente y click-through. Se controla opcionalmente desde Telegram.

## Comportamiento

- Se ejecuta manualmente cuando se necesita; no se inicia con Windows.
- Permanece sobre otras ventanas, pero los clics atraviesan HAL hacia la aplicacion de debajo.
- Se coloca abajo a la derecha del area util del monitor.
- Se muestra como un HUD sin borde, fondo ni panel de cristal para dar prioridad a los iconos.
- Alterna CPU global, RAM principal, red, clima, reloj analogico, Bitcoin, Copilot, DeepSeek, mascota HAL y foto cada cuatro segundos; añade tarea prioritaria si esta definida.
- Muestra la RAM como un anillo de progreso con el porcentaje en el centro.
- Detecta la ejecucion de `OpenCode.exe` y, en la vista de CPU, sustituye el ecualizador por una onda sinusoidal SVG sin cambiar la metrica global.
- Consulta la temperatura y condicion meteorologica al iniciar y cada 30 minutos mediante Open-Meteo.
- Solicita la ubicacion del sistema para el clima. Si se deniega el permiso o falla la consulta, muestra `CLIMA NO DISP.`.
- Consulta los creditos de Copilot al iniciar y cada 30 minutos desde `%APPDATA%\HAL\copilot.json`.
- Consulta el saldo de DeepSeek al iniciar y cada 30 minutos desde `%APPDATA%\HAL\deepseek.json`.

| Vista | Umbral medio | Umbral alto | Colores |
| --- | --- | --- | --- |
| CPU | 40% | 70% | Verde, azul, rojo |
| RAM | 60% | 80% | Anillo verde, azul, rojo |
| Red | No aplica | No aplica | Onda de bajada verde y subida celeste |
| Clima | No aplica | No aplica | Semantico segun la condicion |
| Reloj | No aplica | No aplica | Esfera tecnica verde de HAL |
| Bitcoin | No aplica | $76,000 USD | Naranja, verde/rojo y alerta roja |
| Copilot | 70% usado | 90% usado | Anillo en el color del modo, azul y rojo |
| DeepSeek | No aplica | `alert_below` | Anillo y saldo en el color del modo, amarillo y rojo |
| Mascota HAL | No aplica | No aplica | Cabeza con visor, ojos, brillo y antena en el color del modo |
| Tarea | No aplica | No aplica | Lente unico estilo HAL con insignia de prioridad |
| Foto | No aplica | No aplica | Foto local a pantalla completa |

Cada vista conserva su metrica durante cuatro segundos. Las barras de CPU usan `Math.random()` acotado por la metrica visible. La vista de clima muestra un icono SVG de sol verde, nubes y niebla celeste, lluvia azul, nieve blanco azulado o tormenta violeta. El reloj analogico muestra solo hora y minutos, con la lectura `HAL HH:MM`. La vista de red muestra la velocidad de bajada y subida en MB/s con dos ondas (bajada solida y subida punteada).

## Bitcoin

HAL consulta BTC/USD en CoinGecko al iniciar y cada seis horas. Guarda localmente el ultimo precio valido durante un maximo de 24 horas para mantener la lectura si falla la red o el servicio. La flecha verde o roja compara la ultima consulta con la anterior. Cuando BTC llega a `$76,000 USD` o menos, el icono y la lectura se vuelven rojos como alerta visual de compra.

## Red

La vista de red (`/on red`, tambien `net` o `banda`) muestra el ancho de banda en tiempo real: la velocidad de descarga y de subida en MB/s, con una onda solida para la bajada y una onda punteada celeste para la subida. Los datos se obtienen de forma nativa con `sysinfo` (`Networks`), sumando todas las interfaces y calculando el delta por segundo, sin dependencias nuevas. Se actualiza una vez por segundo.

## Copilot

La vista Copilot (`/on cop`, tambien `copilot`) muestra los creditos de GitHub AI del plan personal como `usados / total` (por ejemplo `1519 / 1500`), con un anillo que se llena segun el consumo. El anillo y la lectura usan el color del modo activo (normal verde, developer cian, focus azul, relax violeta, happy amarillo); al 70 % usado pasan a azul y al 90 % a rojo, y si los creditos se agotan o GitHub deja de otorgar cuota, la vista queda en rojo. Los datos se consultan al iniciar y cada 30 minutos.

HAL lee la configuracion de `%APPDATA%\HAL\copilot.json`:

```json
{
  "token": "ghp_...",
  "refresh_minutes": 30
}
```

`refresh_minutes` es opcional (1 a 1440; por defecto 30). El archivo esta fuera del proyecto y no se publica en Git. Para generar el token:

1. Abre `https://github.com/settings/tokens/new` (token clasico).
2. Marca solo el permiso `read:user`.
3. Define una expiracion (por ejemplo 90 dias) y genera el token.
4. Copia el valor `ghp_...` en el campo `token` de `copilot.json` y reinicia HAL.

Cuando el token vence o se revoca, la vista muestra `TOKEN VENCIDO`. Genera uno nuevo y reemplaza el campo `token`; HAL relee el archivo en cada ciclo, asi que no hace falta reiniciar. La consulta usa el endpoint interno `GET https://api.github.com/copilot_internal/user`, el mismo que usan los clientes de Copilot: no esta documentado oficialmente, puede cambiar sin aviso y solo devuelve datos del plan personal (no creditos facturados a una organizacion o empresa). Si falla, la vista muestra `NO DISP.` y el resto de HAL sigue funcionando. El token se envia solo en la cabecera `Authorization` y nunca se registra en logs.

## DeepSeek

La vista DeepSeek (`/on ds`, tambien `deepseek`) muestra el saldo disponible de la cuenta de platform.deepseek.com como `$1.78`, con un anillo que representa la distancia hasta el umbral de aviso. El anillo y la lectura usan el color del modo activo; al acercarse al umbral pasan a amarillo y, por debajo de el o si la cuenta no puede operar, a rojo. Lo que se muestra es `total_balance` (saldo de regalo mas saldo recargado), no el saldo de regalo por separado. Los datos se consultan al iniciar y cada 30 minutos.

HAL lee la configuracion de `%APPDATA%\HAL\deepseek.json`:

```json
{
  "api_key": "sk-...",
  "refresh_minutes": 30,
  "alert_below": 2
}
```

`refresh_minutes` es opcional (1 a 1440; por defecto 30) y `alert_below` es el umbral en la moneda de la cuenta para el aviso (por defecto 1). El archivo esta fuera del proyecto y no se publica en Git. Para generar la clave:

1. Abre `https://platform.deepseek.com/api_keys`.
2. Crea una clave de API y copiala en el momento (solo se muestra una vez).
3. Pega el valor `sk-...` en el campo `api_key` de `deepseek.json` y reinicia HAL.

La consulta usa el endpoint oficial `GET https://api.deepseek.com/user/balance` con la cabecera `Authorization`; devuelve `is_available`, `total_balance`, `granted_balance` y `topped_up_balance` en USD o CNY. Si la clave no es valida, la vista muestra `API KEY INVALIDA`; si no hay conexion, `SIN CONEXION`. HAL relee `deepseek.json` en cada ciclo, de modo que una clave nueva no requiere reiniciar. La clave se envia solo a `api.deepseek.com` y nunca se registra en logs.

## Mascota HAL

La vista de mascota muestra una cabeza SVG con casco redondeado, visor, dos ojos luminosos con brillo y una antena; su lectura es `HAL EN LINEA`. Los modos añaden capucha para developer, lentes para focus, audifonos con notas para relax y, en happy, ojos cerrados, sonrisa y rubor. Todos los trazos usan el color del modo, de modo que la cara es estable y no cambia con la carga de CPU. La vista de tarea, en cambio, usa un lente unico estilo HAL con la insignia de prioridad.

## Fotos

La vista de foto muestra una sola imagen local durante 6 horas: HAL elige una al azar de la carpeta `%APPDATA%\HAL\fotos` y la conserva hasta que pasan 6 horas o la foto desaparece. La foto llena por completo la superficie de HAL, sin marco ni margen: la imagen se estira al area de la ventana para rellenarla entera. La carpeta se crea automaticamente al iniciar. Si esta vacia, si el archivo falla o si no hay fotos validas, HAL muestra una imagen predeterminada con el texto `HAL FOTOS`. Se aceptan imagenes `.png`, `.jpg`, `.jpeg`, `.gif`, `.webp` y `.bmp`.

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
| `/help` | Muestra la lista completa de comandos |
| `/inventory` | Muestra las vistas activas y desactivadas, junto a los modos disponibles |
| `/mode normal` | Paleta y cara original de HAL |
| `/mode developer` | Capucha y acentos cian |
| `/mode focus` | Lentes y acentos azules |
| `/mode relax` | Auriculares y acentos violetas |
| `/on nombre` | Activa `cpu`, `ram`, `red`, `clima`, `reloj`, `btc`, `cara`, `tarea`, `foto`, `cop` o `ds` |
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
| `/status` | Informa CPU, RAM, BTC, Copilot, DeepSeek, modo, tarea y vistas activas |

Los modos, las vistas activas, la frase, la tarea y los avisos se guardan localmente en HAL y se conservan al reiniciar. Al cumplirse un aviso, HAL muestra el ojo rojo de HAL con pulso y parpadeo durante 30 segundos y Telegram recibe una confirmacion. `/status` informa CPU, RAM, BTC, Copilot, DeepSeek, modo, tarea y vistas activas. Telegram solo se consulta para recibir comandos; si no hay token o red, HAL sigue funcionando normalmente. La vista Copilot se controla igual con `/on cop` y `/off cop`, y su token se guarda aparte en `copilot.json`; la vista DeepSeek usa `/on ds` y su clave se guarda en `deepseek.json`.

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

- NSIS: `src-tauri\target\release\bundle\nsis\HAL_1.6.0_x64-setup.exe`
- MSI: `src-tauri\target\release\bundle\msi\HAL_1.6.0_x64_en-US.msi`

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

- `src-tauri/src/main.rs`: mide CPU, RAM, red, procesos y OpenCode una vez por segundo; recibe comandos, mantiene avisos, envia alertas de Telegram, consulta los creditos de Copilot y el saldo de DeepSeek, y expone las fotos locales.
- `src/main.ts`: alterna CPU, RAM, red, clima, reloj, Bitcoin, Copilot, DeepSeek, mascota, tarea y foto; conserva modos, vistas, frase y tarea localmente.
- `src/style.css`: define el HUD, iconos, anillo de RAM, widget de red, anillos de Copilot y DeepSeek, marco de fotos, reloj analogico SVG, mascota por modo, tarea, avisos y estado visual de Bitcoin.

HAL mide la CPU global deliberadamente. La deteccion de OpenCode solo modifica el patron y frecuencia de animacion; no registra actividad, no analiza contenido y no cambia la metrica mostrada. Para el clima, el navegador solicita ubicacion y envia las coordenadas a Open-Meteo; HAL no las guarda. El reloj no realiza llamadas ni crea temporizadores propios: reutiliza el ciclo visual y cambia sus agujas solo al cambiar el minuto. Para Bitcoin, HAL guarda solo el precio, el precio anterior y la fecha de consulta en el almacenamiento local. Para Copilot, HAL lee `%APPDATA%\HAL\copilot.json` y consulta `api.github.com` con el token del usuario; no envia esos datos a ningun otro servicio. Para DeepSeek, HAL lee `%APPDATA%\HAL\deepseek.json` y consulta `api.deepseek.com` con la clave del usuario; tampoco envia esos datos a otros servicios.
