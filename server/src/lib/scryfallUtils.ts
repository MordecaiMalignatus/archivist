// Basic interface for a Scryfall Card.
// Expand this with more properties as needed based on the Scryfall API documentation.
export interface ScryfallCard {
    id: string;
    name: string;
    oracle_id?: string;
    lang?: string;
    released_at?: string;
    uri?: string;
    scryfall_uri?: string;
    layout?: string;
    mana_cost?: string;
    cmc?: number;
    type_line?: string;
    oracle_text?: string;
    colors?: string[];
    color_identity?: string[];
    set?: string;
    set_name?: string;
    collector_number?: string;
    rarity?: string;
    image_uris?: { 'large': string, 'art_crop': string, 'border_crop': string, 'normal': string, 'small': string };
    // etc.
}

// Generic interface for a Scryfall API list object.
export interface ScryfallList<T> {
    object: 'list';
    total_cards?: number; // Not always present, e.g., for rulings
    has_more: boolean;
    next_page: string | null;
    data: T[];
    warnings?: string[];
}

/**
 * Fetches all items from a paginated Scryfall API endpoint.
 *
 * @param initialUrl The initial URL to fetch from (e.g., a search query or a set's cards endpoint).
 * @param fetchFn The fetch function to use (e.g., SvelteKit's fetch or the global fetch).
 * @returns A promise that resolves to an array containing all fetched items.
 */
export async function fetchAllScryfallItems<T>(
    initialUrl: string,
    fetchFn: typeof fetch
): Promise<T[]> {
    let allItems: T[] = [];
    let nextUrl: string | null = initialUrl;

    while (nextUrl) {
        try {
            const response = await fetchFn(nextUrl);
            if (!response.ok) {
                const errorData = await response.json().catch(() => ({})); // Try to parse error, otherwise empty object
                throw new Error(
                    `Scryfall API error: ${response.status} ${response.statusText}. ` +
                    `URL: ${nextUrl}. ` +
                    `Details: ${JSON.stringify(errorData)}`
                );
            }

            const listObject: ScryfallList<T> = await response.json();
            allItems = allItems.concat(listObject.data);
            nextUrl = listObject.has_more ? listObject.next_page : null;
        }
    }

    return allItems;
}

export async function fetchCardsForCodes(setCodes: string[], fetchFn: typeof fetch): Promise<ScryfallCard[]> {
    let query = setCodes.map(set => `s:${set}`).join(" or ");
    let queryFragment = encodeURIComponent(query); 
    let uri =  `https://api.scryfall.com/cards/search?q=${queryFragment}`;
    return fetchAllScryfallItems<ScryfallCard>(uri, fetch); 
}