import {appWindow, LogicalSize} from "@tauri-apps/api/window";

export default async function animateResize(
    width: number,
    fromHeight: number,
    toHeight: number,
    duration = 500
) {
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