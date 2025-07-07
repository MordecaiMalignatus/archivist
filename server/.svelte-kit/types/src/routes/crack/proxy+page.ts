// @ts-nocheck
import type { PageLoad } from "./$types";

const boosterToSetcoodes = [
    {name: "Bloomburrow Play Booster", sets: ["BLB", "SPG"] },
    {name: "Duskmourn Play Booster", sets: ["DSK", "SPG"] }, 
]

export const load = async ({fetch}: Parameters<PageLoad>[0]) => {
    return {
        sets: await (await fetch("/static/setcodes.json")).json(),
        boosters: boosterToSetcoodes, 
    };
}; 

