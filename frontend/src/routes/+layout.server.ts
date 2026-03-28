import type { LayoutServerLoad } from './$types';
import jwt from 'jsonwebtoken';
import { JWT_SECRET } from '$env/static/private';

export const load: LayoutServerLoad = async ({ cookies }) => {
  const token = cookies.get('auth_token');

  if (!token) {
    return { user: null, token: null };
  }

  try {
    const decoded = jwt.verify(token, JWT_SECRET) as {
      sub: string;
      user_id: string;
      username: string;
    };

    return {
      user: {
        id: decoded.user_id,
        username: decoded.username,
      },
      token,
    };
  } catch {
    // Token invalid or expired
    cookies.delete('auth_token', { path: '/' });
    return { user: null, token: null };
  }
};
