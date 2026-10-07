import { mount, unmount } from "svelte";
import Fixture from "./IslandFixture.svelte";
const fixture = mount(Fixture, { target: document.getElementById("app")! });
(window as any).unmountIsland = () => unmount(fixture);
