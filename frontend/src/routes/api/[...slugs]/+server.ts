import { Elysia, t } from 'elysia';
import { makeBackend } from 'archivist-backend'; 

const app = makeBackend('/api'); 

type RequestHandler = (v: { request: Request }) => Response | Promise<Response>

export const GET: RequestHandler = ({ request }) => app.handle(request)
export const POST: RequestHandler = ({ request }) => app.handle(request)