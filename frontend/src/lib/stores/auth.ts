import { writable } from 'svelte/store';

export interface AuthUser {
  id: string;
  username: string;
  email?: string;
  two_factor_enabled: boolean;
  force_password_change: boolean;
}

export interface AuthState {
  user: AuthUser | null;
  token: string | null;
  isAuthenticated: boolean;
  isLoading: boolean;
}

function createAuthStore() {
  const { subscribe, set, update } = writable<AuthState>({
    user: null,
    token: null,
    isAuthenticated: false,
    isLoading: true,
  });

  return {
    subscribe,
    init: (user: AuthUser | null, token: string | null) => {
      set({
        user,
        token,
        isAuthenticated: !!user && !!token,
        isLoading: false,
      });
    },
    login: (user: AuthUser, token: string) => {
      set({
        user,
        token,
        isAuthenticated: true,
        isLoading: false,
      });
    },
    logout: () => {
      set({
        user: null,
        token: null,
        isAuthenticated: false,
        isLoading: false,
      });
    },
    setLoading: (loading: boolean) => {
      update((state) => ({ ...state, isLoading: loading }));
    },
  };
}

export const authStore = createAuthStore();
