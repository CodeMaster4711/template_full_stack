import { authStore } from '$lib/stores/auth';
import { get } from 'svelte/store';
import { goto } from '$app/navigation';

const API_BASE = '/api';

export class ApiClient {
  private static getToken(): string | null {
    return get(authStore).token;
  }

  private static async handleUnauthorized() {
    authStore.logout();
    // Clear the auth cookie
    await fetch('/api/set-auth-cookie', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ token: null }),
    });
    goto('/signin');
  }

  static async fetch(path: string, options: RequestInit = {}): Promise<Response> {
    const token = this.getToken();
    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
      ...(options.headers as Record<string, string>),
    };

    if (token) {
      headers['Authorization'] = `Bearer ${token}`;
    }

    const response = await fetch(`${API_BASE}${path}`, {
      ...options,
      headers,
    });

    if (response.status === 401) {
      await this.handleUnauthorized();
    }

    return response;
  }

  static get(path: string): Promise<Response> {
    return this.fetch(path, { method: 'GET' });
  }

  static post(path: string, body?: unknown): Promise<Response> {
    return this.fetch(path, {
      method: 'POST',
      body: body !== undefined ? JSON.stringify(body) : undefined,
    });
  }

  static put(path: string, body?: unknown): Promise<Response> {
    return this.fetch(path, {
      method: 'PUT',
      body: body !== undefined ? JSON.stringify(body) : undefined,
    });
  }

  static delete(path: string): Promise<Response> {
    return this.fetch(path, { method: 'DELETE' });
  }
}
