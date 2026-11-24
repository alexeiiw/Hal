# HAL 1.9.7

HAL es un indicador de escritorio minimalista para Windows. Muestra CPU, RAM, red (ancho de banda), clima, hora, Bitcoin, creditos de GitHub Copilot, saldo de DeepSeek, actividad de OpenCode, la agenda y el correo de Outlook, un buscapersonas con la tarea prioritaria, una mascota HAL y un marco de fotos en una ventana flotante, transparente y click-through. Se controla opcionalmente desde Telegram.

## Comportamiento

- Se ejecuta manualmente cuando se necesita; no se inicia con Windows.
- Reproduce un beeper al mostrarse un aviso: citas de Outlook, correo nuevo o `/aviso` de Telegram.
- Permanece sobre otras ventanas, pero los clics atraviesan HAL hacia la aplicacion de debajo.
- Se coloca abajo a la derecha del area util del monitor.
- Se muestra como un HUD sin borde, fondo ni panel de cristal para dar prioridad a los iconos.
- Alterna CPU global, RAM principal, red, clima, reloj analogico, agenda de Outlook, Bitcoin, Copilot, DeepSeek, OpenCode, mascota HAL y foto cada cuatro segundos (seis en OpenCode); añade tarea prioritaria si esta definida.
- Muestra la RAM como un anillo de progreso con el porcentaje en el centro, y un pie con las 3 apps que mas memoria usan (agrupadas por nombre de proceso, por ejemplo `BRAVE 2.1 GB`). Cada 10 segundos.
- Detecta la ejecucion de `OpenCode.exe` y, en la vista de CPU, sustituye el ecualizador por una onda sinusoidal SVG sin cambiar la metrica global.
- Consulta la temperatura y condicion meteorologica al iniciar y cada 30 minutos mediante Open-Meteo.
- Traduce las coordenadas a un nombre de ciudad con BigDataCloud para mostrarlo bajo el icono del clima. La consulta sale desde el backend de HAL, no desde la ventana.
- Solicita la ubicacion del sistema para el clima. Si se deniega el permiso o falla la consulta, muestra `CLIMA NO DISP.`.
- Consulta los creditos de Copilot al iniciar y cada 30 minutos desde `%APPDATA%\HAL\copilot.json`.
- Consulta el saldo de DeepSeek al iniciar y cada 30 minutos desde `%APPDATA%\HAL\deepseek.json`.
- Consulta el servidor local de OpenCode cada 5 segundos desde `%APPDATA%\HAL\opencode.json` y muestra estado, herramienta, archivo, sesion, modelo, tokens, coste, permisos y rama.
- Lee la agenda y el correo de Outlook cada 45 segundos mientras Outlook clasico esta abierto; nunca arranca Outlook por su cuenta.

| Vista | Umbral medio | Umbral alto | Colores |
| --- | --- | --- | --- |
| CPU | 40% | 70% | Verde, azul, rojo |
| RAM | 60% | 80% | Anillo verde, azul, rojo |
| Red | No aplica | No aplica | Onda de bajada verde y subida celeste |
| Clima | No aplica | No aplica | Semantico segun la condicion |
| Reloj | No aplica | No aplica | Esfera tecnica verde de HAL |
| Agenda | No aplica | 15 min antes de la cita | Tarjeta con hora, asunto y pie `HOY n · MAIL n` en el color del modo |
| Correo | No aplica | No aplica | Tarjeta con remitente, asunto y estado del ultimo correo recibido |
| Bitcoin | No aplica | $76,000 USD visual | Naranja, verde/rojo y alerta roja |
| Copilot | 70% usado | 90% usado | Anillo en el color del modo, azul y rojo |
| DeepSeek | No aplica | `alert_below` | Anillo y saldo en el color del modo, amarillo y rojo |
| OpenCode | No aplica | No aplica | Tarjeta terminal en el color del modo |
| Mascota HAL | No aplica | No aplica | Cabeza con visor, ojos, brillo y antena en el color del modo |
| Tarea | No aplica | No aplica | Buscapersonas Motorola con LCD verde y mensaje en mayusculas |
| Foto | No aplica | No aplica | Foto local a pantalla completa |

Cada vista conserva su metrica durante cuatro segundos. Las barras de CPU usan `Math.random()` acotado por la metrica visible. La vista de clima muestra un icono SVG de sol verde, nubes y niebla celeste, lluvia azul, nieve blanco azulado o tormenta violeta, con la ciudad bajo el icono y la lectura de temperatura y condicion en la linea inferior. Si el servicio de geocodificacion inversa no responde, el icono y la temperatura siguen visibles sin ciudad. El reloj analogico muestra solo hora y minutos, con la lectura `HAL HH:MM`. La vista de red muestra la velocidad de bajada y subida en MB/s con dos ondas (bajada solida y subida punteada).

