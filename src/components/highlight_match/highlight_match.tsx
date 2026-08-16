export default function HighlightMatch({text, query}: { text: string; query: string }) {
    if (!query) return <>{text}</>;

    const lowerText = text.toLowerCase();
    const lowerQuery = query.toLowerCase();

    if (!lowerText.includes(lowerQuery)) return <>{text}</>;

    const index = lowerText.indexOf(lowerQuery);
    const start = text.slice(0, index);
    const matched = text.slice(index, index + query.length);
    const rest = text.slice(index + query.length);

    return <>{start}<u>{matched}</u>{rest}</>;
}