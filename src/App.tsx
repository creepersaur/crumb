import {register, isRegistered, unregister} from "@tauri-apps/api/globalShortcut";
import {appWindow, LogicalSize} from "@tauri-apps/api/window";
import Window from "./components/window/window.tsx";
import {invoke} from "@tauri-apps/api";
import * as Lucide from "lucide-react";
import React from "react";
import "./App.css";

const animateResize = async (
    width: number,
    fromHeight: number,
    toHeight: number,
    duration = 500
) => {
    if (Math.abs(fromHeight - toHeight) < 0.5) return;

    const start = performance.now();
    const ease = (t: number) => 1 - Math.pow(1 - t, 3);

    return new Promise<void>((resolve) => {
        const step = async (now: number) => {
            const t = Math.min((now - start) / duration, 1);
            const eased = ease(t);
            const height = fromHeight + (toHeight - fromHeight) * eased;

            try {
                await appWindow.setSize(new LogicalSize(width, height + 10));
            } catch {
                resolve();
                return;
            }

            if (t < 1) {
                requestAnimationFrame(step);
            } else {
                resolve();
            }
        };
        requestAnimationFrame(step);
    });
};

type InstalledApp = {
    name: string;
    icon_data: string | null;
    path: string;
};

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

export default function App() {
    const windowRef = React.useRef<HTMLDivElement>(null);
    const fixedWidthRef = React.useRef<number>(0);
    const isAnimatingRef = React.useRef(false);
    const pendingHeightRef = React.useRef<number | null>(null);

    const [selected, setSelected] = React.useState(0);
    const [appsList, setAppsList] = React.useState<any[]>([]);
    React.useEffect(() => {
        let cancelled = false;

        async function loadApps() {
            const apps = await invoke<InstalledApp[]>("get_apps");

            if (cancelled) return;

            setAppsList(apps.map(app => {
                console.log(app.name);

                return {
                    Name: app.name,
                    Icon: app.icon_data,
                    Action: async () => {
                        console.log(`Opening app: ${app.path}`);
                        await invoke("open_app", {path: app.path});
                    }
                }
            }));
        }

        loadApps().then();

        return () => {
            cancelled = true;
        };
    }, []);

    const resizeToHeight = React.useCallback(async (targetHeight: number) => {
        if (isAnimatingRef.current) {
            pendingHeightRef.current = targetHeight;
            return;
        }

        isAnimatingRef.current = true;

        try {
            let current = await appWindow.innerSize();
            const scaleFactor = await appWindow.scaleFactor();
            let fromHeight = current.height / scaleFactor;

            await animateResize(fixedWidthRef.current, fromHeight, targetHeight, 100);

            while (pendingHeightRef.current !== null) {
                const next = pendingHeightRef.current;
                pendingHeightRef.current = null;

                current = await appWindow.innerSize();
                fromHeight = current.height / scaleFactor;

                await animateResize(fixedWidthRef.current, fromHeight, next, 100);
            }
        } finally {
            isAnimatingRef.current = false;
        }
    }, []);

    React.useEffect(() => {
        const el = windowRef.current;
        if (!el) return;

        const rect = el.getBoundingClientRect();
        fixedWidthRef.current = Math.ceil(rect.width);

        appWindow.setSize(new LogicalSize(Math.ceil(rect.width), Math.ceil(rect.height))).then();

        const observer = new ResizeObserver(() => {
            const height = el.getBoundingClientRect().height;
            resizeToHeight(Math.ceil(height)).then();
        });

        observer.observe(el);

        return () => observer.disconnect();
    }, [resizeToHeight]);

    return <>
        <div className="background"/>
        <Window ref={windowRef as any} selected={selected} setSelected={setSelected} buttons={[
            {
                Name: "Apps",
                Icon: Lucide.Grip,
                Submenu: appsList,
            },
            {
                Name: "Trigger",
                Icon: Lucide.Rocket,
                Submenu: true,
            },
            {
                Name: "Style",
                Icon: Lucide.WandSparkles,
                Submenu: true,
            },
            {
                Name: "Install",
                Icon: Lucide.Save,
                Submenu: true,
            },
            {
                Name: "Remove",
                Icon: Lucide.Trash2,
                Submenu: true,
            },
            {
                Name: "Update",
                Icon: Lucide.RefreshCw,
                Submenu: true,
            },
            {
                Name: "About",
                Icon: Lucide.Info,
            },
            {
                Name: "System",
                Icon: Lucide.Power,
                Submenu: [
                    {
                        Name: "Settings",
                        Icon: Lucide.Settings,
                        Submenu: [
                            {Name: "Hello"},
                            {Name: "World"},
                        ],
                    },
                    {
                        Name: "Sleep",
                        Icon: Lucide.BedDouble,
                    },
                    {
                        Name: "Restart",
                        Icon: Lucide.RefreshCw,
                    },
                    {
                        Name: "Turn Off",
                        Icon: Lucide.Power,
                    },
                ],
            },
        ]}/>
    </>;
}