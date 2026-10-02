import React from "react";
import ReactDOM from "react-dom/client";

import App from "./App";
import "./styles/global.css";

// Purpose: Boots the React application inside the Vite entry document.
ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);

