import { mount } from "svelte";
import "highlight.js/styles/github-dark.css";
import App from "./App.svelte";
import "./app.css";

const target = document.getElementById("app");
if (!target) throw new Error("missing #app mount point");

mount(App, { target });
