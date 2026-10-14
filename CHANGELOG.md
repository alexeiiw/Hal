# Changelog

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
