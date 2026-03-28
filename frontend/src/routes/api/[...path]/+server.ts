import type { RequestHandler } from './$types';
import { BACKEND_URL } from '$env/static/private';

const handler: RequestHandler = async ({ request, params }) => {
  const url = `${BACKEND_URL}/api/${params.path}`;

  const headers = new Headers(request.headers);
  // Remove host header to avoid conflicts
  headers.delete('host');

  const response = await fetch(url, {
    method: request.method,
    headers,
    body: request.method !== 'GET' && request.method !== 'HEAD' ? request.body : undefined,
    // @ts-expect-error - duplex is required for streaming bodies in Node 18+
    duplex: 'half',
  });

  return new Response(response.body, {
    status: response.status,
    statusText: response.statusText,
    headers: response.headers,
  });
};

export const GET = handler;
export const POST = handler;
export const PUT = handler;
export const DELETE = handler;
export const PATCH = handler;
