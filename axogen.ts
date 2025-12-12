import {cmd, defineConfig, liveExec} from "@axonotes/axogen";

export default defineConfig({
    commands: {
        install: cmd({
            help: "Install xpatch demo dependencies",
            command: "bun install",
        }),
        dev: cmd({
            help: "Start the xpatch demo editor",
            exec: async () => {
                const isLinux = process.platform === "linux";
                const command = `${isLinux ? "__NV_DISABLE_EXPLICIT_SYNC=1 " : ""}bun run tauri dev`;
                await liveExec(command, {
                    outputPrefix: "DEV",
                });
            },
        })
    }
})