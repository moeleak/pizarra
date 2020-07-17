Pizarra
=======

[![pipeline status](https://gitlab.com/categulario/pizarra/badges/master/pipeline.svg)](https://gitlab.com/categulario/pizarra/-/commits/master)

Nada más que eso, una pizarra. En este repositorio vive solamente el backend, es
decir la biblioteca con las funciones necesarias para implementar una pizarra en
algún frontend (como gtk).

Para el código de una aplicación que puedes ejecutar en tu escritorio visita
[pizarra-gtk](https://gitlab.com/categulario/pizarra-gtk)

## Características

* espacio de dibujo infinito¹
* zoom infinito¹
* guardado en formato svg
* exportado en formato png
* integración con tableta digitalizadora

---------------------------
1. Ok, no es tan infinito. El espacio de dibujo guarda coordenadas en números
flotantes, entonces está sujeto a sus límites. El zoom está sujeto a los límites
de precisión de coma flotante.
