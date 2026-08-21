import {Dispatch, SetStateAction} from "react";

type Reminder = {
    name: string,
    id: number,
    length: number,
}

export function newReminder(name: string, length: number, id: number) {
    return {name, length, id}
}

export default function addReminder(
    name: string,
    length: number,
    setReminders: Dispatch<SetStateAction<Reminder[]>>
) {
    setReminders(prev => [...prev, newReminder(
        name,
        length,
        setTimeout(() => {
            // nothing yet
        }, length))
    ])
}