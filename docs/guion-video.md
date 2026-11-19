# Guion del Video de Entrega (60–90 s)

## Ajustes de Grabación

- **Comando**: `cargo run --release`
- **Resolución de ventana**: `800×600` (resolución interna por defecto `400×300` para mantener > 30 FPS)
- **Tasa de cuadros de grabación**: `30 FPS` (sin VSync bloqueando por debajo o con OBS a `800×600 @ 30 FPS`)
- **Duración objetivo**: `75 s` (rango permitido: `60–90 s`)

## Lista de Verificación Previa

- [ ] Compilar y ejecutar en perfil release: `cargo run --release`
- [ ] Confirmar resolución interna `400×300` (HUD superior indica `400x300`) y `F3` activado si se muestran tiempos por etapa
- [ ] Confirmar que `G` (gravedad) y `X` (frustum culling) están en `ON` al iniciar
- [ ] Ocultar el panel de ayuda (`H`) después de mostrarlo brevemente o al final para no tapar los planetas
- [ ] Tener listo OBS / grabador de pantalla capturando la ventana de `800×600`

## Tomas del Recorrido

| Tiempo | Duración | Toma | Teclas | Qué mostrar y narrar |
|---|---:|---|---|---|
| `0:00–0:14` | 14 s | **1. Vuelo espacial y vista del sistema** | `W`, `Shift`, `Mouse`, `H` (2 s) | Despegue frente al Sol (`Star` con halo) y vista general del sistema orbital; abrir y cerrar `H` para mostrar el HUD propio. |
| `0:14–0:30` | 16 s | **2. Culling y LOD en vivo** | `X`, `Z`, `M`, `F3` | Mirar hacia el sistema y luego al vacío con `X` (culling ON/OFF mostrando `tris_before -> tris_after` llegando a `0`); pulsar `M` (wireframe) y `Z` para mostrar el cambio de nivel LOD. |
| `0:30–0:48` | 18 s | **3. Acercamiento y vuelta completa en modo local** | `W`, `Shift`, `Espacio`, `L`, `Q / E`, `Flechas` | Acercarse a `Zebes` hasta entrar en `Local(Zebes)` (altitud decreciente y gravedad activa), dar una vuelta completa rodeando el relieve y las instancias en superficie. |
| `0:48–1:02` | 14 s | **4. Semilla y edición en vivo** | `I`, `O`, `Tab`, `9 / 0`, `4 / 5` | Sobre `Zebes`, pulsar `I` / `O` para regenerar la distribución de instancias con nuevas semillas; luego `Tab` + `4 / 5` + `9 / 0` para ajustar parámetro de shader y radio en vivo. |
| `1:02–1:18` | 16 s | **5. Sobrevuelo de planetas nuevos y alejamiento** | `W`, `Shift`, `S` | Alejarse de `Zebes` y sobrevolar `Ardens` (grietas Voronoi), `Abismo` (océano y maelstrom), `Basquia` (oilstick 6 FPS), `Vestigio` (VHS) y `Mural` (pintura descascarada), cerrando con plano general. |