## Bitcoin

HAL consulta BTC/USD en CoinGecko al iniciar y cada seis horas para la vista. Guarda localmente el ultimo precio valido durante un maximo de 24 horas para mantener la lectura si falla la red o el servicio. La flecha verde o roja compara la ultima consulta con la anterior. Cuando BTC llega a `$76,000 USD` o menos, el icono y la lectura se vuelven rojos como alerta visual de compra.

La alerta remota de Telegram se consulta al iniciar y cada 30 minutos, y tiene un umbral independiente configurable. Por defecto avisa al llegar a `$76,000 USD` o menos. Usa `/btc alert 80000` para fijar otro precio, `/btc alert` para consultarlo y `/btc off` para desactivarla. HAL avisa una vez al cruzar hacia abajo; se rearma solamente cuando el precio vuelve a estar por encima del umbral. El precio acepta formatos como `80000`, `80,000` o `$80,000` y debe estar entre `$1,000` y `$10,000,000 USD`.

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

## OpenCode

La vista OpenCode (`/on oc`, tambien `opencode`) consulta solo el servidor local de OpenCode y muestra una tarjeta terminal con el color del modo activo. Mientras la vista esta en pantalla cambia sus detalles cada 1,5 segundos y permanece 6 segundos en el ciclo. Muestra el estado (`TRABAJANDO`, `EN ESPERA` o `PERMISO`), los permisos pendientes, la herramienta en uso con el archivo implicado (`EDITANDO main.ts`, `LEYENDO main.rs`, `EJECUTANDO`, `BUSCANDO`), el titulo de la sesion, el modelo, los tokens con su coste en USD, la rama y los archivos modificados. Si no hay una sesion en ejecucion, usa la ultima sesion registrada en lugar de quedarse en `EN ESPERA` sin datos. Si el servidor no responde, muestra `SIN SERVIDOR`; HAL sigue funcionando sin OpenCode.

HAL lee `%APPDATA%\HAL\opencode.json`:

```json
{
  "url": "http://127.0.0.1:4096",
  "refresh_seconds": 5
}
```

`url` apunta al servidor local de OpenCode y `refresh_seconds` es opcional (2 a 60; por defecto 5). HAL consulta `GET /api/session/active`, `GET /api/session?limit=1&order=desc`, `GET /api/session/{id}`, `GET /api/session/{id}/message?limit=1&order=desc`, `GET /api/permission/request`, `GET /api/vcs/branch`, `GET /api/vcs/status` y `GET /api/experimental/usage/limits`. Del mensaje solo extrae el nombre de la herramienta, su estado y la ruta del archivo; no lee prompts, respuestas, razonamiento ni contenido de archivos, y no ejecuta acciones en OpenCode. Para el servicio local estándar, HAL lee la contraseña privada de `%USERPROFILE%\.config\opencode\service.json` y la usa solo en `127.0.0.1`; no se copia en `opencode.json` ni se registra. Si ejecutas otro servidor protegido, puedes añadir temporalmente un campo `password` en `opencode.json`. Si tu servidor usa otro puerto, actualiza `url`. No expongas ese servidor a Internet para HAL.

El API de OpenCode no informa cuanto tiempo queda en los modelos gratuitos. `GET /api/experimental/usage/limits` solo devuelve datos (`rolling`, `weekly`, `monthly` con porcentaje y fecha de renovacion) cuando hay una sesion iniciada en OpenCode Console; sin ella responde `data: null`, y los limites del nivel gratuito solo se manifiestan como errores de rate limit al generar. HAL muestra `GO <porcentaje>%` cuando ese dato existe.

## Outlook

La vista de agenda (`/on agenda`, tambien `calendario` o `citas`) lee el calendario y la bandeja de entrada de las versiones clasicas de Outlook instaladas en el equipo, mediante COM y el perfil ya configurado. No usa Microsoft Graph, no necesita permisos de administrador ni consentimiento de TI, y no escribe nada en el buzon: solo lectura.

La vista muestra la proxima cita con su hora y asunto en dos lineas de 18 caracteres, y el pie `HOY n · MAIL n` con las citas que quedan hoy y el correo sin leer. Si no quedan citas hoy, la lista continua con las de mañana; si no hay ninguna en 48 horas, muestra `SIN CITAS` y sigue mostrando el correo pendiente. Si Outlook esta cerrado, la vista queda en `SIN ACCESO` con el pie `OUTLOOK CERRADO`, y si el acceso COM falla, `ERROR OUTLOOK` en ambar. HAL nunca abre Outlook: toma los datos de la instancia que el usuario ya tiene abierta, aunque este minimizada.

