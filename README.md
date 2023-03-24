Pizarra
=======

[![pipeline status](https://gitlab.com/categulario/pizarra/badges/master/pipeline.svg)](https://gitlab.com/categulario/pizarra/-/commits/master)

Nada más que eso, una pizarra. En este repositorio vive solamente el backend, es
decir la biblioteca con las funciones necesarias para implementar una pizarra en
algún frontend (como [gtk](https://gitlab.com/categulario/pizarra-gtk) o
[webassembly](https://gitlab.com/categulario/pizarra-wasm)).

Para el código de una aplicación que puedes ejecutar en tu escritorio visita
[pizarra-gtk](https://gitlab.com/categulario/pizarra-gtk)

Para usar la versión web visita https://pizarra.categulario.xyz

## Características

* espacio de dibujo infinito¹
* zoom infinito¹
* guardado en formato svg
* exportado en formato png
* integración con tableta digitalizadora

## Desarrollo

Naturalmente, necesitas [rust](https://rust-lang.org)

Corre los tests:

    cargo test

Publica una nueva versión en [crates.io](https://crates.io)

    cargo publish

---------------------------
1. Ok, no es tan infinito. El espacio de dibujo guarda coordenadas en números
flotantes, entonces está sujeto a sus límites. El zoom está sujeto a los límites
de precisión de coma flotante.

## Publicación en crates.io

Esto es para mi, no para ti

* Sube el tag de la versión (`vbump`)
* `git push && git push --tags`
* `cargo publish`
