import { Elysia } from "elysia";

export const makeBackend = (prefix: string) => {
  return new Elysia({prefix})
    .get("/", "hello there")
}

makeBackend("").listen(3000);