Cada 15 minutos antes de una cita, HAL muestra un aviso en la ventana con la hora y el asunto y, si Telegram esta vinculado, envia el mismo aviso al chat. El aviso se emite una sola vez por cita, usando el identificador global y la hora de inicio, y se guarda en `%APPDATA%\HAL\outlook-avisos.json` con dos dias de antiguedad maxima.

Para el correo, HAL lee solo remitente, asunto, fecha y estado de los cinco mensajes mas recientes de la bandeja de entrada; nunca lee el cuerpo ni los adjuntos. Cuando llega correo nuevo sin leer muestra una vista previa de 8 segundos como maximo cada dos minutos, agrupada si llegan varios. El modo del preview se controla con `/mail preview asunto`, `remitente`, `ambos` u `off`, y el aviso por Telegram es independiente con `/mail aviso on` y `/mail aviso off` (desactivado por defecto).

La integracion se apoya en un puente de PowerShell persistente (`outlook-bridge.ps1`) que HAL escribe en `%APPDATA%\HAL` al arrancar y vigila por si deja de responder; si el proceso muere, HAL lo reinicia. El puente se ejecuta sin ventana y con `-ExecutionPolicy Bypass`, por lo que funciona con la politica de ejecucion restringida del equipo. HAL comprueba antes que `outlook.exe` este en ejecucion; si no lo esta, el puente no toca COM y solo informa del estado.

## Tarea

La vista de tarea sustituye el mensaje inferior por un buscapersonas Motorola Advisor recreado en CSS y SVG: cuerpo oscuro, altavoz, LED de mensaje nuevo, LCD verde de tres lineas, hora y fecha, teclas decorativas, D-pad y la marca MOTOROLA ADVISOR. Solo aparece cuando hay una tarea definida; con `/task clear` desaparece del ciclo.

El mensaje se escribe en mayusculas y se reparte en hasta tres lineas de 18 caracteres, por lo que una tarea de 30 caracteres cabe en dos lineas. Si una linea excede el ancho del LCD se comprime ligeramente para no recortar el texto. El reloj del pager usa formato de 24 horas (`HH:MM`) y la fecha `DD/MM/AA`. El LED parpadea durante unos tres segundos cuando llega una tarea nueva. El LCD mantiene el verde fijo y no hereda el color del modo. La ventana es click-through, asi que las teclas y el D-pad son decorativos.

La fuente pixel Silkscreen (licencia OFL, incluida en `src/assets`) se carga localmente, sin depender de Internet, con Consolas como respaldo.

## Mascota HAL

La vista de mascota muestra una cabeza SVG con casco redondeado, visor, dos ojos luminosos con brillo y una antena; su lectura es `HAL EN LINEA`. Los modos añaden capucha para developer, lentes para focus, audifonos con notas para relax y, en happy, ojos cerrados, sonrisa y rubor. Todos los trazos usan el color del modo, de modo que la cara es estable y no cambia con la carga de CPU.

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
| `/on nombre` | Activa `cpu`, `ram`, `red`, `clima`, `reloj`, `btc`, `cara`, `tarea`, `foto`, `cop`, `ds`, `oc`, `agenda` o `correo` |
| `/off nombre` | Desactiva una vista del ciclo |
| `/off hal` | Cierra HAL de forma remota |
| `/reiniciar` | Reinicia HAL de forma remota (confirma por Telegram y vuelve a abrirlo) |
| `/btc alert 80000` | Fija el aviso remoto de BTC cuando el precio sea igual o menor a $80,000 USD |
| `/btc alert` | Muestra el umbral remoto de BTC actual |
| `/btc off` | Desactiva el aviso remoto de BTC |
| `/phrase texto` | Define una frase de hasta 30 caracteres para la mascota |
| `/phrase clear` | Restaura `HAL EN LINEA` |
| `/task texto` | Define la tarea prioritaria de hasta 30 caracteres |
| `/task clear` | Elimina la tarea y su vista del ciclo |
| `/aviso 10` | Programa un aviso dentro de 10 minutos |
| `/aviso 10 Llamar cliente` | Igual, con un texto de hasta 30 caracteres en el aviso |
| `/aviso 08:00` | Programa un aviso para la proxima hora local 08:00 |
| `/aviso 08:30 Revisar proceso` | Igual, con texto en el aviso |
| `/aviso list` | Lista los avisos activos |
| `/aviso cancel` | Cancela todos los avisos |
| `/agenda` | Muestra las proximas citas de Outlook con hora y asunto |
| `/cita` | Muestra el aviso de citas activo |
| `/cita 15` | Fija el aviso de citas a 15 minutos antes (`/cita 0` lo apaga) |
| `/mail` | Muestra el correo sin leer y los mensajes mas recientes |
| `/mail aviso on` | Activa el aviso de correo nuevo por Telegram |
| `/mail aviso off` | Desactiva el aviso de correo por Telegram |
| `/mail preview asunto` | Cambia la vista previa de correo a solo asunto (`remitente`, `ambos` u `off`) |
| `/outlook` | Muestra el estado de la conexion con Outlook |
| `/memoria` | Muestra la RAM usada y el top 5 de apps por memoria en texto |
| `/status` | Informa CPU, RAM, BTC y su alerta remota, Copilot, DeepSeek, OpenCode (estado, herramienta, tokens y coste), Outlook (sin leer y proxima cita), modo, tarea y vistas activas |

