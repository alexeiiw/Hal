# Changelog

## 1.5.0

- Unifica color y modo: cada modo define un color fijo (normal verde, developer cian, focus azul, relax violeta, happy amarillo).
- Elimina los temas de color y el comando `/theme`; se retiran el campo `theme` de la configuracion de Telegram y su estado guardado.
- La rotacion de la vista de foto pasa de 24 a 6 horas.
- Actualiza `/help` para listar todos los comandos disponibles.

## 1.4.0

- Añade la vista de red (`/on red`, alias `net`/`banda`) con ancho de banda de descarga y subida en tiempo real.
- El widget de red dibuja dos ondas (bajada solida y subida punteada) y muestra los MB/s de cada direccion.
- Los datos vienen de `sysinfo` (`Networks`), sin dependencias nuevas, muestreados cada segundo por el evento `evento-red`.
- El aviso/alarma ahora muestra un ojo rojo de HAL con pulso y parpadeo durante los 30 segundos de la alerta.

## 1.3.0

- Añade cinco temas de color (`menta`, `durazno`, `lavanda`, `cian`, `ambar`) con una variante por modo, controlados con `/theme` y conservados al reiniciar.
- Reemplaza el ecualizador de RAM por un anillo de progreso con el porcentaje en el centro.
- Hace que la onda de OpenCode aumente su frecuencia de forma continua segun el consumo de CPU.
- Añade una vista de marco de fotos que muestra una sola imagen local durante 24 horas.
- La foto llena toda la superficie de HAL sin marco, estirandose al area de la ventana.
- Muestra una imagen predeterminada `HAL FOTOS` si la carpeta esta vacia o si la foto falla.
- Añade los comandos `list_photos` y `read_photo` para leer `%APPDATA%\HAL\fotos` en base64.
- Refina las caras por modo: panel tecnico normal, borde de capucha developer, lentes focus y audifonos con notas musicales en relax.
- Rediseña la mascota como una cabeza coherente con casco, visor, ojos luminosos y antena; la tarea pasa a un lente unico estilo HAL.
- Unifica el color de la cara con el del modo (y el tema), dejando de reaccionar a la carga de CPU.

## 1.2.0

- Añade el modo happy y corrige la distribucion de los audifonos de la mascota relax.
- Añade avisos persistentes por Telegram con alerta visual de 30 segundos y confirmacion remota.
- Añade alerta remota de BTC al llegar a $76,000 USD o menos.
- Amplia `/inventory` y `/status` con el estado de vistas y sistema.

## 1.1.0

- Añade `/inventory` para consultar vistas y modos disponibles desde Telegram.
- Añade `/task` y una vista de tarea prioritaria con mascota e insignia.
- Simplifica la mascota normal y mejora los modos developer, focus y relax.

## 1.0.1

- Añade `/off hal` para cerrar HAL desde el chat autorizado de Telegram.

## 1.0.0

- Añade control desde Telegram para modos, vistas activas y frase de la mascota.
- Añade emparejamiento del primer chat mediante `/pair` y almacenamiento local del token fuera del proyecto.
- Añade variantes de mascota developer, focus y relax en SVG.

## 0.9.0

- Añade la vista final de mascota HAL con un SVG original de bajo consumo.

## 0.8.0

- Añade la vista BTC/USD con icono, flecha de variacion y alerta roja en $76,000 USD o menos.
- Consulta CoinGecko al iniciar y cada seis horas; conserva un cache local valido durante 24 horas.

## 0.7.1

- Elimina el borde, fondo de cristal, sombra y escaneo para convertir HAL en un HUD transparente.
- Amplia los iconos meteorologicos y el reloj analogico dentro de la misma ventana.

## 0.7.0

- Añade un reloj analogico propio de HAL con agujas de hora y minuto.
- Reduce el trabajo visual en las vistas estaticas de clima y reloj; el reloj actualiza sus agujas solo cuando cambia el minuto.

## 0.6.0

- Añade una vista de clima con temperatura, condicion e icono SVG dinamico.
- Obtiene el clima desde Open-Meteo con permiso de ubicacion del sistema, sin almacenar coordenadas.
- Usa colores semanticos para el icono: verde, celeste, azul, blanco azulado y violeta segun la condicion.

## 0.5.0

- Añade la vista alternada de RAM principal junto a CPU global.
- Añade cristal, halo reactivo, estela de onda, escaneo y lectura tecnica.

## 0.4.1

- Corrige el retorno al ecualizador global al cerrar OpenCode, eliminando los procesos finalizados en cada muestreo.
- Simplifica el calculo de CPU agregada de OpenCode.

## 0.4.0

- Sustituye las barras por una señal sinusoidal SVG continua mientras OpenCode esta abierto.
- Mantiene el ecualizador para el modo de CPU global.

## 0.3.1

- Corrige la deteccion de OpenCode en Windows al aceptar `OpenCode` y `OpenCode.exe`.
- Mantiene el modo de senal ondulada cuando OpenCode esta abierto.

## 0.3.0

- Añade una señal ondulada para OpenCode y ajusta su frecuencia segun la CPU agregada de sus procesos.
- Conserva el ecualizador aleatorio para la medicion global de CPU.

## 0.2.0

- Añade deteccion local de OpenCode y documentacion inicial para GitHub.

## 0.1.0

- Primera version de HAL con indicador de CPU global y ventana flotante click-through.
