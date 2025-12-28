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
        }),
        "build:ubuntu": cmd({
            help: "Build for Ubuntu (AppImage + .deb) using Docker",
            exec: async () => {
                // Build Docker image
                await liveExec(
                    "docker build -f Dockerfile.ubuntu-build -t tauri-ubuntu-builder .",
                    { outputPrefix: "DOCKER-BUILD" }
                );

                // Build the app
                await liveExec(
                    `docker run --rm \
                -v $(pwd):/app:z \
                -w /app \
                tauri-ubuntu-builder \
                sh -c 'export PATH=/root/.cargo/bin:/root/.bun/bin:\$PATH && bun install && bun run tauri build'`,
                    { outputPrefix: "BUILD" }
                );
            },
        }),
    }
})