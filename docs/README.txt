rtvc full web emulator

This directory contains the full browser version of rtvc, built with:

  cargo xtask bundle-web-full docs

It opens the normal emulator UI in a browser and can load local CAS, DSK, ZIP,
and snapshot files through the app menus. Dragging a CAS, DSK, or ZIP onto the
page boots TVC 1.2 VT-DOS from the clean snapshot and runs it.

Example:

  python -m http.server 8000

Then open:

  http://localhost:8000/
