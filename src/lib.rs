/*!
Pizarra pretende ser una herramienta para hacer presentaciones en vivo, por que
creo que tener un pizarrón y gises es lo mejor para explicar cosas.

La mejor forma de utilizar pizarra es teniendo una tabla de dibujo.

# Atajos de teclado

Pizarra por el momento se puede controlar con algunos atajos de teclado que
permiten cambiar la herramienta en uso, colores y otras configuraciones.

## Herramientas de dibujo

Para dibujar es necesario hacer click con el botón izquierdo del mouse y
moverlo por el área de dibujo. Puedes cambiar la herramienta usando los
siguientes atajos:

* `Ctrl+R` Rectángulo
* `Ctrl+L` Línea

## Colores

* `Alt+G` Verde
* `Alt+R` Rojo
* `Alt+B` Azul
* `Alt+Y` Amarillo
* `Alt+O` Naranja
* `Alt+W` Blanco

## Desplazamiento

* Puedes moverte por el canvas usando el botón de enmedio del ratón
* Para cambiar el nivel de zoom se usan las teclas `+` y `-`
* Puedes regresar al centro de la vista (incluído zoom) usando la tecla `0`

# Archivo de salida

Pizarra guarda actualmente un archivo svg en el directorio en que es ejecutada
con el dibujo hecho durante la sesión. El nombre del archivo es único y lleva
la fecha en hora local en que se hizo.

*/

pub mod color;
pub mod shape;
pub mod storage;
pub mod app;
pub mod draw_commands;
pub mod action;
pub mod point;
pub mod graphics;

pub use shape::ShapeType;
pub use app::App;
