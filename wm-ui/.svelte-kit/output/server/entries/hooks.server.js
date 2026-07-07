import { n as startWm, r as stopWm } from "../chunks/wm-bridge.js";
//#region src/hooks.server.ts
startWm().then(() => console.log("WM engine started")).catch((e) => console.error("WM engine failed:", e));
process.on("exit", () => stopWm());
//#endregion
export {};
