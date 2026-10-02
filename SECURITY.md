# Politica de seguridad

## Versiones compatibles

Solo la version publicada mas reciente recibira correcciones de seguridad.

## Reportar una vulnerabilidad

No publiques vulnerabilidades ni detalles de explotacion en incidencias publicas. Cuando el repositorio este en GitHub, utiliza **Security > Report a vulnerability** para enviar un informe privado.

Incluye una descripcion clara, pasos para reproducir el problema, impacto potencial y una posible correccion si la conoces.

## Alcance

HAL funciona localmente, no recopila datos, no realiza solicitudes de red y no ejecuta IA. La deteccion local de `OpenCode.exe` se limita al nombre del proceso y no inspecciona contenido, argumentos ni archivos. Las revisiones de seguridad deben prestar especial atencion a futuras integraciones de procesos, actualizaciones automaticas y permisos de Tauri.
