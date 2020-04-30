Pizarra
=======

Nada más que eso, una pizarra.

## Características

* espacio de dibujo infinito¹
* zoom infinito¹
* guardado en formato svg
* integración con tableta digitalizadora

## ¿Dónde funciona?

Debería funcionar en cualquier plataforma en la que funcione Rust y OpenGl pero solo lo he logrado correr en linux.

## Para ejecutar

`cargo run`

## ¿Y cómo se ve esto?

En este momento se ve así:

![Vista de la tercera versión de la pizarra](https://categulario.tk/pizarra.png)

---------------------------
1. Ok, no es tan infinito. El espacio de dibujo guarda coordenadas en números flotantes, entonces está sujeto a sus límites. El zoom usa un entero de 32 bits, entonces también hay un número finito de niveles de zoom pero para fines prácticos esto no importa.
