# Guion del Video de Entrega (60–90 s)

## Ajustes de Grabación

- **Comando**: `cargo run --release`
- **Resolución de ventana**: `800×600` (resolución interna por defecto `400×300` para mantener > 30 FPS)
- **Tasa de cuadros de grabación**: `30 FPS` (sin VSync bloqueando por debajo o con OBS a `800×600 @ 30 FPS`)
- **Duración objetivo**: `80–90 s` (rango permitido: `60–90 s`)

## Lista de Verificación Previa

- [ ] Compilar y ejecutar en perfil release: `cargo run --release`
- [ ] Confirmar resolución interna `400×300` (el HUD superior indica `Imagen: 400x300`) y `F3` activado si se muestran tiempos por etapa
- [ ] Confirmar que `Gravedad [G]` y `Objetos descartados por no verse [X]` están en `SÍ` al iniciar vuelo
- [ ] Ocultar el panel de ayuda (`H`) después de mostrarlo brevemente para no tapar los planetas
- [ ] Tener listo OBS / grabador de pantalla capturando la ventana de `800×600`

## Tomas del Recorrido

| Tiempo | Duración | Toma | Teclas | Qué mostrar y narrar |
|---|---:|---|---|---|
| `0:00–0:08` | 8 s | **1. Menú principal y entrada al sistema realista** | `W/S`, `Enter` | Mostrar el menú sobre el sistema orbital animado y entrar a `Sistema solar realista`. |
| `0:08–0:20` | 12 s | **2. Vuelo espacial, culling y LOD** | `W`, `Shift`, `H` (2 s), `X`, `Z`, `M`, `F3` | Despegue frente al Sol con halo, abrir/cerrar `H`, alternar `X` (descartar fuera de vista), `M` (ver malla) y `Z` (nivel de detalle). |
| `0:20–0:36` | 16 s | **3. Acercamiento local y semilla en Zebes** | `W`, `Espacio`, `L`, `Q/E`, `I/O`, `Tab`, `4/5` | Acercarse a `Zebes` (`Vuelo: Cerca de Zebes`), rodear el relieve y objetos en superficie, cambiar semilla (`I/O`) y ajustar el parámetro en vivo (`4/5`). |
| `0:36–0:50` | 14 s | **4. Sobrevuelo de planetas procedurales** | `W`, `Shift`, `S` | Pasar sobre `Ardens`, `Abismo`, `Basquia`, `Vestigio` y `Mural` para mostrar variedad de shaders sin repetir técnicas. |
| `0:50–1:10` | 20 s | **5. Sistema de favoritos en vuelo** | `Esc`, `S`, `Enter`, `W`, `Shift`, `Tab` | Volver al menú con `Esc`, entrar a `Sistema de favoritos`; mostrar el `Sol de los Phoenix Suns` al centro y sobrevolar `La Bombonera`, `Casco de Samus`, `Raqueta de tenis`, `Cresta del Tottenham` y `Megaminx`. |
| `1:10–1:25` | 15 s | **6. Galería de trofeos** | `Esc`, `S`, `Enter`, `A/D`, `Mouse`, `4/5` | Volver al menú y abrir `Galería de trofeos`; recorrer varios trofeos con `A`/`D`, rotar con ratón y ajustar el parámetro en vivo (`4`/`5`). |

## Cómo grabar los 3 GIF de portada (`docs/gifs/`)

Para la tabla de tres columnas del `README.md`, graba clips cortos (`6–10 s`, `800×600`, `15–20 FPS`, `< 5–8 MB` cada uno) usando el mismo comando `cargo run --release` (o recortando las tomas del video con `ffmpeg`):

1. **`docs/gifs/sistema_realista.gif`**: vuelo corto frente al `Sol` y acercamiento a `Zebes` o vista orbital de los 9 cuerpos.
2. **`docs/gifs/sistema_favoritos.gif`**: vista orbital y sobrevuelo pasando cerca del `Sol de los Phoenix Suns`, `La Bombonera`, `Casco de Samus` y `Raqueta de tenis`.
3. **`docs/gifs/galeria.gif`**: recorrido en la `Galería de trofeos` cambiando entre 3 o 4 cuerpos con `A`/`D` y rotando con el ratón.

Ejemplo de conversión desde un clip MP4/MKV a GIF optimizado con `ffmpeg`:

```bash
ffmpeg -ss 00:00:52 -t 8 -i grabacion.mp4 \
  -vf "fps=15,scale=480:-1:flags=lanczos,split[s0][s1];[s0]palettegen[p];[s1][p]paletteuse" \
  docs/gifs/sistema_favoritos.gif
```
