import Window from "./components/window/window.tsx";
import {invoke} from "@tauri-apps/api";
import * as Lucide from "lucide-react";
import React from "react";
import "./App.css";
import registerShortcut from "./utils/registerShortcut.tsx";
import resizeWindow from "./utils/resizeWindow.tsx";
import LoadApps from "./components/appLoader.tsx";

registerShortcut();

export default function App() {
    const windowRef = React.useRef<HTMLDivElement>(null);
    const [selected, setSelected] = React.useState(0);
    const [appsList, _] = LoadApps();

    resizeWindow(windowRef);

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
                        Action: () => {
                            invoke("power", {action: "sleep"}).then()
                        }
                    },
                    {
                        Name: "Restart",
                        Icon: Lucide.RefreshCw,
                        Action: () => {
                            invoke("power", {action: "restart"}).then()
                        }
                    },
                    {
                        Name: "Turn Off",
                        Icon: Lucide.Power,
                        Action: () => {
                            invoke("power", {action: "shutdown"}).then()
                        }
                    },
                ],
            },
        ]}/>
    </>;
}