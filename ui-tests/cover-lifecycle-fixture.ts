import { mount, unmount } from "svelte";
import Fixture from "./CoverLifecycleFixture.svelte";
const fixture = mount(Fixture, {target: document.getElementById("app")!});
(window as any).destroyCover = () => unmount(fixture);
