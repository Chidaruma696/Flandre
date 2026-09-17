[English](README.md) · [Español](README.es.md)

# Flandre 〜 フランドール

**Los colores del fondo de pantalla en todo el escritorio GNOME, con un solo binario.** GNOME
Shell, apps libadwaita y GTK 3, el tema de iconos, Ptyxis, GNOME Console, Black Box y cualquier
terminal que ya esté abierta: todo se regenera del fondo cada vez que cambia (o cambia la
preferencia claro/oscuro).

![Rust](https://img.shields.io/badge/Rust-2024-orange) ![GNOME](https://img.shields.io/badge/GNOME-47%E2%80%9350-blue) ![Licencia](https://img.shields.io/badge/licencia-MIT-green)

## Por qué

La forma habitual de tener Material You en GNOME es una extensión con un backend en Python que
se clona su propio venv, una copia en User Themes de una hoja de estilo compilada para una versión
concreta de la Shell, un script de Nyarch que reinstala un tema de iconos de 118 MB a golpe de
`sed`, y pywal escribiendo códigos de escape en cada pty. Funciona hasta que deja de hacerlo.
Flandre es esa misma tubería en un solo programa en Rust, sin más dependencia en ejecución que
GLib, y llega más lejos:

| Destino | Cómo |
|---|---|
| **libadwaita (GTK 4)** y **adw-gtk3 (GTK 3)** | Todos los colores con nombre que expone libadwaita (acento, superficies, headerbar, sidebars, cards, diálogos, popovers, las filas `blue_1`…`dark_5`) en `~/.config/gtk-{3,4}.0/gtk.css`, más unas reglas que empujan el tinte a selecciones, pestañas, switches y OSD. GTK 4 recibe los dos modos como variables CSS bajo `@media (prefers-color-scheme)`, así que las apps abiertas siguen el cambio claro/oscuro sin reiniciarse. |
| **GNOME Shell** | Se lee la hoja de estilo del gnome-shell *instalado* desde su GResource y se reescribe cada color (los grises toman el matiz del fondo, los saturados se armonizan, las palabras clave del acento pasan a ser el primario). Se carga con User Themes, así que nunca se queda atrás de una actualización de la Shell. |
| **Iconos** | Un tema `Flandre-<familia>-<hex>` encima de Tela o Papirus: se recolorea cada SVG que lleve el color de carpeta de la familia (todos los tamaños, nombres por symlink incluidos) con el color del esquema que elijas (primary container por defecto, como Material You) y el resto se hereda. Menos de un segundo. |
| **Ptyxis** | Una `.palette` nativa (claro + oscuro, con titlebar) seleccionada en todos los perfiles. Paquete del sistema y Flatpak. |
| **GNOME Console** | Una *livery* propia escrita en `org.gnome.Console custom-liveries` y seleccionada (Console 49+). |
| **Black Box** | Esquemas JSON `Flandre Dark` / `Flandre Light` más `theme-dark` / `theme-light` / `pretty`. Paquete del sistema y Flatpak. |
| **Colores de terminal** | `Flandre` (por defecto) saca los 16 colores ANSI del propio esquema: azul = primary, magenta = tertiary, rojo = error, verde, amarillo y cian armonizados con el fondo. O elige una base clásica (GNOME, Tango, Solarized, Monokai, Gruvbox, Dracula, Nord, Catppuccin, Tokyo Night, Everforest, Rosé Pine, Ayu, Kanagawa) y un deslizador de *fusión* decide cuánto se acerca al fondo. Variantes clara y oscura; sintetizada para los esquemas que solo tienen oscura. |
| **Opacidad de terminal** | Un solo control para Ptyxis (`opacity` en cada perfil), Console (transparencia de la livery) y Black Box (`opacity`). |
| **Terminales abiertas** | Secuencias OSC 4/10/11/12 a todos los pty, cacheadas en `~/.cache/flandre/sequences` para el rc de la shell, con `colors.json` / `colors.sh` para tus scripts. |
| **Flatpaks** | `flatpak override --user` para que las apps en sandbox lean el CSS de GTK, los iconos y los temas. |

Tres niveles de tinte (`soft`, `normal`, `strong`) controlan cuánto entra el matiz del fondo en
los grises y cuánto acento baña headerbars, sidebars y popovers. `strong` es el predeterminado y
está pensado para ser agresivo; mantiene oscuras las superficies oscuras porque tiñe el matiz a
tono constante, y el control `darken` lleva los tonos oscuros desde los de Material hasta casi negro.

## Ventana de ajustes

`flandre settings` (también "Flandre" en la parrilla de apps tras el `setup`) es una ventana
libadwaita con previsualización en vivo: un escritorio de mentira (barra superior, headerbar,
sidebar, card, popover, botones), la paleta, las carpetas recoloreadas y los colores de terminal.
Edita esquema, tinte, oscuridad, familia de iconos (Tela / Papirus), qué color del esquema pinta
los iconos (o un tono a medida), el estilo de la barra superior (negra, de color, transparente) y
los destinos, y aplica con un botón.

## Instalación

```sh
git clone https://github.com/Chidaruma696/Flandre
cd Flandre
cargo build --release
./target/release/flandre setup
```

`setup` copia el binario a `~/.local/bin/flandre`, instala y arranca un servicio de usuario de
systemd (`flandre watch`), activa User Themes, desactiva la extensión Material You Colors si está
(se pelearían por `gtk.css`), añade los overrides de Flatpak, agrega a `~/.bashrc` (y `~/.zshrc`
/ fish si existen) un fragmento que pinta las terminales nuevas, escribe una configuración por
defecto y aplica el tema una vez. `--no-rc` y `--no-flatpak` saltan esos dos pasos.

Requisitos: GNOME 47 o más nuevo, el paquete `gnome-shell-extensions` (User Themes), `adw-gtk3`
y Tela o Papirus (`tela-icon-theme` / `papirus-icon-theme` en Arch). Para compilar hacen falta
Rust y las cabeceras de GTK 4 y libadwaita (`gtk4`, `libadwaita` en Arch; `libgtk-4-dev`,
`libadwaita-1-dev` en Debian).

## Uso

```
flandre apply            # regenera desde el fondo actual (no hace nada si nada cambió)
flandre apply --force    # regenera igualmente
flandre apply --wallpaper ~/Imágenes/x.jpg --variant vibrant --tint normal --light
flandre apply --color '#6750a4'
flandre watch            # lo que ejecuta el servicio
flandre doctor           # comprueba cada pieza
flandre colors [--json]  # imprime la paleta
flandre sequences        # imprime los códigos de escape
flandre settings         # la ventana de previsualización
```

La configuración vive en `~/.config/flandre/config.toml`; mira [`config.example.toml`](config.example.toml).

## Lo que no hace

libadwaita solo deja sobreescribir sus colores con nombre; una app con colores fijos en su propio
CSS se queda como está. Los iconos que no sean carpetas, lugares, dispositivos y las acciones de
color conservan los colores de Tela. GNOME Console anterior a 49 no tiene API de paletas, así que
solo se pinta con las secuencias de escape (y el fragmento del rc para pestañas nuevas).

## Créditos

Ver [CREDITS.md](CREDITS.md). La tubería la montaron primero Material You Colors,
adwaita-material-you y NyarchLinux; Flandre la reescribe en vez de copiarla.

## Licencia

MIT. Touhou Project y sus personajes pertenecen a Team Shanghai Alice (ZUN); obra de fans no
oficial hecha según sus directrices para obras derivadas, sin afiliación ni respaldo.
