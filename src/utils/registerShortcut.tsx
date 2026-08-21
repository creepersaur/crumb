import {register, isRegistered, unregister} from "@tauri-apps/api/globalShortcut";
import {appWindow} from "@tauri-apps/api/window";

export default function registerShortcut() {
    const registerToggleShortcut = async () => {
        const already = await isRegistered("Alt+Space");
        if (already) return;

        await register("Alt+Space", async () => {
            const visible = await appWindow.isVisible();
            if (visible) {
                await appWindow.hide();
            } else {
                await appWindow.show();
                await appWindow.setFocus();
            }
        });
    };

    registerToggleShortcut().then();
    appWindow.onFocusChanged(({payload: focused}) => {
        if (!focused) {
            appWindow.hide().then();
        }
    }).then();

    if (import.meta.hot) {
        import.meta.hot.dispose(async () => {
            try {
                await unregister("Alt+Space");
            } catch {
            }
        });
    }
}