import type { PageLoad } from "./$types";

export const load: PageLoad = async ({fetch}) => {
    return {
        sets: await (await fetch("/static/setcodes.json")).json()
    };
}; 