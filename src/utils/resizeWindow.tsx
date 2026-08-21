import React from "react";
import {appWindow, LogicalSize} from "@tauri-apps/api/window";
import animateResize from "./animateResize.tsx";

export default function resizeWindow(windowRef: any) {
    const fixedWidthRef = React.useRef<number>(0);
    const isAnimatingRef = React.useRef(false);
    const pendingHeightRef = React.useRef<number | null>(null);

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
}