Universidad del Valle de Guatemala

Facultad de Ingeniería

Gráficas por Computadora

# Laboratorio 6: planetas y estrellas con shaders

Juan Francisco Orozco Mijangos (24647)

## Descripción del proyecto

En este laboratorio recreamos los dos soles de Tatooine, Scarif, Bespin y Naboo para practicar cómo se puede cambiar por completo la apariencia de un objeto con un shader. Cargamos una sola vez la esfera que nos dieron y reutilizamos sus 98 vértices y 192 triángulos para los cinco cuerpos. En la vista de Tatooine dibujamos la malla dos veces con posiciones, escalas y materiales distintos. Los otros planetas se ven uno a la vez para apreciar sus detalles.

Reconstruimos el pipeline en Rust siguiendo el material de clase sobre el rasterizador y el dibujo de múltiples modelos. Primero pasamos cada vértice por las transformaciones de modelo y vista y lo proyectamos a pantalla. Después usamos coordenadas baricéntricas y un búfer de profundidad para decidir qué fragmentos son visibles. En cada fragmento interpolamos la posición local con corrección de perspectiva y obtenemos una normal radial. Con ella, la luz y el tiempo calculamos el color usando ruido tridimensional, máscaras y funciones matemáticas. Toda esa parte corre en CPU. Raylib abre la ventana y muestra los píxeles que ya calculamos. [Esfera](assets/sphere.obj) · [Pipeline](src/pipeline.rs) · [Shaders](src/shaders.rs)

## Compilación y ejecución

Se necesita Rust estable con Cargo, CMake y un compilador C/C++ con Clang. En macOS, las herramientas de línea de comandos de Xcode proporcionan Clang. Si hacen falta, se pueden instalar con `xcode-select --install` y, con Homebrew, instalar CMake con `brew install cmake`.

Desde la raíz del repositorio:

```sh
cargo build --release --locked
cargo run --release --locked
```

| Tecla | Acción |
| --- | --- |
| `1` | Mostrar juntos los dos soles de Tatooine |
| `2`, `3`, `4` | Mostrar Scarif, Bespin o Naboo, respectivamente |
| `←` / `→` | Girar la dirección de la luz |
| `↑` / `↓` | Aumentar o reducir la cobertura de nubes de Scarif |
| Espacio | Pausar o reanudar la animación del material y las nubes. La luz se puede mover en pausa. |
| `S` | Alternar las sombras de nubes de Scarif para compararlas con la superficie iluminada |
| `R` | Restaurar el tiempo, la luz, la cobertura y las sombras a sus valores iniciales |
| Esc | Cerrar la ventana |

## Cuerpos celestes

### Los dos soles de Tatooine

**Izquierda, sol ámbar.** Combinamos gránulos finos con ruido de menor frecuencia para distribuir manchas oscuras. Otro campo de ruido deforma el plasma y el tiempo hace que cambie la red de filamentos dorados. `PLASMA_SPEED` cambia la velocidad de esa deformación. El umbral de `spots` controla cuántas manchas aparecen.

**Derecha, sol rojo.** Estiramos el patrón de ruido en latitud para crear vetas más largas y usamos arcos rojizos de gran escala para separar regiones calientes. El vector de frecuencia en `storm` controla cuánto se estiran las vetas. Ambos soles emiten su propio color y se animan con la geometría inmóvil. [Animación de los soles](docs/soles.mp4)

![Los soles de Tatooine, ámbar a la izquierda y rojo a la derecha](docs/soles.png)

### Scarif

Usamos un campo de altura para separar la selva, la arena, las aguas someras y el océano profundo. El umbral de `land` mueve la línea de costa y el de `shallow` cambia el ancho del agua turquesa. La máscara de tierra reserva el reflejo especular para el agua. Agregamos luces costeras en la noche y combinamos el terreno con una máscara de nubes que se desplaza por separado. [Movimiento de las nubes](docs/scarif-nubes.mp4) · [Barrido de luz](docs/scarif-luz.mp4)

![Scarif con archipiélagos verdes, aguas turquesa y nubes dispersas](docs/scarif.png)

### Bespin

