import React from "react";
import {invoke} from "@tauri-apps/api";

type InstalledApp = {
    name: string;
    icon_data: string | null;
    path: string;
};

export default function LoadApps() {
    const appsListDispatch = React.useState<any[] | string>("loading apps...\n(try again in a few seconds)");
    const setAppsList = appsListDispatch[1];

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

    return appsListDispatch;
}