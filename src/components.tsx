// Standalone entry for the component gallery. Kept separate from the desk app
// (`main.tsx`) so the gallery never ships inside the real UI. Open
// `/components.html` in the dev server, or build and open `dist/components.html`.

import React from "react";
import ReactDOM from "react-dom/client";
import { Gallery } from "./pages/Gallery";
import "./index.css";
import "@fontsource-variable/inter/wght.css";
import "@fontsource-variable/geist-mono/wght.css";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <Gallery />
  </React.StrictMode>,
);
