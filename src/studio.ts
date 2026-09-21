import "./app.css";
import { mount } from "svelte";
import Studio from "./Studio.svelte";
document.addEventListener("contextmenu", (event) => event.preventDefault());
const target=document.getElementById("app"); if(!target) throw new Error("找不到挂载点 #app"); export default mount(Studio,{target});
