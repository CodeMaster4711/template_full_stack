import forge from 'node-forge';

const API_BASE = '/api';

export interface LoginResponse {
  token: string;
  user_id: string;
  username: string;
  two_factor_enabled: boolean;
  force_password_change: boolean;
}

export interface NormalizedLoginResponse {
  token: string;
  user: {
    id: string;
    username: string;
    email?: string;
    two_factor_enabled: boolean;
    force_password_change: boolean;
  };
}

export class AuthService {
  static async getPublicKey(): Promise<string> {
    const response = await fetch(`${API_BASE}/public-key`);
    if (!response.ok) {
      throw new Error('Failed to fetch public key');
    }
    const data = await response.json();
    return data.public_key;
  }

  static async encryptPassword(password: string, publicKeyPem: string): Promise<string> {
    const publicKey = forge.pki.publicKeyFromPem(publicKeyPem);
    const encrypted = publicKey.encrypt(password, 'RSA-OAEP', {
      md: forge.md.sha256.create(),
    });
    return forge.util.encode64(encrypted);
  }

  private static normalize(raw: LoginResponse): NormalizedLoginResponse {
    return {
      token: raw.token,
      user: {
        id: raw.user_id,
        username: raw.username,
        two_factor_enabled: raw.two_factor_enabled,
        force_password_change: raw.force_password_change,
      },
    };
  }

  static async register(username: string, password: string, email?: string): Promise<NormalizedLoginResponse> {
    const publicKey = await this.getPublicKey();
    const encryptedPassword = await this.encryptPassword(password, publicKey);

    const response = await fetch(`${API_BASE}/register`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ username, encrypted_password: encryptedPassword, email }),
    });

    if (!response.ok) {
      const error = await response.text();
      throw new Error(error || 'Registration failed');
    }

    return this.normalize(await response.json());
  }

  static async login(username: string, password: string, totpCode?: string): Promise<NormalizedLoginResponse> {
    const publicKey = await this.getPublicKey();
    const encryptedPassword = await this.encryptPassword(password, publicKey);

    const body: Record<string, string> = { username, encrypted_password: encryptedPassword };
    if (totpCode) {
      body.two_factor_code = totpCode;
    }

    const response = await fetch(`${API_BASE}/login`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(body),
    });

    if (response.status === 403) {
      throw new Error('2FA_REQUIRED');
    }

    if (!response.ok) {
      const error = await response.text();
      throw new Error(error || 'Login failed');
    }

    return this.normalize(await response.json());
  }

  static async logout(token: string): Promise<void> {
    await fetch(`${API_BASE}/logout`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        Authorization: `Bearer ${token}`,
      },
    });
  }
}
