# Politica de seguridad

## Versiones compatibles

Solo la version publicada mas reciente recibira correcciones de seguridad.

## Reportar una vulnerabilidad

No publiques vulnerabilidades ni detalles de explotacion en incidencias publicas. Cuando el repositorio este en GitHub, utiliza **Security > Report a vulnerability** para enviar un informe privado.

Incluye una descripcion clara, pasos para reproducir el problema, impacto potencial y una posible correccion si la conoces.

## Alcance

HAL funciona localmente y no recopila datos ni ejecuta IA. Realiza solicitudes de red solo para el clima (Open-Meteo, con las coordenadas del sistema), Bitcoin (CoinGecko), Telegram (si hay token) y Copilot (`api.github.com`, si hay token). La deteccion local de `OpenCode.exe` se limita al nombre del proceso y no inspecciona contenido, argumentos ni archivos.

El token de Copilot se guarda en texto plano en `%APPDATA%\HAL\copilot.json`, fuera del repositorio, y solo se envia a `api.github.com` en la cabecera `Authorization`; nunca se registra en logs. Usa un token clasico con el unico permiso `read:user` y una expiracion corta, y revocalo en GitHub si el archivo se expone. La clave de DeepSeek se guarda igualmente en texto plano en `%APPDATA%\HAL\deepseek.json`, solo se envia a `api.deepseek.com` y tampoco se registra; usa una clave dedicada y revocala si el archivo se expone. Las revisiones de seguridad deben prestar especial atencion a futuras integraciones de procesos, actualizaciones automaticas y permisos de Tauri.
