import React from "react";

export default function HighlightMatch({text, query}: {
    text: string;
    query: string;
}) {
    if (!query) return <>{text}</>;

    const lowerText = text.toLowerCase();
    const lowerQuery = query.toLowerCase();

    const indices: number[] = [];
    let qi = 0;

    for (let i = 0; i < lowerText.length && qi < lowerQuery.length; i++) {
        if (lowerText[i] === lowerQuery[qi]) {
            indices.push(i);
            qi++;
        }
    }

    // Query wasn't fully matched.
    if (qi !== lowerQuery.length) {
        return <>{text}</>;
    }

    const parts: React.ReactNode[] = [];
    let last = 0;

    for (const index of indices) {
        if (index > last) {
            parts.push(text.slice(last, index));
        }

        parts.push(<u key={index}>{text[index]}</u>);
        last = index + 1;
    }

    if (last < text.length) {
        parts.push(text.slice(last));
    }

    return <>{parts}</>;
}