Los modos, las vistas activas, la frase, la tarea, los avisos y el umbral remoto de BTC se guardan localmente en HAL y se conservan al reiniciar. Al cumplirse un aviso, HAL muestra el ojo rojo de HAL con pulso y parpadeo durante 30 segundos y Telegram recibe una confirmacion. `/status` informa CPU, RAM, BTC y su alerta, Copilot, DeepSeek, OpenCode, modo, tarea y vistas activas. Telegram solo se consulta para recibir comandos; si no hay token o red, HAL sigue funcionando normalmente. La vista Copilot se controla igual con `/on cop` y `/off cop`, su token se guarda aparte en `copilot.json`; DeepSeek usa `/on ds` y `deepseek.json`; OpenCode usa `/on oc` y `opencode.json`.

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

- NSIS: `src-tauri\target\release\bundle\nsis\HAL_1.9.7_x64-setup.exe`
- MSI: `src-tauri\target\release\bundle\msi\HAL_1.9.7_x64_en-US.msi`

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

- `src-tauri/src/main.rs`: mide CPU, RAM, red, procesos y OpenCode una vez por segundo; recibe comandos, mantiene avisos, envia alertas de Telegram, consulta los creditos de Copilot, el saldo de DeepSeek y el estado de OpenCode, y expone las fotos locales.
- `src-tauri/src/outlook.rs`: lanza y vigila el puente de Outlook, normaliza la agenda y el correo, decide los avisos de cita y correo, y evita duplicados con `%APPDATA%\HAL\outlook-avisos.json`.
- `src-tauri/resources/outlook-bridge.ps1`: proceso persistente en PowerShell que lee Outlook por COM (solo lectura) y entrega una linea JSON por ciclo.
- `src/main.ts`: alterna CPU, RAM, red, clima, reloj, agenda, Bitcoin, Copilot, DeepSeek, mascota, tarea y foto; conserva modos, vistas, frase y tarea localmente; reparte el mensaje del pager, actualiza su reloj y muestra la ciudad del clima.
- `src/style.css`: define el HUD, iconos, anillo de RAM, widget de red, anillos de Copilot y DeepSeek, tarjeta de agenda, marco de fotos, reloj analogico SVG, mascota por modo, buscapersonas de tarea, avisos y estado visual de Bitcoin.

HAL mide la CPU global deliberadamente. La deteccion de OpenCode solo modifica el patron y frecuencia de animacion; no registra actividad, no analiza contenido y no cambia la metrica mostrada. Para el clima, el navegador solicita ubicacion, envia las coordenadas a Open-Meteo y a BigDataCloud (geocodificacion inversa) y solo guarda en memoria el nombre de la ciudad; HAL no las almacena. El reloj no realiza llamadas ni crea temporizadores propios: reutiliza el ciclo visual y cambia sus agujas solo al cambiar el minuto. Para Bitcoin, HAL guarda solo el precio, el precio anterior y la fecha de consulta en el almacenamiento local. Para Copilot, HAL lee `%APPDATA%\HAL\copilot.json` y consulta `api.github.com` con el token del usuario; no envia esos datos a ningun otro servicio. Para DeepSeek, HAL lee `%APPDATA%\HAL\deepseek.json` y consulta `api.deepseek.com` con la clave del usuario; tampoco envia esos datos a otros servicios. Para Outlook, HAL solo lee del perfil local lo necesario para la agenda y el preview (remitente, asunto, fecha y estado), no accede al cuerpo ni a los adjuntos y no envia nada a Internet salvo el texto del aviso a Telegram cuando el usuario lo activa.

## Licencia

MIT. Consulta [LICENSE](LICENSE).

La tipografia Silkscreen incluida en `src/assets` se distribuye bajo la SIL Open Font License 1.1 (`src/assets/Silkscreen-OFL.txt`) y no queda cubierta por esta licencia.

El sonido `src/assets/beeper.wav` (origen: BigSoundBank, «ALRMElec_Digital pager», ID 3513) no es de esta licencia; revisa sus condiciones de uso en su pagina de origen antes de redistribuir HAL.