Mezclamos ondas de distintas frecuencias con ruido para producir bandas crema, rosa y malva. Cerca de la tormenta rotamos las coordenadas antes de evaluar las bandas. La rotación disminuye con la distancia, así las franjas se curvan alrededor del vórtice y vuelven a su dirección original al alejarse. `VORTEX_RADIUS` cambia el tamaño de la zona afectada.

![Bespin con bandas de gas que se curvan alrededor del vórtice](docs/bespin.png)

### Naboo

Combinamos ruido a varias escalas para separar mares y lagos de las llanuras, añadir grano a la tierra y aclarar las crestas en regiones altas. Una máscara de latitud añade hielo en los polos. El umbral de `land` cambia la proporción de agua y tierra. Los límites de `ice` controlan la extensión de los casquetes.

![Naboo con llanuras verdes, crestas claras y lagos azules](docs/naboo.png)

## Efectos solicitados para la evaluación

Contamos 60 puntos básicos y los siguientes 60 puntos de efectos, para un objetivo de 120 puntos de rúbrica. El mismo efecto se reclama una sola vez.

| Efecto | Puntos | Cuerpo y función en [shaders.rs](src/shaders.rs) | Cómo funciona y qué podemos cambiar | Evidencia |
| --- | ---: | --- | --- | --- |
| Material animado | 5 | Soles de Tatooine, `tatoo_i` y `tatoo_ii` | El tiempo cambia las coordenadas del ruido para que el plasma se desplace con ambas esferas fijas. `PLASMA_SPEED` regula la velocidad. | [Video de los soles](docs/soles.mp4) |
| Apariencia de día y noche | 5 | Scarif, `coastal_lights` | La máscara de luz habilita los asentamientos solo en las costas nocturnas. `NIGHT_LIGHT_STRENGTH` cambia su brillo. | [Barrido de luz](docs/scarif-luz.mp4). Fijarse en los puntos ámbar cerca del borde entre día y noche. |
| Respuestas diferentes a la iluminación | 10 | Scarif, `water_reflection` y `scarif` | La máscara de agua permite un brillo especular que se mueve con la luz. La tierra usa luz difusa. Subir `WATER_SPECULAR_POWER` hace más estrecho el reflejo. | [Barrido de luz](docs/scarif-luz.mp4). El brillo blanco se desplaza sobre el agua. |
| Nubes procedurales | 10 | Scarif, `cloud_mask` y `scarif` | El ruido de la máscara se evalúa en coordenadas que giran con el tiempo, mientras las islas se quedan fijas. `↑` y `↓` ajustan la cobertura. `CLOUD_SPEED` controla la velocidad. | [Video de nubes](docs/scarif-nubes.mp4). La dirección de la luz permanece fija. |
| Vórtice localizado | 10 | Bespin, `bespin_vortex` y `bespin` | Deformamos las coordenadas de las bandas con un giro que se debilita hacia el borde de la tormenta. `VORTEX_RADIUS` cambia el alcance del giro. | [Captura de Bespin](docs/bespin.png) |
| Sombras de nubes | 20 | Scarif, `cloud_shadow` y `scarif` | Desde la superficie seguimos la dirección de la luz hasta una altura virtual y evaluamos allí la misma máscara de nubes. Reducimos la luz directa según esa máscara. `CLOUD_HEIGHT` cambia cuánto se separa la sombra de la nube cuando la luz llega de lado. | [Comparación en video](docs/scarif-sombras.mp4). Izquierda con sombras, derecha con sombras desactivadas. Las nubes quedan fijas durante el barrido. |

Para repetir la comparación en la ventana, seleccionamos Scarif con `2`, pausamos con Espacio y alternamos `S` mientras cambiamos la dirección de la luz con las flechas. Así podemos ver cómo cambia la posición de las sombras con respecto a las nubes. Para verificar el resto de la animación, reanudamos el tiempo y dejamos quieta la luz.

Las capturas y los videos salen del mismo rasterizador que se usa en la ventana. Por ejemplo, podemos regenerar el video de los soles con FFmpeg instalado.

```sh
cargo run --release --locked -- --frames soles 144 | ffmpeg -f image2pipe -framerate 24 -c:v ppm -i pipe:0 -an -c:v libx264 -pix_fmt yuv420p soles.mp4
```
