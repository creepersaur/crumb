import "./window.css"
import React from "react";
import WinButton from "../window_button/winbutton.tsx";
import {appWindow} from "@tauri-apps/api/window";
import HighlightMatch from "../highlight_match/highlight_match.tsx";
import {fuzzyFilter} from "../fuzzy/fuzzy_filter.tsx";

type Buttons = {
    Name: string,
    Icon?: any,
    Submenu?: boolean | Buttons | string,
    Action?: (Submenu: any, setScreen: any) => void,
}[];
type HistoryItem = {
    name: string,
    buttons: string | Buttons
}
type WindowProps = {
    children?: React.ReactNode;
    buttons?: Buttons,
    selected: number,
    setSelected: React.Dispatch<React.SetStateAction<number>>,
};

const Window = React.forwardRef<HTMLDivElement, WindowProps>(
    ({children, buttons, selected, setSelected}, ref) => {
        const searchRef = React.useRef<HTMLInputElement>(null);

        const [screen, setScreen] = React.useState<Buttons | string>(buttons ?? []);
        const [history, setHistory] = React.useState<HistoryItem[]>([{name: "Home", buttons: buttons!}]);
        const [filter, setFilter] = React.useState<string>("");

        React.useEffect(() => {
            if (!buttons) return;

            setScreen(prev => {
                // We're on the root screen
                if (history.length === 1) {
                    return buttons;
                }

                // Find the current history entry and update its buttons
                const current = history[history.length - 1];

                if (current.buttons instanceof Array) {
                    return current.buttons;
                }

                return prev;
            });

            setHistory(prev => {
                const next = [...prev];
                next[0] = {name: "Home", buttons};
                return next;
            });
        }, [buttons]);

        React.useEffect(() => {
            const handleKeyDown = (e: KeyboardEvent) => {
                if (e.key == "Escape") {
                    if (history.length < 2) {
                        appWindow.hide().then();
                        return;
                    }

                    searchRef.current!.value = "";
                    setFilter("");
                    setHistory(prev => {
                        const previous = prev[prev.length - 2];
                        setScreen(previous.buttons);
                        setSelected(0);

                        return prev.slice(0, -1);
                    });

                    return;
                }

                if (e.key >= "0" && e.key <= "9" && e.ctrlKey) {
                    e.preventDefault();

                    const index = Number(e.key);
                    if (typeof screen == "string") return;
                    const filtered = fuzzyFilter(screen, filter);
                    const btn = filtered[index];

                    searchRef.current!.value = "";
                    setFilter("");

                    setSelected(index);
                    if (btn.Action) btn.Action(btn.Submenu, setScreen);

                    if (btn.Submenu) {
                        setSelected(0);
                        const next = btn.Submenu === true ? {
                            name: btn.Name,
                            buttons: btn.Name
                        } : {
                            name: btn.Name,
                            buttons: btn.Submenu
                        };
                        setScreen(btn.Submenu === true ? btn.Name : btn.Submenu);
                        setHistory(prev => [...prev, next]);
                    }

                    return;
                }

                if (typeof screen != "string") {
                    const length = fuzzyFilter(screen, filter).length;
                    if (e.key == "ArrowDown" || e.key == "Tab") {
                        setSelected(prev => ++prev % length);
                        return;
                    } else if (e.key == "ArrowUp") {
                        setSelected(prev => (--prev + length) % length);
                        return;
                    }
                }

                if (e.key == "Enter") {
                    e.preventDefault();
                    if (typeof screen == "string") return;
                    const filtered = fuzzyFilter(screen, filter);
                    const btn = filtered[selected];

                    searchRef.current!.value = "";
                    setFilter("");

                    if (btn.Action) btn.Action(btn.Submenu, setScreen);

                    if (btn.Submenu) {
                        setSelected(0);
                        const next = btn.Submenu === true ? {
                            name: btn.Name,
                            buttons: btn.Name
                        } : {
                            name: btn.Name,
                            buttons: btn.Submenu
                        };
                        setScreen(btn.Submenu === true ? btn.Name : btn.Submenu);
                        setHistory(prev => [...prev, next]);
                    }
                }

                const isPrintableKey = (e.key.length === 1 || e.key == "Backspace") && !e.ctrlKey && !e.metaKey && !e.altKey;

                if (isPrintableKey && document.activeElement !== searchRef.current) {
                    searchRef.current?.focus();
                    return;
                }
            };

            window.addEventListener("keydown", handleKeyDown);

            return () => {
                window.removeEventListener("keydown", handleKeyDown);
            };
        }, [history, selected, filter]);

        return (
            <div className="window" ref={ref}>
                <input ref={searchRef} className="search-bar" placeholder="Go..." onChange={() => {
                    setFilter(searchRef.current!.value);
                    setSelected(0);
                }}/>

                <div className="path">{history.map((item, i) => {
                    const is_not_last = i < history.length - 1;

                    return <React.Fragment key={`path_${i}`}>
                        <p className={!is_not_last ? "last" : ""} onClick={() => setHistory(prev => {
                            const sliced = prev.slice(0, i + 1);
                            setScreen(sliced[sliced.length - 1].buttons)
                            return sliced;
                        })}>{item.name}</p>
                        {is_not_last && <p className="btn-arrow">{">"}</p>}
                    </React.Fragment>
                })}</div>

                <div className="window-buttons">
                    {typeof screen == "string" &&
                        <div className="screen-text" style={{whiteSpace: "pre-line"}}>{screen}</div>}
                    {(() => {
                            if (typeof screen == "string") return;

                            let items = fuzzyFilter(screen, filter);

                            if (!items.length) return <div className="screen-text" style={{whiteSpace: "pre-line"}}>{"<No Items>"}</div>

                            return typeof screen != "string" && items
                                .map(({Name, Icon, Submenu, Action}, i) => {
                                    return <WinButton
                                        key={`path_${i}`}
                                        id={i}
                                        selected={selected === i}
                                        submenu={Submenu ? Name : null}
                                        onClick={() => {
                                            if (Submenu) {
                                                setScreen(Submenu == true ? Name : Submenu);
                                                setHistory(prev => [...prev, Submenu == true ? {
                                                    name: Name,
                                                    buttons: Name
                                                } : {
                                                    name: Name,
                                                    buttons: Submenu
                                                }]);
                                            }

                                            searchRef.current!.value = "";
                                            setFilter("");
                                            console.log("click")
                                            if (Action) Action(Submenu, setScreen);
                                        }}
                                        onMouseEnter={() => setSelected(i)}
                                    >
                                        {Icon && (typeof Icon == "string" ? <img src={Icon} alt="icon"/> :
                                            <Icon/>)}<HighlightMatch
                                        text={Name} query={filter}/>
                                    </WinButton>
                                })
                        }
                    )()}
                </div>

                {
                    children
                }
            </div>
        )
            ;
    }
);

export default Window;