import "./winbutton.css"
import React from "react";
import {ChevronRight} from "lucide-react";

export default function WinButton(props: React.PropsWithChildren & {
    id: number,
    selected: boolean,
    submenu?: string | null,
    onClick?: React.MouseEventHandler<HTMLButtonElement>
    onMouseEnter?: React.MouseEventHandler<HTMLButtonElement>
    onKeyDown?: React.KeyboardEventHandler<HTMLButtonElement>
}) {
    const buttonRef = React.useRef<HTMLButtonElement>(null);

    if (props.selected) {
        buttonRef.current?.scrollIntoView({
            behavior: "smooth",
            block: "nearest"
        });
    }

    return <button ref={buttonRef} key={`button_${props.id}`} className={`win-button${props.selected ? " selected" : ""}`} onClick={props.onClick}
                   onMouseEnter={props.onMouseEnter}>
        {props.id <= 9 && <div className="btn-index">{props.id}</div>}
        {props.children}
        {props.submenu && <div className="btn-arrow">
            <ChevronRight/>
        </div>}
    </button>
}