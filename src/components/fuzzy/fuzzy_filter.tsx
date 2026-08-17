export function fuzzyScore(text: string, query: string): number | null {
    text = text.toLowerCase();
    query = query.toLowerCase();

    if (!query) return 0;

    let qi = 0;
    let score = 0;
    let lastMatch = -1;

    for (let i = 0; i < text.length && qi < query.length; i++) {
        if (text[i] !== query[qi]) continue;

        // Bonus for beginning of a word.
        const wordStart =
            i === 0 ||
            text[i - 1] === " " ||
            text[i - 1] === "-" ||
            text[i - 1] === "_" ||
            text[i - 1] === ".";

        if (wordStart) score += 10;

        // Bonus for consecutive characters.
        if (lastMatch === i - 1) score += 5;

        // Bonus for matching at the beginning.
        if (i === 0) score += 20;

        score += 1;

        lastMatch = i;
        qi++;
    }

    // Didn't match the entire query.
    if (qi !== query.length) return null;

    // Shorter results are generally preferable.
    score -= text.length * 0.01;

    return score;
}

export function fuzzyFilter<T>(
    items: T[],
    query: string,
    getText = (item: any) => item.Name
): T[] {
    if (!query) return items;

    return items
        .map(item => ({
            item,
            score: fuzzyScore(getText(item), query)
        }))
        .filter(x => x.score !== null)
        .sort((a, b) => b.score! - a.score!)
        .map(x => x.item);
}